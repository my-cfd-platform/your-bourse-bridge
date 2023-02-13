use std::collections::HashMap;

use async_trait::async_trait;
use my_tcp_sockets::TcpClientSocketSettings;
use serde::{Deserialize, Serialize};

#[derive(my_settings_reader::SettingsModel, Serialize, Deserialize, Debug, Clone)]
pub struct SettingsModel {
    #[serde(rename = "SeqConnString")]
    pub seq_conn_string: String,
    #[serde(rename = "YourBourseUrl")]
    pub your_bourse_url: String,
    #[serde(rename = "YourBoursePass")]
    pub your_bourse_pass: String,
    #[serde(rename = "YourBourseSenderCompanyId")]
    pub your_bourse_sender_company_id: String,
    #[serde(rename = "YourBourseTargetCompanyId")]
    pub your_bourse_target_company_id: String,
    #[serde(rename = "InstrumentsMapping")]
    pub instruments_mapping: HashMap<String, String>,
}

#[async_trait]
impl TcpClientSocketSettings for SettingsModel {
    async fn get_host_port(&self) -> String {
        return self.your_bourse_url.clone();
    }
}