use chrono::{DateTime, Utc};
use my_tcp_sockets::TcpContract;
use rust_fix::FixMessageReader;

#[derive(Debug, Clone)]
pub struct LogonCredentials {
    pub password: String,
    pub sender: String,
    pub target: String,
}
#[derive(Debug)]
pub struct YbMarketData {
    pub instrument_id: String,
    pub date: DateTime<Utc>,
    pub bid: f64,
    pub ask: f64,
}

#[derive(Debug)]
pub enum YbFixContract {
    Logon,
    Reject,
    Logout,
    MarketData(YbMarketData),
    MarketDataReject,
    Others,
    Ping,
    Pong,
    SubscribeToInstrument(String),
    Skip(String),
}

impl YbFixContract {
    pub fn deserialize(fix_payload: Vec<u8>) -> Self {
        let fix_message_reader = FixMessageReader::from_bytes(&fix_payload);

        match fix_message_reader.get_message_type().unwrap() {
            "A" => Self::Logon,
            "W" => {
                let model = super::model_deserializer::deserialize_market_data(&fix_message_reader);
                match model {
                    Ok(model) => Self::MarketData(model),
                    Err(err) => Self::Skip(err),
                }
            }
            "V" => Self::Skip("Got V Message".to_string()),

            "Y" => Self::MarketDataReject,
            "3" => Self::Reject,
            "5" => Self::Logout,
            _ => Self::Others,
        }
    }
}

impl TcpContract for YbFixContract {
    fn is_pong(&self) -> bool {
        match self {
            Self::Pong => true,
            _ => false,
        }
    }
}

/*
pub enum FixPayload<'s> {
    Logon(FixMessageReader<'s>),
    Reject(FixMessageReader<'s>),
    Logout(FixMessageReader<'s>),
    MarketData(FixMessageReader<'s>),
    MarketDataReject(FixMessageReader<'s>),
    Others(FixMessageReader<'s>),
}
 */
