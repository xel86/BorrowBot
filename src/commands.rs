use std::sync::Arc;

use crate::bot::BorrowBot;
use crate::types::{PermissionLevel, UserContext};

pub struct Command {
    pub permission_needed: PermissionLevel,

    // user cooldown denoted in seconds
    pub user_cooldown: u64,
}

impl Command {
    pub fn new(permission_needed: PermissionLevel, user_cooldown: u64) -> Self {
        Command {
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
            "ping" => ping(params, source_bot, user_context).await,
            "bot" => bot(params, source_bot, user_context).await,
            "greeting" => greeting(params, source_bot, user_context).await,
            "expensive" => expensive(params, source_bot, user_context).await,
            "setpermissions" => setpermissions(params, source_bot, user_context).await,
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

pub async fn bot(_: std::str::Split<'_, char>, _: Arc<BorrowBot>, _: &UserContext) -> String {
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

pub async fn expensive(_: std::str::Split<'_, char>, _: Arc<BorrowBot>, _: &UserContext) -> String {
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    "Test expensive command finished".to_owned()
}

// raw manipulation of data columns and value inside postgres database
// only available to superusers, but still prone to human error
pub async fn setpermissions(
    params: std::str::Split<'_, char>,
    bot: Arc<BorrowBot>,
    _: &UserContext,
) -> String {
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
