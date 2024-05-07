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
    }
}

impl TcpSerializerState<YbFixContract> for () {
    fn is_tcp_contract_related_to_metadata(&self, _: &YbFixContract) -> bool {
        false
    }

    fn apply_tcp_contract(&mut self, _: &YbFixContract) {}
}
