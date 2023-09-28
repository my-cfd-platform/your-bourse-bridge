use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};
use service_sdk::my_no_sql_sdk::abstractions::MyNoSqlEntity;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstrumentMappingEntity {
    #[serde(rename = "PartitionKey")]
    pub partition_key: String,
    #[serde(rename = "RowKey")]
    pub row_key: String,
    #[serde(rename = "TimeStamp")]
    pub time_stamp: String,
    #[serde(rename = "LpId")]
    pub liquidity_provider_id: String,
    #[serde(rename = "Map")]
    pub map: HashMap<String, String>,
}

impl MyNoSqlEntity for InstrumentMappingEntity {
    const TABLE_NAME: &'static str = "instrument-mapping";

    fn get_partition_key(&self) -> &str {
        &self.partition_key
    }

    fn get_row_key(&self) -> &str {
        &self.row_key
    }

    fn get_time_stamp(&self) -> i64 {
        DateTimeAsMicroseconds::parse_iso_string(self.time_stamp.as_str())
            .unwrap()
            .unix_microseconds
    }
}
