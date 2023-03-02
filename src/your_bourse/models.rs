use my_tcp_sockets::tcp_connection::TcpContract;
use rust_fix::FixMessageBuilder;

#[derive(Debug, Clone)]
pub struct LogonCread{
    pub password: String,
    pub sender: String,
    pub target: String,
}

pub struct FixMessage {
    pub mesage_type: FixMessageType,
    // pub auth_data: LogonCread,
}

pub enum FixMessageType{
    SubscribeToInstrument(String),
    Logon,
    Payload(FixPayload),
    Pong,
    Ping,
}

impl TcpContract for FixMessage {
    fn is_pong(&self) -> bool {
        match self.mesage_type {
            FixMessageType::Pong => true,
            _ => false,
        }
    }
}

pub enum FixPayload {
    Logon(FixMessageBuilder),
    Reject(FixMessageBuilder),
    Logout(FixMessageBuilder),
    MarketData(FixMessageBuilder),
    Others(FixMessageBuilder),
}
