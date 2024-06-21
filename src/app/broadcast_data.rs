use std::{collections::HashMap, sync::Arc};

use prices_tcp_contracts::{BidAskDataTcpModel, BidAskDateTimeTcpModel, BidAskTcpMessage};

use crate::{your_bourse::YbMarketData, BidAskTcpSocketConnection};

pub struct BroadCastData {
    pub maps: HashMap<String, Vec<String>>,
    pub connections: HashMap<i32, Arc<BidAskTcpSocketConnection>>,
    pub lp_id: String,
}

impl BroadCastData {
    pub fn new(lp_id: String) -> Self {
        Self {
            maps: HashMap::new(),
            connections: HashMap::new(),
            lp_id,
        }
    }

    pub async fn broad_cast_bid_ask(&self, market_data: &YbMarketData) -> Option<Vec<String>> {
        let map = self.maps.get(market_data.instrument_id.as_str());

        if map.is_none() {
            return None;
        }

        let map = map.unwrap();

        if map.len() == 0 {
            return None;
        }

        for instrument_id in map {
            let tcp_date_time = BidAskDateTimeTcpModel::Source(market_data.date);

            let tcp_message = BidAskDataTcpModel {
                exchange_id: self.lp_id.clone(),
                instrument_id: instrument_id.to_string(),
                bid: market_data.bid,
                ask: market_data.ask,
                volume: 0.0,
                date_time: tcp_date_time,
            };

            let to_send = BidAskTcpMessage::BidAsk(tcp_message);

            for connection in self.connections.values() {
                connection.send(&to_send).await;
            }
        }

        Some(map.to_vec())
    }
}
