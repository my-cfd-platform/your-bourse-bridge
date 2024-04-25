use std::sync::Arc;

use my_tcp_sockets::TcpSerializerFactory;

use crate::app::AppContext;

use super::{FixMessageSerializer, YbFixContract, YbTcpSate};

pub struct YbSerializerFactory {
    app: Arc<AppContext>,
}

impl YbSerializerFactory {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl TcpSerializerFactory<YbFixContract, FixMessageSerializer, YbTcpSate> for YbSerializerFactory {
    async fn create_serializer(&self) -> FixMessageSerializer {
        FixMessageSerializer::new()
    }
    async fn create_serializer_state(&self) -> YbTcpSate {
        let settings = self.app.get_yb_settings().await;
        YbTcpSate::new(settings)
    }
}
