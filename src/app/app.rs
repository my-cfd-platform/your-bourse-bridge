use std::{collections::HashMap, sync::Arc};

use my_tcp_sockets::TcpClient;
use rust_extensions::AppStates;
use tokio::sync::Mutex;

use crate::{
    setup_price_tcp_server, FixMessageHandler, FixMessageSerializer, LogonCreeds, SettingsModel,
    TcpConnection,
};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub struct AppContext {
    pub app_states: Arc<AppStates>,
    pub settings: Arc<SettingsModel>,
    pub connections: Mutex<HashMap<i32, Arc<TcpConnection>>>,
    pub tcp_client: TcpClient,
}

impl AppContext {
    pub fn new(settings: Arc<SettingsModel>) -> AppContext {
        let tcp_client = TcpClient::new("yourbourse - fix-client".to_string(), settings.clone());

        AppContext {
            app_states: Arc::new(AppStates::create_initialized()),
            settings,
            connections: Mutex::new(HashMap::new()),
            tcp_client,
        }
    }
}

pub async fn setup_and_start(app: &Arc<AppContext>) {
    let settings = app.settings.clone();
    let app_to_spawn = app.clone();

    let fix_auth_creads = LogonCreeds {
        password: settings.your_bourse_pass.clone(),
        sender: settings.your_bourse_sender_company_id.clone(),
        target: settings.your_bourse_target_company_id.clone(),
    };

    app.tcp_client
        .start(
            Arc::new(move || -> FixMessageSerializer {
                FixMessageSerializer::new(fix_auth_creads.clone())
            }),
            Arc::new(FixMessageHandler::new(app_to_spawn)),
            my_logger::LOGGER.clone(),
        )
        .await;

    let tcp_server = setup_price_tcp_server(&app);
    tcp_server.start().await;

    app.app_states.set_initialized();
}
