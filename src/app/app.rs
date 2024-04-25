use std::{collections::HashMap, sync::Arc};

use my_nosql_contracts::{InstrumentMappingEntity, ProductSettings, YbPriceFeedSettings};

use my_tcp_sockets::{TcpClient, TcpClientSocketSettings};
use service_sdk::{my_no_sql_sdk::reader::MyNoSqlDataReaderTcp, ServiceContext};
use tokio::sync::Mutex;

use crate::{
    settings::SettingsReader,
    tcp::{BidAskDataTcpModel, BidAskDateTimeTcpModel, BidAskTcpMessage, TcpConnection},
    your_bourse::YbMarketData,
};

const MAPPING_PK: &str = "im";

pub struct AppContext {
    pub settings: Arc<SettingsReader>,
    pub connections: Mutex<HashMap<i32, Arc<TcpConnection>>>,
    //pub tcp_client: TcpClient,
    pub product_settings: Arc<MyNoSqlDataReaderTcp<ProductSettings>>,
    pub instrument_mapping: Arc<MyNoSqlDataReaderTcp<InstrumentMappingEntity>>,

    pub tcp_client: Mutex<Option<TcpClient>>,
}

impl AppContext {
    pub async fn new(
        settings: Arc<SettingsReader>,
        service_content: &ServiceContext,
    ) -> AppContext {
        //  let tcp_client = TcpClient::new("yourbourse - fix-client".to_string(), settings.clone());

        AppContext {
            settings,
            connections: Mutex::new(HashMap::new()),
            product_settings: service_content.get_ns_reader().await,
            instrument_mapping: service_content.get_ns_reader().await,
            tcp_client: Mutex::new(None),
        }
    }

    pub async fn get_yb_settings(&self) -> Option<YbPriceFeedSettings> {
        self.product_settings.get_enum_case_model().await
    }

    pub async fn broad_cast_bid_ask(
        &self,
        market_data: YbMarketData,
        maps: &HashMap<String, Vec<String>>,
    ) {
        let map = maps.get(market_data.instrument_id.as_str());

        if map.is_none() {
            return;
        }

        let map = map.unwrap();

        if map.len() == 0 {
            return;
        }

        let connections = self.connections.lock().await;

        for instrument_id in map {
            let tcp_datetime = BidAskDateTimeTcpModel::Source(market_data.date);

            let tcp_message = BidAskDataTcpModel {
                exchange_id: "YOUR_BOURSE".to_string(),
                instrument_id: instrument_id.to_string(),
                bid: market_data.bid,
                ask: market_data.ask,
                volume: 0.0,
                datetime: tcp_datetime,
            };

            let to_send = BidAskTcpMessage::BidAsk(tcp_message);

            for connection in connections.values() {
                connection.send(&to_send).await;
            }
        }
    }

    pub async fn get_map(&self) -> HashMap<String, Vec<String>> {
        let liquidity_provider_id = self.settings.get_liquidity_provider_id().await;
        let map_entity = self
            .instrument_mapping
            .get_entity(MAPPING_PK, liquidity_provider_id.as_str())
            .await
            .unwrap();

        let mut map = HashMap::<String, Vec<String>>::new();

        for (our_symbol, external_symbol) in map_entity.map.iter() {
            if !map.contains_key(external_symbol.as_str()) {
                map.insert(external_symbol.to_string(), Vec::new());
            }
            map.get_mut(external_symbol)
                .unwrap()
                .push(our_symbol.to_string());
        }

        map
    }
}

#[async_trait::async_trait]
impl TcpClientSocketSettings for AppContext {
    async fn get_host_port(&self) -> Option<String> {
        let liquidity_provider_id = self.settings.get_liquidity_provider_id().await;
        let map_entity = self
            .instrument_mapping
            .get_entity(MAPPING_PK, liquidity_provider_id.as_str())
            .await;

        if map_entity.is_none() {
            return None;
        }

        let result: Option<YbPriceFeedSettings> = self.get_yb_settings().await;

        let result = result?;

        Some(result.url.clone())
    }
}

/*
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


 */
