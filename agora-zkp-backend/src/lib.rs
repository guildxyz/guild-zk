pub mod balancy;
pub mod config;
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
    let balancy_client = web::Data::new(balancy::BalancyClient::new(
        conf.url_balancy.clone(),
        conf.apikey_balancy.clone(),
        conf.url_pubkey.clone(),
        conf.apikey_pubkey.clone(),
        180,
    ));
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/sign", web::post().to(get_xyz_holders_pubkeys))
            .app_data(conf.clone())
            .app_data(balancy_client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

async fn get_xyz_holders_pubkeys(
    balancy_client: web::Data<balancy::BalancyClient>,
    req_body: web::Json<balancy::ReqXyzHolders>,
) -> impl Responder {
    let req = req_body.into_inner();
    let resp = get_xyz_holders_pubkeys_inner(balancy_client, req).await;
    match resp {
        Ok(pubkeys) => HttpResponse::Ok().json(pubkeys),
        Err(err) => HttpResponse::InternalServerError().body(format!("{:?}", err)),
    }
}

async fn get_xyz_holders_pubkeys_inner(
    balancy_client: web::Data<balancy::BalancyClient>,
    req: balancy::ReqXyzHolders,
) -> Result<Vec<String>, Error> {
    let addresses = balancy_client.get_xyz_holders_addresses(req).await?;
    let pubkeys = balancy_client.get_pubkeys(addresses).await?;
    Ok(pubkeys)
}
