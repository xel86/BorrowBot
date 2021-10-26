pub mod banphrase;
pub mod helix;
pub mod supinic;

use std::sync::Arc;

use helix::Helix;
use supinic::Supinic;

pub struct APIController {
    helix: Arc<Helix>,
    supinic: Arc<Supinic>,
}

impl APIController {
    pub async fn init() -> Self {
        let helix = Arc::new(
            Helix::new()
                .await
                .expect("Error retrieving access token from twitch"),
        );

        let supinic = Arc::new(Supinic::new());

        Self { helix, supinic }
    }

    pub fn helix(&self) -> Arc<Helix> {
        Arc::clone(&self.helix)
    }

    pub fn supinic(&self) -> Arc<Supinic> {
        Arc::clone(&self.supinic)
    }
}
