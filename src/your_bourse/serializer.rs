use std::{sync::{atomic::AtomicU64}, vec, env};

use chrono::Utc;
use my_tcp_sockets::{
    socket_reader::{ReadBuffer, ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer,
};

use rust_fix::{FixMessageBuilder, FIX_DELIMETR, FIX_EQUALS};

use crate::{FixMessage, FixMessageType, FixPayload, LogonCread};

pub struct FixMessageSerializer {
    message_counter: AtomicU64,
    auth_cread: LogonCread,
}

impl FixMessageSerializer {
    pub fn new(auth_cread: LogonCread) -> Self {
        Self {
            message_counter: AtomicU64::new(1),
            auth_cread: auth_cread.to_owned(),
        }
    }

    pub fn serialize_logon(
        &self,
        password: &str,
        sender_comp_id: &str,
        target_comp_id: &str,
    ) -> FixMessageBuilder {
        let date = Utc::now();
        let date_string = date.format("%Y%m%d-%H:%M:%S.%3f").to_string();
        let count = self
            .message_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "A");
        fix_builder.with_value(49, sender_comp_id);
        fix_builder.with_value(56, target_comp_id);
        fix_builder.with_value(52, date_string.as_str());
        fix_builder.with_value(34, count.to_string().as_str());
        fix_builder.with_value(108, "30");
        fix_builder.with_value(141, "Y");
        fix_builder.with_value(554, password);
        fix_builder.with_value(98, "0");

        return fix_builder;
    }

    pub fn serialize_instrument_subscribe(
        &self,
        instrument: &String,
        sender_comp_id: &str,
        target_comp_id: &str,
    ) -> FixMessageBuilder {
        let date = Utc::now();
        let date_string = date.format("%Y%m%d-%H:%M:%S.%3f").to_string();
        let count = self
            .message_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "V");
        let uuid = chrono::Utc::now().timestamp_nanos().to_string();

        fix_builder.with_value(49, sender_comp_id);
        fix_builder.with_value(52, date_string.as_str());
        fix_builder.with_value(56, target_comp_id);
        fix_builder.with_value(34, count.to_string().as_str());
        //MDReqID - can be just a symbol name
        fix_builder.with_value(262, &uuid.to_string());
        //SubscriptionRequestType 1 = Snapshot + Updates
        fix_builder.with_value(263, "1");
        //Market Depth 1 = Top of Book
        fix_builder.with_value(264, "1");
        //MDUpdateType 
        fix_builder.with_value(265, "0");
        //NoMDEntryTypes
        fix_builder.with_value(267, "2");
        //Bid
        fix_builder.with_value(269, "0");
        //Ask
        fix_builder.with_value(269, "1");
        //NoRelatedSym
        fix_builder.with_value(146, "1");
        //Symbol
        fix_builder.with_value(55, instrument);

        return fix_builder;
    }

    pub fn serialize_ping(&self, sender_comp_id: &str, target_comp_id: &str) -> FixMessageBuilder {
        let date = Utc::now();
        let date_string = date.format("%Y%m%d-%H:%M:%S.%3f").to_string();

        let count = self
            .message_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "0");
        fix_builder.with_value(49, sender_comp_id);
        fix_builder.with_value(52, date_string.as_str());
        fix_builder.with_value(56, target_comp_id);
        fix_builder.with_value(34, count.to_string().as_str());

        return fix_builder;
    }
}

#[async_trait::async_trait]
impl TcpSocketSerializer<FixMessage> for FixMessageSerializer {
    const PING_PACKET_IS_SINGLETONE: bool = false;

    fn serialize(&self, contract: FixMessage) -> Vec<u8> {
        let FixMessage {
            mesage_type,
            // auth_data,
        } = contract;
        let LogonCread {
            password,
            sender,
            target,
        } = &self.auth_cread;

        let to_send = match mesage_type {
            FixMessageType::SubscribeToInstrument(instrument) => {
                self.serialize_instrument_subscribe(&instrument, &sender, &target)
            }
            FixMessageType::Logon => self.serialize_logon(&password, &sender, &target),
            FixMessageType::Payload(_) => panic!("cant serialize payload, only for income"),
            FixMessageType::Pong => panic!("cant serialize ping, only for income"),
            FixMessageType::Ping => self.serialize_ping(&sender, &target),
        };

        return to_send.as_bytes().to_vec();
    }

    fn get_ping(&self) -> FixMessage {
        return FixMessage {
            mesage_type: FixMessageType::Ping,
            // auth_data: self.auth_cread.clone(),
        };
    }

    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
    ) -> Result<FixMessage, ReadingTcpContractFail> {
        let mut result = vec![];
        let fix_delimetr = vec![FIX_DELIMETR];
        let mut buff = ReadBuffer::new(1024 * 24);
        let mut parse_error = false;
        loop {
            let chunk = socket_reader
                .read_until_end_marker(&mut buff, &fix_delimetr.as_slice())
                .await;
            match chunk {
                Ok(res) => {
                    let equals_index = res.iter().position(|x| x == &FIX_EQUALS);
                    //sometimes panics here
                    if equals_index == None{
                        parse_error = true;
                        result.extend_from_slice(res);
                        println!("Failed on: {}", String::from_utf8(chunk.unwrap().to_vec()).unwrap());
                        continue;
                    }
                    let equals_index = equals_index.unwrap();
                    let key = String::from_utf8(res[0..equals_index].to_vec()).unwrap();
                    result.extend_from_slice(res);
                    if key == "10".to_string() {
                        break;
                    }
                }
                Err(err) => {
                    println!("Err: {:?}", err);
                    break;
                }
            };
        }
        if parse_error {
            println!("Err here: {}",String::from_utf8(result.to_vec()).unwrap());
            return Err(ReadingTcpContractFail::ErrorReadingSize);
        }
        if result.len() == 0 {
            return Err(ReadingTcpContractFail::ErrorReadingSize);
        }
        match env::var("FIX_DEBUG") {
            Ok(_) => {
                println!("message to serialise: {}", String::from_utf8(result.to_vec()).unwrap());
            }
            Err(_)=>{}
        }

        let fix = FixMessageBuilder::from_bytes(&result, false).unwrap();
        let message_type = fix.get_message_type_as_string();
        
        match env::var("FIX_DEBUG") {
            Ok(_) => {
                println!("debug: {}", fix.clone().to_string());
            }
            Err(_)=>{}
        }

        let payload_type = match message_type.as_str() {
            "A" => FixPayload::Logon(fix),
            "W" => FixPayload::MarketData(fix),
            "3" => FixPayload::Reject(fix),
            "5" => FixPayload::Logout(fix),
            _ => FixPayload::Others(fix),
        };
        return Ok(FixMessage {
            mesage_type: FixMessageType::Payload(payload_type),
            // auth_data: self.auth_cread.clone(),
        });
    }
}
