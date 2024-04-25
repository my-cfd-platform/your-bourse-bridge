use std::{collections::HashMap, sync::Arc};

use crate::tcp::TcpConnection;

pub struct BroadCastData {
    pub maps: HashMap<String, Vec<String>>,
    pub connections: HashMap<i32, Arc<TcpConnection>>,
}

impl BroadCastData {
    pub fn new() -> Self {
        Self {
            maps: HashMap::new(),
            connections: HashMap::new(),
        }
    }
}
