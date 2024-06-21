mod app;
mod date_utils;
mod settings;
mod tcp;
mod your_bourse;
use std::{sync::Arc, time::Duration};
mod timers;

use my_tcp_sockets::{tcp_connection::TcpSocketConnection, TcpClient};

use prices_tcp_contracts::{BidAskTcpMessage, BidAskTcpSerializer};
use timers::UploadSrcPricesTimer;
use your_bourse::{
    FixMessageHandler, FixMessageSerializer, YbFixContract, YbSerializerFactory, YbTcpSate,
};

use crate::app::AppContext;

type FixSocketConnection = TcpSocketConnection<YbFixContract, FixMessageSerializer, YbTcpSate>;
type BidAskTcpSocketConnection = TcpSocketConnection<BidAskTcpMessage, BidAskTcpSerializer, ()>;

#[tokio::main]
async fn main() {
    let settings_reader = crate::settings::SettingsReader::new(".my-cfd-platform").await;
    let settings_reader = Arc::new(settings_reader);

    let mut service_context = service_sdk::ServiceContext::new(settings_reader.clone()).await;

    let app_context = Arc::new(AppContext::new(settings_reader, &service_context).await);

    service_context.register_timer(Duration::from_secs(1), |timer| {
        timer.register_timer(
            "PriceSrc Uploader",
            Arc::new(UploadSrcPricesTimer::new(app_context.clone())),
        );
    });

    let tcp_server =
        crate::tcp::setup_price_tcp_server(&app_context, service_context.app_states.clone());

    tcp_server.start().await;

    let tcp_client = TcpClient::new("Yb-fix-client".to_string(), app_context.clone());

    tcp_client
        .start(
            Arc::new(YbSerializerFactory::new(app_context.clone())),
            Arc::new(FixMessageHandler::new(app_context.clone()).await),
            service_sdk::my_logger::LOGGER.clone(),
        )
        .await;

    service_context.start_application().await;
}
