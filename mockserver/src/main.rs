#![feature(test)]
extern crate test;
use actix_rt;
use actix_session::{
    config::{PersistentSession, SessionLifecycle},
    storage::CookieSessionStore,
    SessionMiddleware,
};
use actix_web::cookie::Key;
use actix_web::{middleware::Logger, web, App, HttpServer};
mod routes;
use mockserverdb as db;
use db::sea_orm;
use futures::executor::block_on;
const MAX_PAYLOAD_SIZE: usize = 262_144;
#[derive(Clone)]
struct AppData {
    data_base: sea_orm::DatabaseConnection,
}

impl AppData {
    fn get_db(&self) -> &db::DatabaseConnection {
        &self.data_base
    }
    fn new(data_base: db::DatabaseConnection) -> AppData {
        AppData {
            data_base: data_base,
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let secret_key = Key::generate();
    //gets database
    let data_base = block_on(db::getdb()).expect("Could not create or find database:");
    std::env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();
    let app_data = AppData::new(data_base);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .wrap(Logger::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .session_lifecycle(SessionLifecycle::PersistentSession(
                        PersistentSession::default(),
                    ))
                    .build(),
            )
            .service(routes::api::trips)
            .service(routes::api::autherization)
            .service(routes::api::order)
    })
    .bind("127.0.0.1:5376")?
    .run()
    .await
}
