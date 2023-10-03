use std::{collections::HashMap, env, sync::Arc};

use chrono::{DateTime, NaiveDateTime, Utc};
use my_tcp_sockets::{tcp_connection::SocketConnection, ConnectionEvent, SocketEventCallback};
use service_sdk::my_logger::LogEventCtx;

use crate::{
    AppContext, BidAskDataTcpModel, BidAskDateTimeTcpModel, BidAskTcpMessage, FixMessageType,
    FixPayload,
};

use super::{FixMessage, FixMessageSerializer};

pub struct FixMessageHandler {
    app: Arc<AppContext>,
    map: HashMap<String, Vec<String>>,
}

impl FixMessageHandler {
    pub fn new(app: Arc<AppContext>, map: HashMap<String, Vec<String>>) -> Self {
        Self { app, map }
    }
}

impl FixMessageHandler {
    async fn send_instrument_subscribe(
        &self,
        connection: &Arc<SocketConnection<FixMessage, FixMessageSerializer>>,
    ) {
        let maps = &self.map;
        println!("Map: {:#?}", maps);
        let mut info_message = "Subscribing to ".to_owned();
        for (external_instrument, _) in maps {
            info_message.push_str(format!("{} ", external_instrument).as_str());
            let subscribe_message = FixMessage {
                message_type: FixMessageType::SubscribeToInstrument(
                    external_instrument.to_string(),
                ),
            };
            connection.send(subscribe_message).await;
        }

        service_sdk::my_logger::LOGGER.write_info(
            String::from("FixMessageHandler"),
            info_message,
            LogEventCtx::new(),
        );
    }
    async fn send_logon(
        &self,
        connection: &Arc<SocketConnection<FixMessage, FixMessageSerializer>>,
    ) {
        let subscribe_message = FixMessage {
            message_type: FixMessageType::Logon,
        };

        connection.send(subscribe_message).await;
    }

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
    async fn send_to_tcp(&self, market: String, time: DateTime<Utc>, bid: f64, ask: f64) {
        let tcp_datetime = BidAskDateTimeTcpModel::Source(time);
        //let bid = bid.as_str().parse::<f64>().unwrap();
        //let ask = ask.as_str().parse::<f64>().unwrap();
        let tcp_message = BidAskDataTcpModel {
            exchange_id: "YOUR_BOURSE".to_string(),
            instrument_id: market,
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
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<FixMessage, FixMessageSerializer> for FixMessageHandler {
    async fn handle(&self, connection_event: ConnectionEvent<FixMessage, FixMessageSerializer>) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                println!("Connected log");
                self.send_logon(&connection).await;
            },
            ConnectionEvent::Disconnected(_) => println!("Disconnected from FIX-Feed"),
            ConnectionEvent::Payload {
                connection,
                payload,
            } => match payload.message_type {
                FixMessageType::Payload(data) => {
                    match data {
                        crate::FixPayload::Logon(_) => {
                            println!("Got logon");
                            self.send_instrument_subscribe(&connection).await
                        }
                        crate::FixPayload::Reject(_) => {
                            println!("Rejected by FIX-Feed");
                        }
                        crate::FixPayload::Logout(_) => {
                            println!("Logged out from FIX-Feed");
                        }
                        crate::FixPayload::MarketData(_) => {
                            self.handle_price_tick_message(data).await
                        }
                        crate::FixPayload::MarketDataReject(message) => {
                            service_sdk::my_logger::LOGGER.write_error(
                                String::from("FixMessageHandler"),
                                format!("Market Data Rejected: {}", message.to_string()),
                                LogEventCtx::new(),
                            );
                        }
                        crate::FixPayload::Others(_message) => match env::var("FIX_DEBUG") {
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
            },
        }
    }
}
