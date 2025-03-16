use my_nosql_contracts::YbPriceFeedSettings;
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
    pub feed_settings: Option<YbPriceFeedSettingsModel>,
}

impl SettingsReader {
    pub async fn get_liquidity_provider_id(&self) -> String {
        let read = self.settings.read().await;
        read.liquidity_provider_id.to_string()
    }

    pub async fn get_yb_price_feed(&self) -> Option<YbPriceFeedSettings> {
        let read = self.settings.read().await;
        let settings = read.feed_settings.as_ref()?;

        let result = YbPriceFeedSettings {
            time_stamp: Default::default(),
            url: settings.host_port.clone(),
            pass: settings.user_password.to_string(),
            sender_company_id: settings.sender_company_id.to_string(),
            target_company_id: settings.target_company_id.to_string(),
        };

        Some(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YbPriceFeedSettingsModel {
    pub host_port: String,
    pub sender_company_id: String,
    pub target_company_id: String,
    pub user_password: String,
}
