use std::sync::Arc;

use my_tcp_sockets::{ConnectionEvent, SocketEventCallback};

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
                println!("New connection {}", connection.id);
                write_access.insert(connection.id, connection);
            }
            ConnectionEvent::Disconnected(connection) => {
                let mut write_access = self.app.connections.lock().await;
                write_access.remove(&connection.id);
                println!("Disconnected {}", connection.id);
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => {
                if payload.is_ping() {
                    connection.send(BidAskTcpMessage::Pong).await;
                }
                println!("Received payload from {:?}", payload);
            }
        }
    }
}
