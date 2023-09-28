use my_tcp_sockets::TcpClientSocketSettings;
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
    pub your_bourse_url: String,
    pub your_bourse_pass: String,
    pub your_bourse_sender_company_id: String,
    pub your_bourse_target_company_id: String,
    pub my_no_sql_tcp_reader: String,
    pub liquidity_provider_id: String,
    pub my_telemetry: String,
}

#[async_trait]
impl TcpClientSocketSettings for SettingsReader {
    async fn get_host_port(&self) -> String{
        let read_access = self.settings.read().await;
        read_access.your_bourse_url.clone()
    }
}