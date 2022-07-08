use agora_zkp_backend::config::get_config;
use agora_zkp_backend::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_config();

    let address = format!("127.0.0.1:{}", config.app_port);
    let listener = TcpListener::bind(address)?;
    run(listener, config.apikey)?.await
}
