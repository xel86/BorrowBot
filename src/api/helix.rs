use std::env;

use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
struct AppAccessToken {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: usize,
    scope: Option<Vec<String>>,
    token_type: String,
}

#[derive(Deserialize)]
pub struct GetUsersResponse {
    data: Vec<User>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct User {
    pub id: String,
    pub login: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub view_count: i32,
    pub email: Option<String>,
    pub created_at: String,
}

pub struct Helix {
    client: Client,
    pub access_token: String,
}

impl Helix {
    pub async fn new() -> Result<Self, reqwest::Error> {
        let client_id =
            env::var("BORROWBOT_CLIENT_ID").expect("Couldn't find env var for bot client id");
        let client_secret = env::var("BORROWBOT_CLIENT_SECRET")
            .expect("Couldn't find env var for bot client secret");
        let access_token = match env::var("BORROWBOT_ACCESS_TOKEN") {
            Ok(token) => Some(token),
            _ => None,
        };

        let access_token = match access_token {
            Some(token) => token,
            None => Self::get_access_token(&client_id[..], &client_secret[..])
                .await
                .expect("Error getting new access token"),
        };

        let mut headers = HeaderMap::new();
        headers.insert("Client-id", client_id.parse().unwrap());
        headers.insert(
            "Authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            access_token,
        })
    }

    pub async fn get_access_token(
        client_id: &str,
        client_secret: &str,
    ) -> Result<String, reqwest::Error> {
        let client = Client::new();

        let resp = client
            .post("https://id.twitch.tv/oauth2/token")
            .query(&[
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await?
            .json::<AppAccessToken>()
            .await?;

        eprintln!("Generated new access_token: {}", resp.access_token);
        Ok(resp.access_token)
    }

    pub async fn get_user_by_login(&self, login: &str) -> Result<Option<User>, reqwest::Error> {
        let mut resp = self
            .client
            .get("https://api.twitch.tv/helix/users")
            .query(&[("login", login)])
            .send()
            .await?
            .json::<GetUsersResponse>()
            .await?;

        if resp.data.is_empty() {
            return Ok(None);
        }

        Ok(Some(resp.data.remove(0)))
    }
}
