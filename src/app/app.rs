use std::{collections::HashMap, sync::Arc};

use my_nosql_contracts::{InstrumentMappingEntity, ProductSettings, YbPriceFeedSettings};

use my_tcp_sockets::{TcpClient, TcpClientSocketSettings};
use service_sdk::{my_no_sql_sdk::reader::MyNoSqlDataReaderTcp, ServiceContext};
use tokio::sync::Mutex;

use crate::{settings::SettingsReader, your_bourse::YbMarketData};

use super::BroadCastData;

pub struct AppContext {
    pub settings: Arc<SettingsReader>,
    pub broadcast_data: Mutex<BroadCastData>,
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

        let lp_id = settings.get_liquidity_provider_id().await;
        AppContext {
            settings,
            broadcast_data: Mutex::new(BroadCastData::new(lp_id)),
            product_settings: service_content.get_ns_reader().await,
            instrument_mapping: service_content.get_ns_reader().await,
            tcp_client: Mutex::new(None),
        }
    }

    pub async fn get_yb_settings(&self) -> Option<YbPriceFeedSettings> {
        self.product_settings.get_enum_case_model().await
    }

    pub async fn broad_cast_bid_ask(&self, market_data: YbMarketData) {
        let broadcast_data = self.broadcast_data.lock().await;
        broadcast_data.broad_cast_bid_ask(market_data).await;
    }

    pub async fn get_map(&self) -> HashMap<String, Vec<String>> {
        let liquidity_provider_id = self.settings.get_liquidity_provider_id().await;
        let map_entity = self
            .instrument_mapping
            .get_entity(
                InstrumentMappingEntity::PARTITION_KEY,
                liquidity_provider_id.as_str(),
            )
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

        let mut lock_map = self.broadcast_data.lock().await;
        lock_map.maps = map.clone();

        map
    }
}

#[async_trait::async_trait]
impl TcpClientSocketSettings for AppContext {
    async fn get_host_port(&self) -> Option<String> {
        let liquidity_provider_id = self.settings.get_liquidity_provider_id().await;

        let map_entity = self
            .instrument_mapping
            .get_entity(
                InstrumentMappingEntity::PARTITION_KEY,
                liquidity_provider_id.as_str(),
            )
            .await;

        if map_entity.is_none() {
            println!("There is no Map configuration. Skipping connection to Fix YourBourse.");
            return None;
        }

        let result: Option<YbPriceFeedSettings> = self.get_yb_settings().await;

        if result.is_none() {
            println!(
                "There is no Yb Fix connection product configuration. Skipping connection to Fix YourBourse."
            );
            return None;
        }

        let result = result.unwrap();
        println!("There is configuration. Url: {}", result.url);

        Some(result.url.clone())
    }
}
