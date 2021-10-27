use tokio_postgres::NoTls;
use twitch_irc::message::{AsRawIRC, PrivmsgMessage};

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
        let table_name = &format!("user_{}", msg.channel_login.to_lowercase());

        let timestamp = &msg.server_timestamp.timestamp();
        let user_id: &i32 = &msg.sender.id.parse().unwrap();
        let username = &msg.sender.login;
        let message = &msg.message_text;
        let raw_irc_message = &msg.source.as_raw_irc();

        let insert_statement = format!(
            "INSERT INTO {} (timestamp, user_id, username, message, raw_irc_message) VALUES ($1, $2, $3, $4, $5)",
            table_name
        );

        if let Err(_) = self
            .client
            .execute(
                &insert_statement[..],
                &[timestamp, user_id, username, message, raw_irc_message],
            )
            .await
        {
            let create_statement = format!(
                "CREATE TABLE {} (timestamp bigint, user_id int, username TEXT, message TEXT, raw_irc_message TEXT)",
                table_name
            );

            self.client
                .execute(&create_statement[..], &[])
                .await
                .unwrap();

            self.client
                .execute(
                    &insert_statement[..],
                    &[timestamp, user_id, username, message, raw_irc_message],
                )
                .await
                .unwrap();
        }
    }
}
