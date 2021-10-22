use std::sync::Arc;
use std::time::Duration;

use crate::bot::BorrowBot;
use crate::types::{PermissionLevel, UserContext};

pub struct Command {
    permission_needed: PermissionLevel,
    global_cooldown: Duration,
    user_cooldown: Duration,
    source_function: String,
}

impl Command {
    pub fn new(
        permission_needed: PermissionLevel,
        global_cooldown: Duration,
        user_cooldown: Duration,
        source_function: String,
    ) -> Self {
        Command {
            permission_needed,
            global_cooldown,
            user_cooldown,
            source_function,
        }
    }

    pub async fn lookup_and_run(
        &self,
        params: std::str::Split<'_, char>,
        bot: Arc<BorrowBot>,
        user_context: &UserContext,
    ) -> String {
        match self.source_function.as_str() {
            "ping" => ping(params, bot, user_context).await,
            "bot_about" => bot_about(params, bot, user_context).await,
            "greeting" => greeting(params, bot, user_context).await,
            "test_expensive" => test_expensive(params, bot, user_context).await,
            "setpermissions" => setpermissions(params, bot, user_context).await,
            _ => "".to_owned(),
        }
    }
}

pub async fn ping(_: std::str::Split<'_, char>, bot: Arc<BorrowBot>, _: &UserContext) -> String {
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

pub async fn bot_about(_: std::str::Split<'_, char>, _: Arc<BorrowBot>, _: &UserContext) -> String {
    String::from("Bot made my 1xelerate. Written in Rust with Tokio, Postgresql, and Rander's Twitch IRC library.")
}

pub async fn greeting(
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

pub async fn test_expensive(
    _: std::str::Split<'_, char>,
    _: Arc<BorrowBot>,
    user_context: &UserContext,
) -> String {
    if let PermissionLevel::Superuser = user_context.permissions {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        "Test expensive command finished".to_owned()
    } else {
        "You don't have the permission to do that!".to_owned()
    }
}

// raw manipulation of data columns and value inside postgres database
// only available to superusers, but still prone to human error
pub async fn setpermissions(
    params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    user_context: &UserContext,
) -> String {
    if let PermissionLevel::Moderator | PermissionLevel::User = user_context.permissions {
        return "Sorry, only superusers have access to the set commands!".to_owned();
    }

    let mut params = params;
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
