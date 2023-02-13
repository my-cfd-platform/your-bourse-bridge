use std::sync::Arc;

use your_borse_bridge::{setup_and_start, AppContext, SettingsModel};

#[tokio::main]
async fn main() {
    let settings = Arc::new(SettingsModel::load(".yourfin").await);
    let app = Arc::new(AppContext::new(settings));
    setup_and_start(&app).await;
    app.app_states.wait_until_shutdown().await;
}
