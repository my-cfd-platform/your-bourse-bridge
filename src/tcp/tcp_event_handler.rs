use std::sync::Arc;

use my_tcp_sockets::{tcp_connection::TcpSocketConnection, SocketEventCallback};
use prices_tcp_contracts::*;
use service_sdk::my_logger::LogEventCtx;

use crate::{app::AppContext, BidAskTcpSocketConnection};

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
        service_sdk::my_logger::LOGGER.write_info(
            String::from("PriceTcpServerCallback"),
            format!(
                "New connection {}. Addr: {:?}",
                connection.id, connection.addr
            ),
            LogEventCtx::new(),
        );
        let mut write_access = self.app.broadcast_data.lock().await;
        write_access.connections.insert(connection.id, connection);
    }

    async fn disconnected(
        &self,
        connection: Arc<TcpSocketConnection<BidAskTcpMessage, BidAskTcpSerializer, ()>>,
    ) {
        service_sdk::my_logger::LOGGER.write_info(
            String::from("PriceTcpServerCallback"),
            format!(
                "Disconnected {}. Addr: {:?}",
                connection.id, connection.addr
            ),
            LogEventCtx::new(),
        );
        let mut write_access = self.app.broadcast_data.lock().await;
        write_access.connections.remove(&connection.id);
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
