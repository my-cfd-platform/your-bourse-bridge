use std::{collections::HashMap, sync::Arc};

use crate::BidAskTcpSocketConnection;

pub struct BroadCastData {
    pub maps: HashMap<String, Vec<String>>,
    pub connections: HashMap<i32, Arc<BidAskTcpSocketConnection>>,
}

impl BroadCastData {
    pub fn new() -> Self {
        Self {
            maps: HashMap::new(),
            connections: HashMap::new(),
        }
    }
}
