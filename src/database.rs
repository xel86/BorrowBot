use std::collections::{HashMap, HashSet};

use tokio_postgres::NoTls;
use twitch_irc::message::PrivmsgMessage;

use crate::commands::Command;
use crate::types::{PermissionLevel, UserContext};

pub struct DBController {
    client: tokio_postgres::Client,
}

impl DBController {
    pub async fn new() -> Self {
        let (client, connection) =
            tokio_postgres::connect("host=localhost user=postgres dbname=testmandb", NoTls)
                .await
                .unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        DBController { client }
    }

    pub async fn get_current_channels(&self) -> HashSet<String> {
        let rows = self
            .client
            .query("SELECT channel FROM channels WHERE joined = true", &[])
            .await
            .unwrap();

        let mut current_channels = HashSet::new();
        for row in &rows {
            current_channels.insert(row.get(0));
        }

        current_channels
    }

    pub async fn get_current_commands(&self) -> HashMap<String, Command> {
        let rows = self
            .client
            .query("SELECT * FROM commands", &[])
            .await
            .unwrap();

        let mut current_commands = HashMap::new();
        for row in &rows {
            let command_name: String = row.get(0);
            let about: String = row.get(1);
            let permission_needed = PermissionLevel::new(row.get(2));
            let user_cooldown: i32 = row.get(3);

            current_commands.insert(
                command_name,
                Command::new(about, permission_needed, user_cooldown as u64),
            );
        }

        current_commands
    }

    // Used for join & leave commands, if joining a channel that is not in the database already, it
    // will insert it with the value of true for joined
    pub async fn modify_or_insert_joined_value(&self, channel: &String, new_joined_value: bool) {
        self.client
            .execute(
                "INSERT INTO channels (channel, joined) VALUES ($1, $2) \
                ON CONFLICT (channel) DO UPDATE SET joined = $2",
                &[channel, &new_joined_value],
            )
            .await
            .unwrap();
    }

    pub async fn get_user_or_insert(&self, msg: &PrivmsgMessage) -> UserContext {
        let uid = msg.sender.id.parse().unwrap();
        match self.get_user_by_uid(uid).await {
            Some(user) => user,
            None => {
                self.client
                    .execute(
                        "INSERT INTO users (uid, username, permissions) VALUES ($1, $2, $3)",
                        &[&uid, &msg.sender.login, &0],
                    )
                    .await
                    .unwrap();

                UserContext::new(uid, msg.sender.login.clone(), 0)
            }
        }
    }

    pub async fn get_user_by_uid(&self, uid: i32) -> Option<UserContext> {
        if let Ok(user) = self
            .client
            .query_one("SELECT * FROM users WHERE uid = $1", &[&uid])
            .await
        {
            return Some(UserContext::new(user.get(0), user.get(1), user.get(2)));
        }

        None
    }

    pub async fn get_user_by_name(&self, name: &String) -> Option<UserContext> {
        if let Ok(user) = self
            .client
            .query_one(
                "SELECT * FROM users WHERE username = $1",
                &[&name.to_lowercase()],
            )
            .await
        {
            return Some(UserContext::new(user.get(0), user.get(1), user.get(2)));
        }

        None
    }

    pub async fn try_set_column_by_name<T: tokio_postgres::types::ToSql + std::marker::Sync>(
        &self,
        name: &String,
        column: &String,
        value: &T,
    ) -> Result<u64, tokio_postgres::Error> {
        let query = format!("UPDATE users SET {} = $1 WHERE username = $2", column);
        self.client.execute(query.as_str(), &[value, name]).await
    }
}
