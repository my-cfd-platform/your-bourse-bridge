use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, Utc};
use my_tcp_sockets::{tcp_connection::SocketConnection, ConnectionEvent, SocketEventCallback};

use crate::{
    AppContext, BidAskDataTcpModel, BidAskDateTimeTcpModel, BidAskTcpMessage, FixMessageType,
    FixPayload,
};

use super::{FixMessage, FixMessageSerializer};

pub struct FixMessageHandler {
    app: Arc<AppContext>,
}

impl FixMessageHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl FixMessageHandler {
    async fn send_instrument_subscribe(
        &self,
        connection: &Arc<SocketConnection<FixMessage, FixMessageSerializer>>,
    ) {
        let instruments_to_subsribe = self
            .app
            .settings
            .instruments_mapping
            .iter()
            .map(|x| x.1.clone())
            .collect::<Vec<String>>();
        for instrument in instruments_to_subsribe {
            let subscribe_message = FixMessage {
                mesage_type: FixMessageType::SubscribeToInstrument(instrument),
            };
            connection.send(subscribe_message).await;
        }
    }
    async fn send_logon(
        &self,
        connection: &Arc<SocketConnection<FixMessage, FixMessageSerializer>>,
    ) {
        let subscribe_message = FixMessage {
            mesage_type: FixMessageType::Logon,
        };
        connection.send(subscribe_message).await;
    }

    async fn handle_price_tick_message(&self, fix_message: FixPayload) {
        if let FixPayload::MarketData(message) = fix_message {
            // there shall be always no_md_entries in the message
            // skip message if it's not exist
            let no_md_entries = message.get_value_string("268");
            if no_md_entries == None {
                println!("Broken message? {}", message.to_string());
                return;
            }
            let no_md_entries = no_md_entries.unwrap().parse::<u32>().unwrap(); //.collect::<u32>().unwrap();

            // not sure why buy sometimes there are no prices available in the message,
            // so we skip the message
            if no_md_entries < 2 {
                println!("Broken message? {}", message.to_string());
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

            let id = message.get_value_string("55").unwrap();
            let datetime = message.get_value_string("52").unwrap();

            let nd = NaiveDateTime::parse_from_str(&datetime, "%Y%m%d-%H:%M:%S%.3f").unwrap();
            let date_time = DateTime::<Utc>::from_utc(nd, Utc);
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
        }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<FixMessage, FixMessageSerializer> for FixMessageHandler {
    async fn handle(&self, connection_event: ConnectionEvent<FixMessage, FixMessageSerializer>) {
        match connection_event {
            ConnectionEvent::Connected(connection) => self.send_logon(&connection).await,
            ConnectionEvent::Disconnected(_) => println!("Disconnected from FIX-Feed"),
            ConnectionEvent::Payload {
                connection,
                payload,
            } => match payload.mesage_type {
                FixMessageType::Payload(data) => {
                    match data {
                        crate::FixPayload::Logon(_) => {
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
                        crate::FixPayload::Others(_message) => {
                            //println!("Found other message: {}", message.to_string());
                        }
                    };
                }
                _ => {}
            },
        }
    }
}
