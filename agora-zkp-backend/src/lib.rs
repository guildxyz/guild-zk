pub mod balancy;
pub mod config;
pub mod eth;
pub mod moralis;

use actix_web::dev::Server;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use std::net::TcpListener;

pub fn run(listener: TcpListener, apikey: String) -> Result<Server, std::io::Error> {
    let apikey = web::Data::new(apikey);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/tx", web::get().to(get_txs))
            .app_data(apikey.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(serde::Deserialize)]
struct GetTxsQuery {
    tx: String,
}

async fn get_txs(apikey: web::Data<String>, query: web::Query<GetTxsQuery>) -> impl Responder {
    println!("api key: {}, tx: {}", apikey.get_ref(), query.tx);
    let res = moralis::get_txhash_by_sender_addr(apikey.to_string(), query.tx.to_string()).await;
    match res {
        Ok(txhash) => HttpResponse::Ok().body(txhash),
        Err(_error) => HttpResponse::NotFound().body(format!("{:?}", _error)),
    }
}
