use std::{env, net::SocketAddr, sync::Arc};

use my_tcp_sockets::TcpServer;
use prices_tcp_contracts::TcpFeedSerializerFactory;
use rust_extensions::AppStates;
use service_sdk::my_logger::LogEventCtx;

use crate::{tcp::tcp_event_handler::PriceTcpServerCallback, AppContext};

pub struct PriceRouterTcpServer {
    pub tcp_server: TcpServer,
    pub app: Arc<AppContext>,
    pub app_states: Arc<AppStates>,
}

impl PriceRouterTcpServer {
    pub async fn start(&self) {
        self.tcp_server
            .start(
                Arc::new(TcpFeedSerializerFactory),
                Arc::new(PriceTcpServerCallback::new(self.app.clone())),
                self.app_states.clone(),
                service_sdk::my_logger::LOGGER.clone(),
            )
            .await;

        println!("TCP server started");
    }
}

pub fn setup_price_tcp_server(
    app: &Arc<AppContext>,
    app_states: Arc<AppStates>,
) -> PriceRouterTcpServer {
    let mut port = 8085;
    match env::var("CUSTOM_PORT") {
        Ok(val) => {
            port = val.parse().unwrap();
        }
        Err(_) => {}
    }
    let tcp_server: TcpServer = TcpServer::new(
        "YourBoursePriceBridge".to_string(),
        SocketAddr::from(([0, 0, 0, 0], port)),
    );

    service_sdk::my_logger::LOGGER.write_info(
        String::from("PriceRouterTcpServer"),
        format!("Listening on port: {}", port),
        LogEventCtx::new(),
    );

    return PriceRouterTcpServer {
        tcp_server,
        app: app.clone(),
        app_states,
    };
}
