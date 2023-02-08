use actix_rt;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use db;
use env_logger::Env;
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    db::add(1, 2);
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
