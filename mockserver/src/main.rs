use actix_rt;
use actix_session::*;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use db::{getdb, resor, sea_orm::*, users, Resor, Users};
use env_logger::Env;

use futures::executor::block_on;
struct App_data {
    db: DatabaseConnection,
}
impl Clone for App_data {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
        }
    }
}
impl App_data {
    fn get_db(&self) -> &db::DatabaseConnection {
        &self.db
    }
    fn new(db: db::DatabaseConnection) -> App_data {
        App_data { db }
    }
    fn to_String(&self) -> String {
        "Hello".to_string()
    }
}
#[get("/autherization")]
async fn autherization(data: web::Data<App_data>) -> Result<String> {
    Ok(data.to_String())
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    //gets database and
    let db = block_on(getdb()).expect("Could not create or find database:");
    env_logger::init_from_env(
        Env::default()
            .default_filter_or("info")
            .default_filter_or("debug"),
    );
    let app_data = App_data::new(db);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .wrap(Logger::default())
            .service(autherization)
    })
    .bind("127.0.0.1:5376")?
    .run()
    .await
}
