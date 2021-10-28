use std::collections::HashSet;
use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, Utc};
use twitch_irc::message::PrivmsgMessage;

use crate::bot::BorrowBot;
use crate::types::{CommandResponse, PermissionLevel, UserContext};

pub struct Command {
    pub about: String,
    pub permission_needed: PermissionLevel,

    // user cooldown denoted in seconds
    pub user_cooldown: u64,
}

impl Command {
    pub fn new(about: String, permission_needed: PermissionLevel, user_cooldown: u64) -> Self {
        Command {
            about,
            permission_needed,
            user_cooldown,
        }
    }

    pub async fn lookup_and_run(
        &self,
        source_function: &str,
        privmsg: &PrivmsgMessage,
        params: std::str::Split<'_, char>,
        source_bot: Arc<BorrowBot>,
        user_context: &UserContext,
    ) -> CommandResponse {
        match source_function {
            "help" => help(params, source_bot, user_context).await,
            "ping" => ping(params, source_bot, user_context).await,
            "bot" => bot(params, source_bot, user_context).await,
            "greeting" => greeting(params, source_bot, user_context).await,
            "expensive" => expensive(params, source_bot, user_context).await,
            "setpermissions" => setpermissions(params, source_bot, user_context).await,
            "join" => join(params, source_bot, user_context).await,
            "leave" => leave(params, source_bot, user_context).await,
            "uid" => uid(params, source_bot, user_context).await,
            "say" => say(params, source_bot, user_context).await,
            "lastmessage" => lastmessage(privmsg, params, source_bot, user_context).await,
            _ => CommandResponse::new("".to_owned(), false),
        }
    }
}

async fn help(
    mut params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    _: &UserContext,
) -> CommandResponse {
    let target_command = params.next().unwrap_or("").to_lowercase();
    let command_list = &bot.commands().command_list;

    let response = if !target_command.is_empty() {
        match command_list.get(&target_command) {
            Some(command) => {
                format!("&{}: {}", target_command, command.about)
            }
            None => "Sorry, I don't know that command".to_owned(),
        }
    } else {
        let mut response = String::from("List of available commands: ");
        for (command_name, _) in command_list {
            response.push_str(command_name);
            response.push_str(", ");
        }
        response.truncate(response.len() - 2);

        response
    };

    CommandResponse {
        response,
        questionable_output: false,
    }
}

async fn ping(
    _: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    _: &UserContext,
) -> CommandResponse {
    let uptime = chrono::Utc::now().time() - bot.start_time;

    let days = uptime.num_days();
    let hours = uptime.num_hours() - (days * 24);
    let minutes = uptime.num_minutes() - ((days * 1440) + (hours * 60));
    let seconds = uptime.num_seconds() - ((days * 86400) + (hours * 3600) + (minutes * 60));

    let response = format!(
        "Pong! Uptime: {}d, {}h, {}m, {}s",
        days, hours, minutes, seconds
    );

    CommandResponse {
        response,
        questionable_output: false,
    }
}

async fn bot(_: std::str::Split<'_, char>, _: Arc<BorrowBot>, _: &UserContext) -> CommandResponse {
    let response = String::from(
        "Bot made my 1xelerate. \
        Written in Rust with Tokio, Postgresql, and Rander's Twitch IRC library.",
    );

    CommandResponse {
        response,
        questionable_output: false,
    }
}

async fn greeting(
    _: std::str::Split<'_, char>,
    _: Arc<BorrowBot>,
    user_context: &UserContext,
) -> CommandResponse {
    let response = match user_context.permissions {
        PermissionLevel::Superuser => "Greetings superuser".to_owned(),
        PermissionLevel::Moderator => "Hello moderator".to_owned(),
        PermissionLevel::User => "What's good".to_owned(),
    };

    CommandResponse {
        response,
        questionable_output: false,
    }
}

async fn expensive(
    _: std::str::Split<'_, char>,
    _: Arc<BorrowBot>,
    _: &UserContext,
) -> CommandResponse {
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let response = "Test expensive command finished".to_owned();

    CommandResponse {
        response,
        questionable_output: false,
    }
}

// raw manipulation of data columns and value inside postgres database
// only available to superusers, but still prone to human error
async fn setpermissions(
    mut params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    _: &UserContext,
) -> CommandResponse {
    let target_user = params.next().unwrap_or("").to_lowercase();
    if target_user.is_empty() {
        return CommandResponse {
            response: "Please provide a username after the set command!".to_owned(),
            questionable_output: false,
        };
    }

    let target_value: i32 = params.next().unwrap_or("").parse().unwrap_or(-1);
    if target_value == -1 {
        return CommandResponse {
            response: "Error parsing value to be set, please give an integer after the username!"
                .to_owned(),
            questionable_output: false,
        };
    }

    let response = match bot
        .db()
        .try_set_column_by_name(
            &target_user.to_owned(),
            &"permissions".to_owned(),
            &target_value,
        )
        .await
    {
        Ok(rows) => {
            if rows == 0 {
                return CommandResponse {
                    response: "Sorry, that user wasn't found in my database!".to_owned(),
                    questionable_output: false,
                };
            }

            format!(
                "Succesfully set {}'s {} column to {}",
                &target_user, "permissions", &target_value
            )
        }
        Err(err) => {
            println!("{:?}", err);
            format!(
                "Error setting column {} to value {}",
                "permissions", &target_value
            )
        }
    };

    CommandResponse {
        response,
        questionable_output: false,
    }
}

