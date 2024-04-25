use my_tcp_sockets::TcpSerializerFactory;

use super::{BidAskTcpMessage, BidAskTcpSerializer};

pub struct BidAskTcpSerializerFactor;

#[async_trait::async_trait]
impl TcpSerializerFactory<BidAskTcpMessage, BidAskTcpSerializer, ()> for BidAskTcpSerializerFactor {
    async fn create_serializer(&self) -> BidAskTcpSerializer {
        BidAskTcpSerializer::new()
    }
    async fn create_serializer_state(&self) -> () {
        ()
    }
}
