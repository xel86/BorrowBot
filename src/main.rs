use std::collections::HashSet;
use std::sync::Arc;

use borrowbot::bot::BorrowBot;

#[tokio::main]
async fn main() {
    let bot = Arc::new(BorrowBot::new().await);

    let mut wanted_channels: HashSet<String> = HashSet::new();
    wanted_channels.insert("1xelerate".to_owned());
    wanted_channels.insert("pajlada".to_owned());

    BorrowBot::run(bot, wanted_channels).await;
}
