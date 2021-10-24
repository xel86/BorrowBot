use std::sync::Arc;

use borrowbot::bot::BorrowBot;

#[tokio::main]
async fn main() {
    let bot = Arc::new(BorrowBot::new().await);

    BorrowBot::run(bot).await;
}
