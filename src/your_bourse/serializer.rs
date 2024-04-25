use std::sync::atomic::AtomicU64;

use my_tcp_sockets::{
    socket_reader::{ReadBuffer, ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer, TcpWriteBuffer,
};

use rust_fix::{utils::FIX_DELIMITER, FixMessageItem};

use super::yb_tcp_state::YbTcpSate;

use super::YbFixContract;

const FIX_DELIMITER_AS_ARR: [u8; 1] = [FIX_DELIMITER];
pub struct FixMessageSerializer {
    message_counter: AtomicU64,
    buffer: ReadBuffer,
}

impl FixMessageSerializer {
    pub fn new() -> Self {
        Self {
            message_counter: AtomicU64::new(1),
            buffer: ReadBuffer::new(2048 * 24),
        }
    }

    fn get_next_message_id(&self) -> u64 {
        self.message_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    async fn receive_fix_payload(
        &mut self,
        socket_reader: &mut impl SocketReader,
    ) -> Result<Vec<u8>, ReadingTcpContractFail> {
        let mut result = vec![];
        loop {
            let chunk = socket_reader
                .read_until_end_marker(&mut self.buffer, &FIX_DELIMITER_AS_ARR.as_slice())
                .await;
            match chunk {
                Ok(res) => {
                    result.extend_from_slice(res);
                    let item = FixMessageItem::from_slice(res);
                    if item.key == "10".to_string() {
                        break;
                    }
                }
                Err(err) => {
                    println!("Err: {:?}", err);
                    break;
                }
            };
        }

        if result.len() == 0 {
            return Err(ReadingTcpContractFail::ErrorReadingSize);
        }

        Ok(result)
    }
}

#[async_trait::async_trait]
impl TcpSocketSerializer<YbFixContract, YbTcpSate> for FixMessageSerializer {
    fn serialize(
        &self,
        out: &mut impl TcpWriteBuffer,
        contract: &YbFixContract,
        state: &YbTcpSate,
    ) {
        let fix_message_writer = match contract {
            YbFixContract::Ping => super::models_serializers::serialize_ping(
                state.get_settings(),
                self.get_next_message_id(),
            ),

            YbFixContract::Logon => super::models_serializers::serialize_logon(
                state.get_settings(),
                self.get_next_message_id(),
            ),
            YbFixContract::SubscribeToInstrument(instrument) => {
                super::models_serializers::serialize_instrument_subscribe(
                    state.get_settings(),
                    self.get_next_message_id(),
                    instrument,
                )
            }
            _ => {
                panic!("Fix message {:?} can not be serialized", contract)
            }
        };

        out.write_slice(fix_message_writer.compile_message().as_slice());
    }

    fn get_ping(&self) -> YbFixContract {
        YbFixContract::Ping
    }

    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
        _state: &YbTcpSate,
    ) -> Result<YbFixContract, ReadingTcpContractFail> {
        let fix_payload = self.receive_fix_payload(socket_reader).await?;

        return Ok(YbFixContract::deserialize(fix_payload));
    }
}