async fn join(
    mut params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    _: &UserContext,
) -> CommandResponse {
    let target_channel = params.next().unwrap_or("").to_lowercase();
    if target_channel.is_empty() {
        return CommandResponse {
            response: "Please provide a channel to join".to_owned(),
            questionable_output: false,
        };
    }

    if let Ok(resp) = bot
        .api()
        .helix()
        .get_user_by_login(&target_channel[..])
        .await
    {
        if let None = resp {
            return CommandResponse {
                response: "Sorry, I couldn't find that channel".to_owned(),
                questionable_output: false,
            };
        }
    } else {
        return CommandResponse {
            response: "Unable to verify if that channel exists, join aborted; Twitch API error"
                .to_owned(),
            questionable_output: false,
        };
    }

    bot.db()
        .modify_or_insert_joined_value(&target_channel, true)
        .await;

    let current_channels_mutex = bot.current_channels();
    let mut current_channels_guard = current_channels_mutex.lock().await;

    if (*current_channels_guard).contains(&target_channel) {
        return CommandResponse {
            response: "I've already joined that channel".to_owned(),
            questionable_output: false,
        };
    }

    (*current_channels_guard).insert(target_channel.clone());
    bot.messenger()
        .client()
        .set_wanted_channels((*current_channels_guard).clone());
    drop(current_channels_guard);

    let mut new_joined_channel = HashSet::new();
    new_joined_channel.insert(target_channel.clone());
    bot.messenger()
        .send_join_messages(&new_joined_channel)
        .await;

    CommandResponse {
        response: "Succesfully joined channel".to_owned(),
        questionable_output: false,
    }
}

async fn leave(
    mut params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    _: &UserContext,
) -> CommandResponse {
    // TODO: VERIFY IF CHANNEL EXISTS?
    let target_channel = params.next().unwrap_or("").to_lowercase();
    if target_channel.is_empty() {
        return CommandResponse {
            response: "Please provide a channel to leave".to_owned(),
            questionable_output: false,
        };
    }

    bot.db()
        .modify_or_insert_joined_value(&target_channel, false)
        .await;

    let current_channels_mutex = bot.current_channels();
    let mut current_channels_guard = current_channels_mutex.lock().await;

    if !(*current_channels_guard).contains(&target_channel) {
        return CommandResponse {
            response: "I'm not currently in that channel!".to_owned(),
            questionable_output: false,
        };
    }

    (*current_channels_guard).remove(&target_channel);
    bot.messenger()
        .client()
        .set_wanted_channels((*current_channels_guard).clone());
    drop(current_channels_guard);

    // leave message?

    CommandResponse {
        response: "Succesfully left channel".to_owned(),
        questionable_output: false,
    }
}

async fn uid(
    mut params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    user_context: &UserContext,
) -> CommandResponse {
    let target_user = params.next().unwrap_or("").to_lowercase();
    if target_user.is_empty() {
        return CommandResponse {
            response: format!("{}", user_context.uid),
            questionable_output: false,
        };
    }

    let response = if let Ok(resp) = bot.api().helix().get_user_by_login(&target_user[..]).await {
        if let Some(user) = resp {
            format!("{}", user.id)
        } else {
            "Sorry, I couldn't find user".to_owned()
        }
    } else {
        "Unable to fetch uid; Twitch API error".to_owned()
    };

    CommandResponse {
        response,
        questionable_output: false,
    }
}

async fn say(
    params: std::str::Split<'_, char>,
    _: Arc<BorrowBot>,
    _: &UserContext,
) -> CommandResponse {
    let mut phrase = String::new();
    for word in params {
        phrase.push_str(word);
        phrase.push(' ');
    }

    CommandResponse {
        response: phrase,
        questionable_output: true,
    }
}

async fn lastmessage(
    privmsg: &PrivmsgMessage,
    mut params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    user: &UserContext,
) -> CommandResponse {
    let mut target_user = params.next().unwrap_or("").to_lowercase();
    if target_user.is_empty() {
        target_user = user.login.clone();
    }

    let mut target_channel = params.next().unwrap_or("").to_lowercase();
    if target_channel.is_empty() {
        target_channel = privmsg.channel_login.to_lowercase();
    }

    if let Some((timestamp, message)) = bot
        .logs()
        .get_last_message_from_username(&target_channel, &target_user)
        .await
    {
        let naive = NaiveDateTime::from_timestamp(timestamp, 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let message = format!(
            "({}) {}: {}",
            datetime.format("%Y-%m-%d %H:%M"),
            target_user,
            message
        );
        CommandResponse {
            response: message,
            questionable_output: true,
        }
    } else {
        CommandResponse {
            response: "Sorry, I didn't find any logs for that user in the selected channel!"
                .to_owned(),
            questionable_output: false,
        }
    }
}
