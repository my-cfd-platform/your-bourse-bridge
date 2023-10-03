use std::{collections::HashMap, sync::Arc, thread::sleep, time::Duration};

use my_tcp_sockets::TcpClient;
use service_sdk::{
    my_logger::LogEventCtx,
    my_no_sql_sdk::reader::{MyNoSqlDataReaderTcp, MyNoSqlTcpConnection},
    ServiceContext,
};
use tokio::sync::Mutex;

use crate::{
    setup_price_tcp_server, FixMessageHandler, FixMessageSerializer, InstrumentMappingEntity,
    LogonCredentials, SettingsReader, TcpConnection,
};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const MAPPING_PK: &str = "im";

pub struct AppContext {
    pub settings: Arc<SettingsReader>,
    pub connections: Mutex<HashMap<i32, Arc<TcpConnection>>>,
    pub tcp_client: TcpClient,
}

impl AppContext {
    pub fn new(settings: Arc<SettingsReader>, _: &ServiceContext) -> AppContext {
        let tcp_client = TcpClient::new("yourbourse - fix-client".to_string(), settings.clone());

        AppContext {
            settings,
            connections: Mutex::new(HashMap::new()),
            tcp_client,
        }
    }
}

pub async fn setup_and_start(app: &Arc<AppContext>, sc: &ServiceContext) {
    service_sdk::my_logger::LOGGER.write_info(
        String::from("Main"),
        String::from("Service is starting"),
        LogEventCtx::new(),
    );

    let settings = app.settings.clone();
    let app_to_spawn = app.clone();

    let map = get_map(&settings).await;

    let settings_model = settings.get_settings().await;
    let fix_auth_creds = LogonCredentials {
        password: settings_model.your_bourse_pass.clone(),
        sender: settings_model.your_bourse_sender_company_id.clone(),
        target: settings_model.your_bourse_target_company_id.clone(),
    };

    app.tcp_client
        .start(
            Arc::new(move || -> FixMessageSerializer {
                FixMessageSerializer::new(fix_auth_creds.clone())
            }),
            Arc::new(FixMessageHandler::new(app_to_spawn, map)),
            service_sdk::my_logger::LOGGER.clone(),
        )
        .await;

    let tcp_server = setup_price_tcp_server(&app, sc.app_states.clone());
    tcp_server.start().await;

    service_sdk::my_logger::LOGGER.write_info(
        String::from("App"),
        String::from("Service is started"),
        LogEventCtx::new(),
    );
}

async fn get_map(settings: &Arc<SettingsReader>) -> HashMap<String, Vec<String>> {
    let nosql_connection: MyNoSqlTcpConnection = MyNoSqlTcpConnection::new(APP_NAME.to_string(), settings.clone());
    nosql_connection.start(service_sdk::my_logger::LOGGER.clone()).await;
    let instruments_reader: Arc<MyNoSqlDataReaderTcp<InstrumentMappingEntity>> =
        nosql_connection.get_reader().await;

    let settings_model = settings.get_settings().await;
    let mut map_entity_option = instruments_reader
        .get_entity(MAPPING_PK, settings_model.liquidity_provider_id.as_str())
        .await;

    while map_entity_option.is_none() {
        sleep(Duration::from_secs(2));
        println!("Sleeping 2 seconds");
        map_entity_option = instruments_reader
        .get_entity(MAPPING_PK, settings_model.liquidity_provider_id.as_str())
        .await;
    }


    let mut map = HashMap::<String, Vec<String>>::new();

    if map_entity_option.is_some() {
        let map_entity = map_entity_option.unwrap();
        for (our_symbol, external_symbol) in map_entity.map.iter() {
            if !map.contains_key(external_symbol.as_str()) {
                map.insert(external_symbol.to_string(), Vec::new());
            }
            map.get_mut(external_symbol)
                .unwrap()
                .push(our_symbol.to_string());
        }
    }
    map
}
