use std::collections::HashMap;
use std::sync::{Arc, RwLock};
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
    pub command_list: HashMap<String, Command>,
    user_cooldowns: Arc<RwLock<Vec<(i32, String)>>>,
}

impl CommandHandler {
    pub fn new() -> Self {
        let mut command_list: HashMap<String, Command> = HashMap::new();
        command_list.insert(
            "help".to_owned(),
            Command::new(
                "With no arguments provided, returns all available commands. \
                If provided with a name of a command it will return a short summary of the command"
                    .to_owned(),
                PermissionLevel::User,
                5,
            ),
        );
        command_list.insert(
            "ping".to_owned(),
            Command::new(
                "Returns the a message indicating the bot is running, \
                and some statistics about the bot"
                    .to_owned(),
                PermissionLevel::User,
                5,
            ),
        );
        command_list.insert(
            "bot".to_owned(),
            Command::new(
                "Returns information about the bot, such as the author and language it was made in"
                    .to_owned(),
                PermissionLevel::User,
                5,
            ),
        );
        command_list.insert(
            "greeting".to_owned(),
            Command::new(
                "Returns a greeting based on your permission level".to_owned(),
                PermissionLevel::User,
                5,
            ),
        );
        command_list.insert(
            "expensive".to_owned(),
            Command::new(
                "Runs a command that sleeps for 5 seconds to test await functionality".to_owned(),
                PermissionLevel::Moderator,
                5,
            ),
        );
        command_list.insert(
            "setpermissions".to_owned(),
            Command::new(
                "Sets the permission value for a user in the postgresql database \
                associated with the bot"
                    .to_owned(),
                PermissionLevel::Superuser,
                5,
            ),
        );
        command_list.insert(
            "join".to_owned(),
            Command::new(
                "Makes BorrowBot join a channel, which persists after restart".to_owned(),
                PermissionLevel::Superuser,
                5,
            ),
        );
        command_list.insert(
            "leave".to_owned(),
            Command::new(
                "Makes BorrowBot leave a channel, which persists after restart".to_owned(),
                PermissionLevel::Superuser,
                5,
            ),
        );

        let user_cooldowns = Arc::new(RwLock::new(Vec::new()));

        CommandHandler {
            command_list,
            user_cooldowns,
        }
    }

    pub async fn execute(
        &self,
        bot: Arc<BorrowBot>,
        user_context: &UserContext,
        msg: &PrivmsgMessage,
    ) -> String {
        let mut split = msg.message_text.split(' ');
        let command_name = &split.next().unwrap()[1..];

        if let Some(command) = self.command_list.get(command_name) {
            if !user_context
                .permissions
                .satisfies(command.permission_needed)
            {
                return format!(
                    "Sorry, only {}s have access to the {} command",
                    command.permission_needed, command_name
                );
            }

            if !self
                .user_cooldowns
                .read()
                .unwrap()
                .contains(&(user_context.uid, command_name.to_owned()))
            {
                let response = command
                    .lookup_and_run(command_name, split, bot, user_context)
                    .await;

                if user_context.permissions != PermissionLevel::Superuser {
                    self.start_user_cooldown(
                        user_context.uid,
                        command_name.to_owned(),
                        command.user_cooldown,
                    )
                    .await;
                }

                return response;
            } else {
                "".to_owned()
            }
        } else {
            "".to_owned()
        }
    }

    // pushes the user's id and the command as a tuple into a vector within a RwLock, once the
    // cooldown period is up for the command we will remove that tuple from the vector
    // to indicate that user is not longer on cooldown for that command
    async fn start_user_cooldown(&self, uid: i32, command_name: String, cooldown: u64) {
        let cooldowns = Arc::clone(&self.user_cooldowns);
        tokio::spawn(async move {
            {
                cooldowns.write().unwrap().push((uid, command_name.clone()));
            }
            tokio::time::sleep(Duration::from_secs(cooldown)).await;
            {
                cooldowns
                    .write()
                    .unwrap()
                    .retain(|(id, command)| id != &uid && command != &command_name);
            }
        });
    }
}
