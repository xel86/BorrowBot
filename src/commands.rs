use std::collections::HashSet;
use std::sync::Arc;

use crate::bot::BorrowBot;
use crate::types::{PermissionLevel, UserContext};

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
        params: std::str::Split<'_, char>,
        source_bot: Arc<BorrowBot>,
        user_context: &UserContext,
    ) -> String {
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
            _ => "".to_owned(),
        }
    }
}

async fn help(
    mut params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    _: &UserContext,
) -> String {
    let target_command = params.next().unwrap_or("").to_lowercase();
    let command_list = &bot.commands().command_list;

    if !target_command.is_empty() {
        match command_list.get(&target_command) {
            Some(command) => {
                format!("&{}: {}", target_command, command.about)
            }
            None => {
                format!("Sorry, I don't know the command {}", target_command)
            }
        }
    } else {
        let mut response = String::from("List of available commands: ");
        for (command_name, _) in command_list {
            response.push_str(command_name);
            response.push_str(", ");
        }
        response.truncate(response.len() - 2);

        response
    }
}

async fn ping(_: std::str::Split<'_, char>, bot: Arc<BorrowBot>, _: &UserContext) -> String {
    let uptime = chrono::Utc::now().time() - bot.start_time;

    let days = uptime.num_days();
    let hours = uptime.num_hours() - (days * 24);
    let minutes = uptime.num_minutes() - ((days * 1440) + (hours * 60));
    let seconds = uptime.num_seconds() - ((days * 86400) + (hours * 3600) + (minutes * 60));
    format!(
        "Pong! Uptime: {}d, {}h, {}m, {}s",
        days, hours, minutes, seconds
    )
}

async fn bot(_: std::str::Split<'_, char>, _: Arc<BorrowBot>, _: &UserContext) -> String {
    String::from("Bot made my 1xelerate. Written in Rust with Tokio, Postgresql, and Rander's Twitch IRC library.")
}

async fn greeting(
    _: std::str::Split<'_, char>,
    _: Arc<BorrowBot>,
    user_context: &UserContext,
) -> String {
    match user_context.permissions {
        PermissionLevel::Superuser => "Greetings superuser".to_owned(),
        PermissionLevel::Moderator => "Hello moderator".to_owned(),
        PermissionLevel::User => "What's good".to_owned(),
    }
}

async fn expensive(_: std::str::Split<'_, char>, _: Arc<BorrowBot>, _: &UserContext) -> String {
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    "Test expensive command finished".to_owned()
}

// raw manipulation of data columns and value inside postgres database
// only available to superusers, but still prone to human error
async fn setpermissions(
    mut params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    _: &UserContext,
) -> String {
    let target_user = params.next().unwrap_or("").to_lowercase();
    if target_user.is_empty() {
        return "Please provide a username after the set command!".to_owned();
    }

    let target_value: i32 = params.next().unwrap_or("").parse().unwrap_or(-1);
    if target_value == -1 {
        return "Error parsing value to be set, please give an integer after the username!"
            .to_owned();
    }

    match bot
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
                return "Sorry, that user wasn't found in my database!".to_owned();
            }

            format!(
                "Succesfully set {}'s {} column, to {}",
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
    }
}

async fn join(
    mut params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    _: &UserContext,
) -> String {
    let target_channel = params.next().unwrap_or("").to_lowercase();
    if target_channel.is_empty() {
        return "Please provide a channel to join".to_owned();
    }

    if let Ok(resp) = bot.helix().get_user_by_login(&target_channel[..]).await {
        if let None = resp {
            return format!("Sorry, I couldn't find channel {}", target_channel);
        }
    } else {
        return "Unable to verify if that channel exists, join aborted; Twitch API error"
            .to_owned();
    }

    bot.db()
        .modify_or_insert_joined_value(&target_channel, true)
        .await;

    let current_channels_mutex = bot.current_channels();
    let mut current_channels_guard = current_channels_mutex.lock().await;

    if (*current_channels_guard).contains(&target_channel) {
        return format!("I've already joined channel {}!", target_channel);
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

    format!("Succesfully joined channel {}", target_channel)
}

async fn leave(
    mut params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    _: &UserContext,
) -> String {
    // TODO: VERIFY IF CHANNEL EXISTS?
    let target_channel = params.next().unwrap_or("").to_lowercase();
    if target_channel.is_empty() {
        return "Please provide a channel to leave".to_owned();
    }

    bot.db()
        .modify_or_insert_joined_value(&target_channel, false)
        .await;

    let current_channels_mutex = bot.current_channels();
    let mut current_channels_guard = current_channels_mutex.lock().await;

    if !(*current_channels_guard).contains(&target_channel) {
        return format!("I'm not currently in channel {}!", target_channel);
    }

    (*current_channels_guard).remove(&target_channel);
    bot.messenger()
        .client()
        .set_wanted_channels((*current_channels_guard).clone());
    drop(current_channels_guard);

    // leave message?

    format!("Succesfully left channel {}", target_channel)
}

async fn uid(
    mut params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    user_context: &UserContext,
) -> String {
    let target_user = params.next().unwrap_or("").to_lowercase();
    if target_user.is_empty() {
        return format!("{}", user_context.uid);
    }

    if let Ok(resp) = bot.helix().get_user_by_login(&target_user[..]).await {
        if let Some(user) = resp {
            format!("{}", user.id)
        } else {
            format!("Sorry, I couldn't find user {}", target_user)
        }
    } else {
        "Unable to fetch uid; Twitch API error".to_owned()
    }
}
