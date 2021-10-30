use core::convert::TryFrom;
use tokio_postgres::NoTls;
use twitch_irc::message::{AsRawIRC, IRCMessage, PrivmsgMessage};

pub struct LogController {
    client: tokio_postgres::Client,
}

impl LogController {
    pub async fn new() -> Self {
        let (client, connection) =
            tokio_postgres::connect("host=localhost user=postgres dbname=logs", NoTls)
                .await
                .unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        LogController { client }
    }

    pub async fn log_message(&self, msg: &PrivmsgMessage) {
        let table_name = &format!("channel_{}", msg.channel_login.to_lowercase());

        let timestamp = &msg.server_timestamp.timestamp();
        let user_id: &i32 = &msg.sender.id.parse().unwrap();
        let username = &msg.sender.login;
        let message = &msg.source.as_raw_irc();

        let insert_statement = format!(
            "INSERT INTO {} (timestamp, user_id, username, message) VALUES ($1, $2, $3, $4)",
            table_name
        );

        if let Err(_) = self
            .client
            .execute(
                &insert_statement[..],
                &[timestamp, user_id, username, message],
            )
            .await
        {
            let create_statement = format!(
                "CREATE TABLE {} (timestamp bigint, user_id int, username TEXT, message TEXT)",
                table_name
            );

            self.client
                .execute(&create_statement[..], &[])
                .await
                .unwrap();

            self.client
                .execute(
                    &insert_statement[..],
                    &[timestamp, user_id, username, message],
                )
                .await
                .unwrap();
        }
    }

    pub async fn get_last_message_from_username(
        &self,
        channel: &String,
        username: &String,
    ) -> Option<(i64, String)> {
        let table_name = &format!("channel_{}", channel.to_lowercase());

        let query = format!(
            "SELECT timestamp, message FROM {} WHERE username = $1 ORDER BY timestamp DESC LIMIT 1",
            table_name
        );

        if let Ok(row) = self
            .client
            .query_one(&query[..], &[&username.to_lowercase()])
            .await
        {
            let timestamp: i64 = row.get(0);
            let message: String = row.get(1);
            let message = PrivmsgMessage::try_from(IRCMessage::parse(&message[..]).unwrap())
                .unwrap()
                .message_text;
            Some((timestamp, message))
        } else {
            None
        }
    }

    pub async fn get_random_message_from_username(
        &self,
        channel: &String,
        username: &String,
    ) -> Option<(i64, String)> {
        let table_name = &format!("channel_{}", channel.to_lowercase());

        let query = format!(
            "SELECT timestamp, message FROM {} WHERE username = $1 AND timestamp \
            >= (SELECT random()*(max(timestamp)-min(timestamp)) + min(timestamp) FROM {}) ORDER BY timestamp LIMIT 1",
            table_name, table_name
        );

        if let Ok(row) = self
            .client
            .query_one(&query[..], &[&username.to_lowercase()])
            .await
        {
            let timestamp: i64 = row.get(0);
            let message: String = row.get(1);
            let message = PrivmsgMessage::try_from(IRCMessage::parse(&message[..]).unwrap())
                .unwrap()
                .message_text;
            Some((timestamp, message))
        } else {
            None
        }
    }

    pub async fn get_random_message(&self, channel: &String) -> Option<(i64, String, String)> {
        let table_name = &format!("channel_{}", channel.to_lowercase());

        let query = format!(
            "SELECT timestamp, username, message FROM {} WHERE timestamp \
            >= (SELECT random()*(max(timestamp)-min(timestamp)) + min(timestamp) FROM {}) ORDER BY timestamp LIMIT 1",
            table_name, table_name
        );

        if let Ok(row) = self.client.query_one(&query[..], &[]).await {
            let timestamp: i64 = row.get(0);
            let username: String = row.get(1);
            let message: String = row.get(2);
            let message = PrivmsgMessage::try_from(IRCMessage::parse(&message[..]).unwrap())
                .unwrap()
                .message_text;
            Some((timestamp, username, message))
        } else {
            None
        }
    }
}
