use chrono::Utc;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

use tokio::sync::Mutex;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient};

use crate::commandhandler::CommandHandler;
use crate::database::DBController;
use crate::messenger::Messenger;

pub struct BorrowBot {
    irc_stream: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<ServerMessage>>>,
    db: Arc<DBController>,
    commands: Arc<CommandHandler>,
    messenger: Arc<Messenger>,
    current_channels: Arc<Mutex<HashSet<String>>>,
    pub start_time: chrono::naive::NaiveTime,
}

impl BorrowBot {
    pub async fn new() -> Self {
        let name = "borrowbot".to_owned();
        let oauth =
            env::var("BORROWBOT_OAUTH").expect("Couldn't find environment variable for bot oauth");

        let config = ClientConfig::new_simple(StaticLoginCredentials::new(name, Some(oauth)));

        let (irc_stream, irc_client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        let db = Arc::new(DBController::new().await);
        let commands = Arc::new(CommandHandler::new());
        let messenger = Arc::new(Messenger::new(irc_client));
        let current_channels = Arc::new(Mutex::new(db.get_current_channels().await));
        let start_time = Utc::now().time();

        Self {
            irc_stream: Arc::new(Mutex::new(irc_stream)),
            db,
            commands,
            messenger,
            current_channels,
            start_time,
        }
    }

    pub fn messenger(&self) -> Arc<Messenger> {
        Arc::clone(&self.messenger)
    }

    pub fn stream(&self) -> Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<ServerMessage>>> {
        Arc::clone(&self.irc_stream)
    }

    pub fn db(&self) -> Arc<DBController> {
        Arc::clone(&self.db)
    }

    pub fn commands(&self) -> Arc<CommandHandler> {
        Arc::clone(&self.commands)
    }

    pub fn current_channels(&self) -> Arc<Mutex<HashSet<String>>> {
        Arc::clone(&self.current_channels)
    }

    pub async fn run(bot_self: Arc<BorrowBot>) {
        let bot = Arc::clone(&bot_self);
        bot.messenger().sender_loop();

        let join_handle = tokio::spawn(async move {
            while let Some(raw_message) = bot.stream().lock().await.recv().await {
                if let ServerMessage::Privmsg(msg) = raw_message {
                    if msg.message_text.starts_with("&") {
                        let bot = Arc::clone(&bot);
                        let messenger = bot.messenger();
                        let db = bot.db();
                        let commands = bot.commands();

                        tokio::spawn(async move {
                            let user_context = Arc::new(db.get_user_or_insert(&msg).await);
                            let command_response = commands.execute(bot, &user_context, &msg).await;
                            messenger
                                .chat_response(&msg, &user_context, &command_response)
                                .await;
                        });
                    }
                }
            }
        });

        let current_channels_mutex = bot_self.current_channels();
        let current_channels_guard = current_channels_mutex.lock().await;
        bot_self
            .messenger()
            .client()
            .set_wanted_channels((*current_channels_guard).clone());

        //bot_self
        //    .messenger()
        //    .send_join_messages(&(*current_channels_guard))
        //    .await;
        drop(current_channels_guard);

        join_handle.await.unwrap();
    }
}
