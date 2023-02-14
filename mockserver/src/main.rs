use actix_rt;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use db::{cake, fruit, getdb, sea_orm::*, Cake};
use env_logger::Env;
use futures::executor::block_on;
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let db = match block_on(getdb()) {
        Ok(db) => db,
        Err(err) => panic!("Could not create database: {}", err),
    };

    let cheese = cake::ActiveModel {
        name: ActiveValue::Set("Happy Bakery".to_owned()),
        id: ActiveValue::Set(1),
    };
    let dada = Cake::insert(cheese).exec(&db).await.expect("msg");
    
    env_logger::init_from_env(
        Env::default()
            .default_filter_or("info")
            .default_filter_or("debug"),
    );
    HttpServer::new(move || App::new().wrap(Logger::default()).wrap(Logger::new(" t")))
        .bind("127.0.0.1:5376")?
        .run()
        .await
}
