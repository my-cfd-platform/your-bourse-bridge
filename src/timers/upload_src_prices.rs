use std::sync::Arc;

use rust_extensions::MyTimerTick;

use crate::app::AppContext;

pub struct UploadSrcPricesTimer {
    app: Arc<AppContext>,
}

impl UploadSrcPricesTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for UploadSrcPricesTimer {
    async fn tick(&self) {
        let prices_to_upload = self.app.prices_cache.get_snapshot().await;

        if prices_to_upload.len() == 0 {
            return;
        }

        self.app
            .bid_ask_price_src
            .bulk_insert_or_replace(&prices_to_upload)
            .await
            .unwrap();
    }
}
