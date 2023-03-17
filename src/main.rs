use my_seq_logger::SeqLogger;
use std::sync::Arc;
use your_bourse_bridge::{setup_and_start, AppContext, SettingsModel, APP_NAME, APP_VERSION};

#[tokio::main]
async fn main() {
    let (app_name, app_version) = (APP_NAME, APP_VERSION);
    let settings = Arc::new(SettingsModel::load(".your-fin").await);

    SeqLogger::enable_from_connection_string(
        APP_NAME.to_string(),
        APP_VERSION.to_string(),
        settings.clone(),
        None,
    );

    let app = Arc::new(AppContext::new(settings));
    http_is_alive_shared::start_up::start_server(
        app_name.to_string(),
        app_version.to_string(),
        app.app_states.clone(),
    );
    setup_and_start(&app).await;
    app.app_states.wait_until_shutdown().await;
}
