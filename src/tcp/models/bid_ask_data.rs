use my_tcp_sockets::TcpWriteBuffer;
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub const SOURCE_DATE_TIME: u8 = 'S' as u8;
pub const GENERATED_DATE_TIME: u8 = 'G' as u8;
pub const OUR_DATE_TIME: u8 = 'O' as u8;
pub const MESSAGE_SPLITTER: &[u8; 1] = b" ";

#[derive(Debug, Clone)]
pub struct BidAskDataTcpModel {
    pub exchange_id: String,
    pub instrument_id: String,
    pub bid: f64,
    pub ask: f64,
    pub volume: f64,
    pub date_time: BidAskDateTimeTcpModel,
}

impl BidAskDataTcpModel {
    pub fn serialize(&self, dest: &mut impl TcpWriteBuffer) -> Result<(), SerializeError> {
        dest.write_slice(b"A");
        dest.write_slice(MESSAGE_SPLITTER);
        dest.write_slice(self.exchange_id.as_bytes());
        dest.write_slice(MESSAGE_SPLITTER);
        dest.write_slice(self.instrument_id.as_bytes());
        dest.write_slice(MESSAGE_SPLITTER);
        dest.write_slice(b"B");
        dest.write_slice(format!("{}", self.bid).as_bytes());
        dest.write_slice(MESSAGE_SPLITTER);
        dest.write_slice(b"A");
        dest.write_slice(format!("{}", self.ask).as_bytes());
        dest.write_slice(MESSAGE_SPLITTER);
        dest.write_slice(format!("{}", self.volume).as_bytes());
        dest.write_slice(MESSAGE_SPLITTER);
        self.date_time.serialize(dest)?;

        Ok(())
    }

    pub fn deserialize(src: &[u8]) -> Result<Self, SerializeError> {
        let chunks = src.split(|x| *x == b' ').collect::<Vec<&[u8]>>();
        let exchange_id = String::from_utf8(chunks[1].to_vec()).unwrap();
        let instrument_id = String::from_utf8(chunks[2].to_vec()).unwrap();
        let bid = String::from_utf8(chunks[3][1..].to_vec()).unwrap();
        let ask = String::from_utf8(chunks[4][1..].to_vec()).unwrap();
        let volume = String::from_utf8(chunks[5].to_vec()).unwrap();

        Ok(Self {
            exchange_id,
            instrument_id,
            bid: bid.parse().unwrap(),
            ask: ask.parse().unwrap(),
            volume: volume.parse().unwrap(),
            date_time: BidAskDateTimeTcpModel::deserialize(chunks[6])?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum BidAskDateTimeTcpModel {
    Source(DateTimeAsMicroseconds),
    Our(DateTimeAsMicroseconds),
    Generated(DateTimeAsMicroseconds),
}

impl BidAskDateTimeTcpModel {
    pub fn serialize(&self, dest: &mut impl TcpWriteBuffer) -> Result<(), SerializeError> {
        match self {
            &BidAskDateTimeTcpModel::Source(date) => {
                dest.write_byte(SOURCE_DATE_TIME);
                write_date(dest, date);
            }
            &BidAskDateTimeTcpModel::Our(date) => {
                dest.write_byte(OUR_DATE_TIME);
                write_date(dest, date);
            }
            &BidAskDateTimeTcpModel::Generated(date) => {
                dest.write_byte(GENERATED_DATE_TIME);
                write_date(dest, date);
            }
        };

        return Ok(());
    }

    pub fn deserialize(date_data: &[u8]) -> Result<Self, SerializeError> {
        let date_marker = date_data.first();
        let date = crate::date_utils::parse_tcp_feed_date(&date_data[1..]);

        if let Some(marker_byte) = date_marker {
            let date = match marker_byte {
                &OUR_DATE_TIME => Self::Our(date),
                &SOURCE_DATE_TIME => Self::Source(date),
                &GENERATED_DATE_TIME => Self::Generated(date),
                _ => return Err(SerializeError::InvalidDateMarker),
            };

            return Ok(date);
        }

        return Err(SerializeError::MissingDateMarker);
    }
}

/*
fn deserialize_date(date: &[u8]) -> Result<DateTime<Utc>, SerializeError> {
    let string_date = String::from_utf8(date.to_vec());

    let Ok(date_string) = string_date else {
        return Err(SerializeError::DateSerializeError);
    };

    let Ok(date_time) = NaiveDateTime::parse_from_str(&date_string, "%Y%m%d%H%M%S%.3f") else {
        return Err(SerializeError::InvalidDate);
    };

    let date_time = DateTime::<Utc>::from_utc(date_time, Utc);

    Ok(date_time)
}
 */

fn write_date(out: &mut impl TcpWriteBuffer, dt: DateTimeAsMicroseconds) {
    let str = dt.to_rfc3339();
    let str = str.as_bytes();
    out.write_slice(&str[0..4]);
    out.write_slice(&str[5..7]);
    out.write_slice(&str[8..10]);
    out.write_slice(&str[11..13]);
    out.write_slice(&str[14..16]);
    out.write_slice(&str[17..23]);
}

#[derive(Debug)]
pub enum SerializeError {
    InvalidDate,
    InvalidDateMarker,
    MissingDateMarker,
    DateSerializeError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let message = b"A BINANCE EURUSD B1.55555 A2.55555 50000000 S20230213142225.555";
        let result = BidAskDataTcpModel::deserialize(message).unwrap();

        assert_eq!(result.exchange_id, "BINANCE");
        assert_eq!(result.instrument_id, "EURUSD");
        assert_eq!(result.bid, 1.55555);
        assert_eq!(result.ask, 2.55555);
        assert_eq!(result.volume, 50000000.0);

        let is_source = match result.date_time {
            BidAskDateTimeTcpModel::Source(_) => true,
            BidAskDateTimeTcpModel::Our(_) => false,
            BidAskDateTimeTcpModel::Generated(_) => false,
        };

        assert_eq!(is_source, true);
    }

    #[test]
    fn test_serialize() {
        let message = "A BINANCE EURUSD B1.55555 A2.55555 50000000 S20230213142225.555";

        let dt = DateTimeAsMicroseconds::from_str("2023-02-13T14:22:25.555").unwrap();

        println!("{}", dt.to_rfc3339());

        let result = BidAskDataTcpModel {
            exchange_id: "BINANCE".to_string(),
            instrument_id: "EURUSD".to_string(),
            bid: 1.55555,
            ask: 2.55555,
            volume: 50000000.0,
            date_time: BidAskDateTimeTcpModel::Source(dt),
        };

        let mut serialized: Vec<u8> = Vec::new();

        result.serialize(&mut serialized).unwrap();

        assert_eq!(String::from_utf8(serialized).unwrap(), message);
    }
}
