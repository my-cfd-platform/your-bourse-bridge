use std::sync::Arc;

use my_tcp_sockets::{ConnectionEvent, SocketEventCallback};
use service_sdk::my_logger::LogEventCtx;

use crate::{tcp::models::BidAskTcpMessage, AppContext, BidAskTcpSerializer};

pub struct PriceTcpServerCallback {
    pub app: Arc<AppContext>,
}

impl PriceTcpServerCallback {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<BidAskTcpMessage, BidAskTcpSerializer> for PriceTcpServerCallback {
    async fn handle(
        &self,
        connection_event: ConnectionEvent<BidAskTcpMessage, BidAskTcpSerializer>,
    ) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                let mut write_access = self.app.connections.lock().await;
                service_sdk::my_logger::LOGGER.write_info(
                    String::from("PriceTcpServerCallback"),
                    format!("New connection {}", connection.id),
                    LogEventCtx::new(),
                );
                
                write_access.insert(connection.id, connection);
            }
            ConnectionEvent::Disconnected(connection) => {
                let mut write_access = self.app.connections.lock().await;
                write_access.remove(&connection.id);
                service_sdk::my_logger::LOGGER.write_info(
                    String::from("PriceTcpServerCallback"),
                    format!("Disconnected {}", connection.id),
                    LogEventCtx::new(),
                );
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => {
                if payload.is_ping() {
                    connection.send(BidAskTcpMessage::Pong).await;
                }
                //println!("Received payload from {:?}", payload);
            }
        }
    }
}
