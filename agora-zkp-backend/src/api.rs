use super::app;
use super::balancy;
use super::config;

use actix_web::dev::Server;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use std::net::TcpListener;

pub fn run(listener: TcpListener, conf: config::Settings) -> Result<Server, std::io::Error> {
    let application = web::Data::new(app::Application::new(conf));
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/sign", web::post().to(get_xyz_holders_pubkeys))
            .app_data(application.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

async fn get_xyz_holders_pubkeys(
    app: web::Data<app::Application>,
    req_body: web::Json<balancy::ReqXyzHolders>,
) -> impl Responder {
    let req = req_body.into_inner();
    let resp = app.get_signed_xyz_holders_pubkeys(req).await;
    match resp {
        Ok(pubkeys) => HttpResponse::Ok().json(pubkeys),
        Err(err) => HttpResponse::InternalServerError().body(format!("{:?}", err)),
    }
}
