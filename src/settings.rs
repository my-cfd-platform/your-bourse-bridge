use serde::{Deserialize, Serialize};
service_sdk::macros::use_settings!();

#[derive(
    my_settings_reader::SettingsModel,
    AutoGenerateSettingsTraits,
    SdkSettingsTraits,
    Serialize,
    Deserialize,
    Debug,
    Clone,
)]
pub struct SettingsModel {
    pub seq_conn_string: String,
    pub my_no_sql_tcp_reader: String,
    pub liquidity_provider_id: String,
    pub my_telemetry: String,
    pub my_no_sql_writer: String,
}

impl SettingsReader {
    pub async fn get_liquidity_provider_id(&self) -> String {
        let read = self.settings.read().await;
        read.liquidity_provider_id.to_string()
    }
}
