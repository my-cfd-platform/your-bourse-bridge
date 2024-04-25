use std::sync::Arc;

use my_tcp_sockets::{
    tcp_connection::TcpSocketConnection, SocketEventCallback, TcpSerializerState,
};
use service_sdk::my_logger::LogEventCtx;

use crate::{tcp::models::BidAskTcpMessage, AppContext, BidAskTcpSocketConnection};

use super::BidAskTcpSerializer;

pub struct PriceTcpServerCallback {
    pub app: Arc<AppContext>,
}

impl PriceTcpServerCallback {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<BidAskTcpMessage, BidAskTcpSerializer, ()> for PriceTcpServerCallback {
    async fn connected(&self, connection: Arc<BidAskTcpSocketConnection>) {
        let mut write_access = self.app.connections.lock().await;
        service_sdk::my_logger::LOGGER.write_info(
            String::from("PriceTcpServerCallback"),
            format!("New connection {}", connection.id),
            LogEventCtx::new(),
        );

        write_access.insert(connection.id, connection);
    }

    async fn disconnected(
        &self,
        connection: Arc<TcpSocketConnection<BidAskTcpMessage, BidAskTcpSerializer, ()>>,
    ) {
        let mut write_access = self.app.connections.lock().await;
        write_access.remove(&connection.id);
        service_sdk::my_logger::LOGGER.write_info(
            String::from("PriceTcpServerCallback"),
            format!("Disconnected {}", connection.id),
            LogEventCtx::new(),
        );
    }

    async fn payload(
        &self,
        connection: &Arc<TcpSocketConnection<BidAskTcpMessage, BidAskTcpSerializer, ()>>,
        payload: BidAskTcpMessage,
    ) {
        if payload.is_ping() {
            connection.send(&BidAskTcpMessage::Pong).await;
        }
    }
}

impl TcpSerializerState<BidAskTcpMessage> for () {
    fn is_tcp_contract_related_to_metadata(&self, _: &BidAskTcpMessage) -> bool {
        false
    }

    fn apply_tcp_contract(&mut self, _: &BidAskTcpMessage) {}
}
