use std::env;
use std::sync::Arc;

use reqwest::header::HeaderMap;
use reqwest::Client;

pub struct Supinic {
    client: Arc<Client>,
}

impl Supinic {
    pub fn new() -> Self {
        let user_id = env::var("SUPINIC_ID").unwrap_or_else(|_| {
            eprintln!("Failed to find supinic id env var");
            "".to_owned()
        });
        let api_key = env::var("SUPINIC_KEY").unwrap_or_else(|_| {
            eprintln!("Failed to find supinic key env var");
            "".to_owned()
        });

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Basic {}:{}", user_id, api_key).parse().unwrap(),
        );
        headers.insert(
            "User-Agent",
            format!(
                "BorrowBot - made by @1xelerate using Rust. \
            Source Code: https://github.com/bleusakura/BorrowBot"
            )
            .parse()
            .unwrap(),
        );

        let client = Arc::new(Client::builder().default_headers(headers).build().unwrap());

        Self { client }
    }

    pub async fn start_supinic_ping_loop(&self) {
        let client = Arc::clone(&self.client);
        tokio::spawn(async move {
            loop {
                if let Err(_) = Supinic::ping_supinic(&client).await {
                    format!("{}: Error pinging supinic", chrono::Utc::now());
                }
                tokio::time::sleep(std::time::Duration::from_secs(1800)).await;
            }
        });
    }

    pub async fn ping_supinic(client: &Arc<Client>) -> Result<(), reqwest::Error> {
        client
            .put("https://supinic.com/api/bot-program/bot/active")
            .send()
            .await?;

        Ok(())
    }
}
