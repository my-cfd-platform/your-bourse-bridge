use my_nosql_contracts::YbPriceFeedSettings;
use my_tcp_sockets::TcpSerializerState;

use super::YbFixContract;

pub struct YbTcpSate {
    pub settings: Option<YbPriceFeedSettings>,
}

impl YbTcpSate {
    pub fn new(settings: Option<YbPriceFeedSettings>) -> Self {
        Self { settings }
    }

    pub fn get_settings(&self) -> &YbPriceFeedSettings {
        self.settings.as_ref().unwrap()
    }
}

impl TcpSerializerState<YbFixContract> for YbTcpSate {
    fn is_tcp_contract_related_to_metadata(&self, _: &YbFixContract) -> bool {
        false
    }
    fn apply_tcp_contract(&mut self, _: &YbFixContract) {}
}
