use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct BanphraseResponse {
    pub banned: bool,
    pub input_message: String,
    pub banphrase_data: Option<BanphraseData>,
}

#[derive(Debug, Deserialize)]
pub struct BanphraseData {
    pub id: i32,
    pub name: String,
    pub phrase: String,
    pub length: i32,
    pub permanent: bool,
    pub operator: String,
    pub case_sensitive: bool,
}

pub async fn contains_banphrase(message: &str) -> Result<bool, reqwest::Error> {
    let client = Client::new();

    let mut data = HashMap::new();
    data.insert("message", message);

    let resp = client
        .post("https://forsen.tv/api/v1/banphrases/test")
        .json(&data)
        .send()
        .await?
        .json::<BanphraseResponse>()
        .await?;

    Ok(resp.banned)
}
