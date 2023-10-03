use std::sync::Arc;
use your_bourse_bridge::{setup_and_start, AppContext, SettingsReader};

#[tokio::main]
async fn main() {
    let settings_reader = SettingsReader::new(".my-cfd-platform").await;
    let settings_reader = Arc::new(settings_reader);

    let mut service_context = service_sdk::ServiceContext::new(settings_reader.clone()).await;

    let app_context = Arc::new(AppContext::new(settings_reader, &service_context));

    setup_and_start(&app_context, &service_context).await;

    service_context.start_application().await;
}