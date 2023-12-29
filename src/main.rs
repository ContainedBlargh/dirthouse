mod modules;
mod config;

use actix_web::*;
use std::io::Read;
use serde::Deserialize;
use crate::modules::compile_modules;

#[get("/echo")]
async fn echo(req: HttpRequest) -> impl Responder {
    let conn_info = req.connection_info().clone();
    let host = conn_info.host().to_owned();
    host
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config::load();
    let addrs = (config.host_addr.clone(), config.port);
    let modules = modules::find_modules(&config);
    compile_modules(&config, &modules);
    HttpServer::new(move || {
        let mut app = App::new();
        app = app.service(echo);
        for module in &modules {

        }
        return app;
    })
        .bind(addrs)?
        .run()
        .await
}
