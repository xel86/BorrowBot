use std::collections::VecDeque;
use std::sync::Arc;

use tokio::sync::Mutex;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::PrivmsgMessage;
use twitch_irc::{SecureTCPTransport, TwitchIRCClient};

use crate::types::UserContext;

pub struct Messenger {
    irc_client: Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>,
    message_queue: Arc<Mutex<VecDeque<(String, String)>>>,
    apply_same_message_modifier: Mutex<bool>,
}

impl Messenger {
    pub fn new(client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>) -> Self {
        Messenger {
            irc_client: Arc::new(client),
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
            apply_same_message_modifier: Mutex::new(false),
        }
    }

    pub fn client(&self) -> Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>> {
        Arc::clone(&self.irc_client)
    }

    pub async fn chat_response(
        &self,
        msg: &PrivmsgMessage,
        user_context: &UserContext,
        command_response: &String,
    ) {
        if command_response.is_empty() {
            return;
        }

        let mut apply = self.apply_same_message_modifier.lock().await;
        let same_message_modifier = match *apply {
            true => {
                *apply = false;
                "󠀀"
            }
            false => {
                *apply = true;
                ""
            }
        };
        drop(apply);

        let response = format!(
            "@{}, {}{}",
            user_context.login, command_response, same_message_modifier
        );

        let mut queue = self.message_queue.lock().await;
        (*queue).insert(0, (msg.channel_login.clone(), response));
    }

    // adhere to global 1 second cooldown
    pub fn sender_loop(&self) {
        let message_queue = Arc::clone(&self.message_queue);
        let irc_client = Arc::clone(&self.irc_client);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if let Some((target_user, response)) = (*message_queue.lock().await).pop_front() {
                    irc_client.say(target_user, response).await.unwrap();
                }
            }
        });
    }
}
