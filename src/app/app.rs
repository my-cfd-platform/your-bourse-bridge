use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
    thread::sleep
};

use my_tcp_sockets::TcpClient;
use rust_extensions::AppStates;
use tokio::sync::Mutex;
use my_no_sql_tcp_reader::{MyNoSqlDataReader, MyNoSqlTcpConnection};
use my_logger::MyLogger;

use crate::{
    setup_price_tcp_server,
    FixMessageHandler, FixMessageSerializer, LogonCreds,
    SettingsModel, TcpConnection, InstrumentMappingEntity,
};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const MAPPING_PK: &str = "im";

pub struct AppContext {
    pub app_states: Arc<AppStates>,
    pub settings: Arc<SettingsModel>,
    pub connections: Mutex<HashMap<i32, Arc<TcpConnection>>>,
    pub tcp_client: TcpClient,
    pub logger: Arc<MyLogger>
}

impl AppContext {
    pub fn new(settings: Arc<SettingsModel>) -> AppContext {
        let tcp_client = TcpClient::new("yourbourse - fix-client".to_string(), settings.clone());

        AppContext {
            app_states: Arc::new(AppStates::create_initialized()),
            settings,
            connections: Mutex::new(HashMap::new()),
            tcp_client,
            logger: my_logger::LOGGER.clone()
        }
    }
}

pub async fn setup_and_start(app: &Arc<AppContext>) {
    app.logger.write_log(
        my_logger::LogLevel::Info,
        String::from("Main"),
        String::from("Service is starting"),
        None,
    );

    let settings = app.settings.clone();
    let app_to_spawn = app.clone();

    let map = get_map(&settings).await;   

    // region create fix client with callback event handler
    let fix_auth_creds = LogonCreds {
        password: settings.your_bourse_pass.clone(),
        sender: settings.your_bourse_sender_company_id.clone(),
        target: settings.your_bourse_target_company_id.clone(),
    };

    app.tcp_client
        .start(
            Arc::new(move || -> FixMessageSerializer {
                FixMessageSerializer::new(fix_auth_creds.clone())
            }),
            Arc::new(FixMessageHandler::new(app_to_spawn, map)),
            my_logger::LOGGER.clone(),
        )
        .await;
    // endregion

    let tcp_server = setup_price_tcp_server(&app);
    tcp_server.start().await;

    app.logger.write_log(
        my_logger::LogLevel::Info,
        String::from("App"),
        String::from("Service is started"),
        None,
    );

    app.app_states.set_initialized();
}

async fn get_map(settings:&Arc<SettingsModel>) -> HashMap<String, Vec<String>>{

    let nosql_connection = MyNoSqlTcpConnection::new(APP_NAME.to_string(), settings.clone());
    let instruments_reader: Arc<MyNoSqlDataReader<InstrumentMappingEntity>> =
        nosql_connection.get_reader().await;
    nosql_connection.start().await;

    // Wait 5 sec for nosql_connection to initiate
    sleep(Duration::from_millis(5000));

    let map_entity_option = instruments_reader
        .get_entity(MAPPING_PK, settings.liquidity_provider_id.as_str())
        .await;
    let mut map = HashMap::<String, Vec<String>>::new();
    if map_entity_option.is_some() {
        let map_entity = map_entity_option.unwrap();
        for (our_symbol, external_symbol) in map_entity.map.iter() {
            if !map.contains_key(external_symbol.as_str()){
                map.insert(external_symbol.to_string(), Vec::new());
            }
            map.get_mut(external_symbol).unwrap().push(our_symbol.to_string());
        }
    }
    map
// endregion
}
