use std::collections::HashMap;
use std::sync::Arc;

use twitch_irc::message::PrivmsgMessage;

use crate::bot::BorrowBot;
use crate::database::DBController;
use crate::types::{PermissionLevel, UserContext};

pub struct Command {}

impl Command {
    fn ping(bot: Arc<BorrowBot>) -> String {
        let uptime = chrono::Utc::now().time() - bot.start_time;

        let days = uptime.num_days();
        let hours = uptime.num_hours() - (days * 24);
        let minutes = uptime.num_minutes() - (hours * 60);
        let seconds = uptime.num_seconds() - (minutes * 60);
        format!(
            "Pong! Uptime: {}d, {}h, {}m, {}s",
            days, hours, minutes, seconds
        )
    }

    fn about() -> String {
        String::from("Bot made my 1xelerate. Written in Rust with Tokio, Postgresql, and Rander's Twitch IRC library.")
    }

    fn greeting(permission_level: &PermissionLevel) -> String {
        match permission_level {
            PermissionLevel::Superuser => "Greetings superuser".to_owned(),
            PermissionLevel::Moderator => "Hello moderator".to_owned(),
            PermissionLevel::User => "What's good".to_owned(),
        }
    }

    async fn test_expensive_command(permission_level: &PermissionLevel) -> String {
        if let PermissionLevel::Superuser = permission_level {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            "Test expensive command finished".to_owned()
        } else {
            "You don't have the permission to do that!".to_owned()
        }
    }

    // raw manipulation of data columns and value inside postgres database
    // only available to superusers, but still prone to human error
    async fn setpermissions(
        params: &mut std::str::Split<'_, char>,
        db: Arc<DBController>,
        permission_level: &PermissionLevel,
    ) -> String {
        if let PermissionLevel::Moderator | PermissionLevel::User = permission_level {
            return "Sorry, only superusers have access to the set commands!".to_owned();
        }

        let target_user = params.next().unwrap_or("").to_lowercase();
        if target_user.is_empty() {
            return "Please provide a username after the set command!".to_owned();
        }

        let target_value: i32 = params.next().unwrap_or("").parse().unwrap_or(-1);
        if target_value == -1 {
            return "Error parsing value to be set, please give an integer after the username!"
                .to_owned();
        }

        match db
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
}

pub struct CommandHandler {}

impl CommandHandler {
    pub fn new() -> Self {
        let mut user_permission_map: HashMap<String, PermissionLevel> = HashMap::new();
        user_permission_map.insert("140114344".to_owned(), PermissionLevel::Superuser);

        CommandHandler {}
    }

    pub async fn execute(
        &self,
        bot: Arc<BorrowBot>,
        user_context: &UserContext,
        msg: &PrivmsgMessage,
    ) -> String {
        let mut split = msg.message_text.split(' ');
        let command_name = &split.next().unwrap()[2..];

        match command_name.to_lowercase().as_str() {
            "ping" => Command::ping(bot),
            "about" => Command::about(),
            "greeting" => Command::greeting(&user_context.permissions),
            "expensive" => Command::test_expensive_command(&user_context.permissions).await,
            "setpermissions" => {
                Command::setpermissions(&mut split, bot.db(), &user_context.permissions).await
            }
            _ => "".to_owned(),
        }
    }
}
