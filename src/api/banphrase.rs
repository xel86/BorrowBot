use reqwest::Client;
use serde::Deserialize;

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

    let resp = client
        .post("https://forsen.tv/api/v1/banphrases/test")
        .query(&[("message", message)])
        .send()
        .await?
        .json::<BanphraseResponse>()
        .await?;

    Ok(resp.banned)
}
