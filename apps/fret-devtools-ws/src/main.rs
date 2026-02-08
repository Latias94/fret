use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use fret_diag_ws::server::{DevtoolsWsServer, DevtoolsWsServerConfig};

fn main() -> Result<(), String> {
    let port = env_u16("FRET_DEVTOOLS_WS_PORT").unwrap_or(7331);
    let token =
        std::env::var("FRET_DEVTOOLS_TOKEN").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

    let bind = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    eprintln!("fret-devtools-ws: bind={bind} token={token}");
    eprintln!("fret-devtools-ws: url=ws://127.0.0.1:{port}/?fret_devtools_token={token}");

    DevtoolsWsServer::new(DevtoolsWsServerConfig { bind, token }).run()
}

fn env_u16(key: &str) -> Option<u16> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}
