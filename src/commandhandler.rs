use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use twitch_irc::message::PrivmsgMessage;

use crate::bot::BorrowBot;
use crate::commands::Command;
use crate::types::{PermissionLevel, UserContext};

// go function pointer route ?
//type AsyncCommandFunction = Arc<
//    dyn Fn(
//        Arc<std::str::Split<'static, char>>,
//        Arc<BorrowBot>,
//        Arc<UserContext>,
//    ) -> Pin<Arc<dyn Future<Output = String> + Send + Sync>>,
//>;

pub struct CommandHandler {
    command_list: HashMap<String, Command>,
}

impl CommandHandler {
    pub fn new() -> Self {
        let mut command_list: HashMap<String, Command> = HashMap::new();
        command_list.insert(
            "ping".to_owned(),
            Command::new(
                PermissionLevel::User,
                Duration::from_secs(5),
                Duration::from_secs(5),
                "ping".to_owned(),
            ),
        );
        command_list.insert(
            "bot".to_owned(),
            Command::new(
                PermissionLevel::User,
                Duration::from_secs(5),
                Duration::from_secs(5),
                "bot_about".to_owned(),
            ),
        );
        command_list.insert(
            "greeting".to_owned(),
            Command::new(
                PermissionLevel::User,
                Duration::from_secs(5),
                Duration::from_secs(5),
                "greeting".to_owned(),
            ),
        );
        command_list.insert(
            "expensive".to_owned(),
            Command::new(
                PermissionLevel::User,
                Duration::from_secs(5),
                Duration::from_secs(5),
                "test_expensive".to_owned(),
            ),
        );
        command_list.insert(
            "setpermissions".to_owned(),
            Command::new(
                PermissionLevel::Superuser,
                Duration::from_secs(5),
                Duration::from_secs(5),
                "setpermissions".to_owned(),
            ),
        );

        CommandHandler { command_list }
    }

    pub async fn execute(
        &self,
        bot: Arc<BorrowBot>,
        user_context: &UserContext,
        msg: &PrivmsgMessage,
    ) -> String {
        let mut split = msg.message_text.split(' ');
        let command_name = &split.next().unwrap()[2..];

        if let Some(command) = self.command_list.get(command_name) {
            command.lookup_and_run(split, bot, user_context).await
        } else {
            "".to_owned()
        }
    }
}
