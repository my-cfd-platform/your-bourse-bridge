use std::sync::Arc;

use my_tcp_sockets::{
    tcp_connection::TcpSocketConnection, SocketEventCallback, TcpSerializerState,
};
use service_sdk::my_logger::LogEventCtx;

use crate::{AppContext, FixSocketConnection};

use super::{FixMessageSerializer, YbFixContract, YbTcpSate};

pub struct FixMessageHandler {
    app: Arc<AppContext>,
}

impl FixMessageHandler {
    pub async fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl FixMessageHandler {
    async fn send_instrument_subscribe(&self, connection: &Arc<FixSocketConnection>) {
        let maps = self.app.get_map().await;
        println!("Map: {:#?}", maps);
        let mut info_message = "Subscribing to ".to_owned();
        for external_instrument in maps.keys() {
            info_message.push_str(format!("{} ", external_instrument).as_str());
            let subscribe_message =
                YbFixContract::SubscribeToInstrument(external_instrument.to_string());
            connection.send(&subscribe_message).await;
        }

        service_sdk::my_logger::LOGGER.write_info(
            String::from("FixMessageHandler"),
            info_message,
            LogEventCtx::new(),
        );
    }

    /*
    async fn handle_price_tick_message(&self, fix_message: FixPayload) {
        if let FixPayload::MarketData(message) = fix_message {
            // there shall be always no_md_entries in the message
            // skip message if it's not exist
            let no_md_entries = message.get_value_string("268");
            if no_md_entries == None {
                service_sdk::my_logger::LOGGER.write_error(
                    String::from("FixMessageHandler"),
                    format!("Broken Message: {}", message.to_string()),
                    LogEventCtx::new(),
                );
                return;
            }
            let no_md_entries = no_md_entries.unwrap().parse::<u32>().unwrap(); //.collect::<u32>().unwrap();

            // not sure why buy sometimes there are no prices available in the message,
            // so we skip the message
            if no_md_entries < 2 {
                service_sdk::my_logger::LOGGER.write_error(
                    String::from("FixMessageHandler"),
                    format!("Broken Message: {}", message.to_string()),
                    LogEventCtx::new(),
                );
                return;
            }
            let prices = message
                .get_values_string("270")
                .iter()
                .map(|x| x.parse::<f64>().unwrap())
                .collect::<Vec<f64>>();

            // I think the clients have to know that we do like this,
            // this may be a regulatory issue for them if they not aware
            //let (bid, ask) = match prices[1] > prices[0] {
            //    true => (prices[0], prices[1]),
            //    false => (prices[1], prices[0]),
            //};

            let (bid, ask) = (prices[0], prices[1]);

            let external_market = message.get_value_string("55").unwrap();
            let datetime = message.get_value_string("52").unwrap();
            let nd = NaiveDateTime::parse_from_str(&datetime, "%Y%m%d-%H:%M:%S%.3f").unwrap();
            let date_time = DateTime::<Utc>::from_utc(nd, Utc);

            if let Some(mapped_market) = self.map.get(&external_market) {
                let markets = mapped_market.to_owned();
                for market in markets {
                    self.send_to_tcp(market, date_time, bid, ask).await;
                }
            }

            /*

            let tcp_datetime = BidAskDateTimeTcpModel::Source(date_time);

            let tcp_message = BidAskDataTcpModel {
                exchange_id: "YOUR_BOURSE".to_string(),
                instrument_id: id,
                bid,
                ask,
                volume: 0.0,
                datetime: tcp_datetime,
            };
            for connection in self.app.connections.lock().await.values() {
                connection
                    .send(BidAskTcpMessage::BidAsk(tcp_message.clone()))
                    .await;
            }
            */
        }
    }
     */
}

#[async_trait::async_trait]
impl SocketEventCallback<YbFixContract, FixMessageSerializer, YbTcpSate> for FixMessageHandler {
    async fn connected(
        &self,
        connection: Arc<TcpSocketConnection<YbFixContract, FixMessageSerializer, YbTcpSate>>,
    ) {
        println!("Connected log");
        connection.send(&YbFixContract::Logon).await;
    }

    async fn disconnected(
        &self,
        _connection: Arc<TcpSocketConnection<YbFixContract, FixMessageSerializer, YbTcpSate>>,
    ) {
        println!("Disconnected from FIX-Feed");
    }

    async fn payload(
        &self,
        connection: &Arc<TcpSocketConnection<YbFixContract, FixMessageSerializer, YbTcpSate>>,
        contract: YbFixContract,
    ) {
        match contract {
            YbFixContract::Logon => {
                self.send_instrument_subscribe(&connection).await;
            }
            YbFixContract::Reject => {}
            YbFixContract::Logout => {}
            YbFixContract::MarketData(market_data) => {
                self.app.broad_cast_bid_ask(market_data).await;
            }
            YbFixContract::MarketDataReject => {}
            YbFixContract::Others => {}
            YbFixContract::Ping => {}
            YbFixContract::Pong => {}
            YbFixContract::SubscribeToInstrument(_) => {}
            YbFixContract::Skip(reason) => {
                println!("Fixing Fix message: {}", reason);
            }
        }

        /*
        match contract.message_type {
            FixMessageType::Payload(data) => {
                match data {
                    FixPayload::Logon(_) => {
                        println!("Got logon");
                        self.send_instrument_subscribe(&connection).await
                    }
                    FixPayload::Reject(_) => {
                        println!("Rejected by FIX-Feed");
                    }
                    FixPayload::Logout(_) => {
                        println!("Logged out from FIX-Feed");
                    }
                    FixPayload::MarketData(_) => self.handle_price_tick_message(data).await,
                    FixPayload::MarketDataReject(message) => {
                        service_sdk::my_logger::LOGGER.write_error(
                            String::from("FixMessageHandler"),
                            format!("Market Data Rejected: {}", message.to_string()),
                            LogEventCtx::new(),
                        );
                    }
                    FixPayload::Others(_message) => match env::var("FIX_DEBUG") {
                        Ok(_) => {
                            service_sdk::my_logger::LOGGER.write_info(
                                String::from("FixMessageHandler"),
                                format!("Other Message: {}", _message.to_string()),
                                LogEventCtx::new(),
                            );
                        }
                        Err(_) => {}
                    },
                };
            }
            _ => {}
        }

         */
    }
}

impl TcpSerializerState<YbFixContract> for () {
    fn is_tcp_contract_related_to_metadata(&self, _: &YbFixContract) -> bool {
        false
    }

    fn apply_tcp_contract(&mut self, _: &YbFixContract) {}
}
