pub mod balancy;
pub mod config;
pub mod eth;
pub mod moralis;
pub mod signer;

use actix_web::dev::Server;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use anyhow::Error;
use std::net::TcpListener;

pub fn run(listener: TcpListener, conf: config::Settings) -> Result<Server, std::io::Error> {
    let conf = web::Data::new(conf);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/tx", web::get().to(get_txs))
            .route("/sign", web::post().to(get_xyz_holders_pubkeys))
            .app_data(conf.clone())
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

async fn get_txs(
    conf: web::Data<config::Settings>,
    query: web::Query<GetTxsQuery>,
) -> impl Responder {
    println!("api key: {}, tx: {}", conf.apikey, query.tx);
    let res =
        moralis::get_txhash_by_sender_addr(conf.apikey.to_string(), query.tx.to_string()).await;
    match res {
        Ok(txhash) => HttpResponse::Ok().body(txhash),
        Err(_error) => HttpResponse::NotFound().body(format!("{:?}", _error)),
    }
}

async fn get_xyz_holders_pubkeys(
    conf: web::Data<config::Settings>,
    req_body: web::Json<balancy::ReqXyzHolders>,
) -> impl Responder {
    let req = req_body.into_inner();
    let resp = get_xyz_holders_pubkeys_inner(conf, req).await;
    match resp {
        Ok(pubkeys) => HttpResponse::Ok().json(pubkeys),
        Err(err) => HttpResponse::InternalServerError().body(format!("{:?}", err)),
    }
}

async fn get_xyz_holders_pubkeys_inner(
    conf: web::Data<config::Settings>,
    req: balancy::ReqXyzHolders,
) -> Result<Vec<String>, Error> {
    let addresses = balancy::get_xyz_holders_addresses(
        conf.url_balancy.clone(),
        conf.apikey_balancy.clone(),
        req,
    )
    .await?;
    let pubkeys = balancy::get_pubkeys(
        conf.url_pubkey.as_str(),
        conf.apikey_pubkey.as_str(),
        addresses,
    )
    .await?;
    Ok(pubkeys)
}
