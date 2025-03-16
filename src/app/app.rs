use std::{collections::HashMap, sync::Arc};

use my_nosql_contracts::{
    price_src::BidAskPriceSrc, InstrumentMappingEntity, ProductSettings, YbPriceFeedSettings,
};

use my_tcp_sockets::TcpClientSocketSettings;
use service_sdk::{
    my_no_sql_sdk::{
        data_writer::{CreateTableParams, MyNoSqlDataWriter},
        reader::MyNoSqlDataReaderTcp,
    },
    ServiceContext,
};
use tokio::sync::Mutex;

use crate::{settings::SettingsReader, your_bourse::YbMarketData};

use super::{BroadCastData, PriceCache};

pub struct AppContext {
    pub broadcast_data: Mutex<BroadCastData>,
    pub bid_ask_price_src: MyNoSqlDataWriter<BidAskPriceSrc>,
    //pub tcp_client: TcpClient,
    pub product_settings: Arc<MyNoSqlDataReaderTcp<ProductSettings>>,
    pub instrument_mapping: Arc<MyNoSqlDataReaderTcp<InstrumentMappingEntity>>,
    pub prices_cache: PriceCache,
    settings_reader: Arc<SettingsReader>,
    pub lp_id: String,
}

impl AppContext {
    pub async fn new(
        settings_reader: Arc<SettingsReader>,
        service_content: &ServiceContext,
    ) -> AppContext {
        let lp_id = settings_reader.get_liquidity_provider_id().await;

        let bid_ask_price_src = MyNoSqlDataWriter::new(
            settings_reader.clone(),
            Some(CreateTableParams {
                persist: false,
                max_partitions_amount: None,
                max_rows_per_partition_amount: None,
            }),
            service_sdk::my_no_sql_sdk::abstractions::DataSynchronizationPeriod::Sec5,
        );
        //  let tcp_client = TcpClient::new("yourbourse - fix-client".to_string(), settings.clone());

        AppContext {
            lp_id: lp_id.clone(),
            broadcast_data: Mutex::new(BroadCastData::new(lp_id)),
            product_settings: service_content.get_ns_reader().await,
            instrument_mapping: service_content.get_ns_reader().await,
            prices_cache: PriceCache::new(),
            bid_ask_price_src,
            settings_reader,
        }
    }

    pub async fn get_yb_settings(&self) -> Option<YbPriceFeedSettings> {
        if let Some(settings) = self.settings_reader.get_yb_price_feed().await {
            return Some(settings);
        }

        self.product_settings.get_enum_case_model().await
    }

    pub async fn broad_cast_bid_ask(&self, market_data: YbMarketData) {
        let broadcast_data = self.broadcast_data.lock().await;
        if let Some(instruments) = broadcast_data.broad_cast_bid_ask(&market_data).await {
            let mut to_upload = Vec::with_capacity(instruments.len());
            for instrument_id in instruments {
                let price_src = BidAskPriceSrc {
                    partition_key: self.lp_id.clone(),
                    row_key: instrument_id.clone(),
                    src_id: market_data.instrument_id.clone(),
                    time_stamp: "".to_string(),
                    bid: market_data.bid,
                    ask: market_data.ask,
                    dt: market_data.date.to_rfc3339(),
                };

                to_upload.push(price_src);
            }

            self.prices_cache.update(to_upload.into_iter()).await;
        }
    }

    pub async fn get_map(&self) -> HashMap<String, Vec<String>> {
        let map_entity = self
            .instrument_mapping
            .get_entity(InstrumentMappingEntity::PARTITION_KEY, self.lp_id.as_str())
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
        let map_entity = self
            .instrument_mapping
            .get_entity(InstrumentMappingEntity::PARTITION_KEY, self.lp_id.as_str())
            .await;

        if map_entity.is_none() {
            println!("There is no Map configuration. Skipping connection to Fix YourBourse.");
            return None;
        }

        let result = self.get_yb_settings().await;

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
