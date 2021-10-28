use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use twitch_irc::message::PrivmsgMessage;

use crate::bot::BorrowBot;
use crate::commands::Command;
use crate::database::DBController;
use crate::types::{CommandResponse, PermissionLevel, UserContext};

pub struct CommandHandler {
    pub command_list: HashMap<String, Command>,
    user_cooldowns: Arc<RwLock<Vec<(i32, String)>>>,
}

impl CommandHandler {
    pub async fn new(db: Arc<DBController>) -> Self {
        let command_list = db.get_current_commands().await;
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
    ) -> CommandResponse {
        let mut split = msg.message_text.split(' ');
        let command_name = &split.next().unwrap()[1..];

        if let Some(command) = self.command_list.get(command_name) {
            if !user_context
                .permissions
                .satisfies(command.permission_needed)
            {
                return CommandResponse {
                    response: format!(
                        "Sorry, only {}s have access to the {} command",
                        command.permission_needed, command_name
                    ),
                    questionable_output: false,
                };
            }

            if !self
                .user_cooldowns
                .read()
                .unwrap()
                .contains(&(user_context.uid, command_name.to_owned()))
            {
                let response = command
                    .lookup_and_run(command_name, msg, split, bot, user_context)
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
                CommandResponse {
                    response: "".to_owned(),
                    questionable_output: false,
                }
            }
        } else {
            CommandResponse {
                response: "".to_owned(),
                questionable_output: false,
            }
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
