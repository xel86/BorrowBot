use std::collections::HashSet;

use tokio_postgres::NoTls;
use twitch_irc::message::PrivmsgMessage;

use crate::types::UserContext;

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
            .query("SELECT channel FROM channels", &[])
            .await
            .unwrap();

        let mut current_channels = HashSet::new();
        for row in &rows {
            current_channels.insert(row.get(0));
        }

        current_channels
    }

    pub async fn insert_new_channel(&self, channel: &String) {
        self.client
            .execute(
                "INSERT INTO channels (channel) VALUES ($1) ON CONFLICT DO NOTHING",
                &[channel],
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
