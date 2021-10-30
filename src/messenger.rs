use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use tokio::sync::Mutex;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::{SecureTCPTransport, TwitchIRCClient};

use crate::api::banphrase;
use crate::types::{CommandResponse, UserContext};

pub struct Messenger {
    irc_client: Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>,
    message_queue: Arc<Mutex<VecDeque<(String, String)>>>,
}

impl Messenger {
    pub fn new(client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>) -> Self {
        Messenger {
            irc_client: Arc::new(client),
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn client(&self) -> Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>> {
        Arc::clone(&self.irc_client)
    }

    pub async fn send_join_messages(&self, channels: &HashSet<String>) {
        for channel in channels {
            let mut queue = self.message_queue.lock().await;
            (*queue).insert(0, ((*channel).clone(), "🚨".to_owned()));
        }
    }

    pub async fn chat_response(
        &self,
        msg: &PrivmsgMessage,
        user_context: &UserContext,
        command_response: &CommandResponse,
    ) {
        let response = &command_response.response;
        let questionable_output = command_response.questionable_output;
        if response.is_empty() {
            return;
        }

        let response = format!("@{}, {}", user_context.login, response);

        let ensured_response = if questionable_output {
            if let Ok(is_banned) = banphrase::contains_banphrase(&response).await {
                if !is_banned {
                    response
                } else {
                    "Uh oh, the anticipated response contained a banphrase monkaS".to_owned()
                }
            } else {
                "Couldn't reach banphrase API monkaS".to_owned()
            }
        } else {
            response
        };

        let mut queue = self.message_queue.lock().await;
        (*queue).insert(0, (msg.channel_login.clone(), ensured_response));
    }

    // adhere to global 1 second cooldown
    pub fn sender_loop(&self) {
        let message_queue = Arc::clone(&self.message_queue);
        let irc_client = Arc::clone(&self.irc_client);
        tokio::spawn(async move {
            let mut apply_modifier = false;
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if let Some((target_channel, response)) = (*message_queue.lock().await).pop_front()
                {
                    let same_message_modifier = if apply_modifier {
                        apply_modifier = false;
                        " 󠀀"
                    } else {
                        apply_modifier = true;
                        ""
                    };

                    irc_client
                        .say(target_channel, response + same_message_modifier)
                        .await
                        .unwrap();
                }
            }
        });
    }
}
