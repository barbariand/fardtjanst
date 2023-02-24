use actix_rt;
use actix_session::config::{PersistentSession, SessionLifecycle};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Result};
use db::{getdb, resor, sea_orm::*, users, Resor, Users};
use env_logger::Env;
use futures::executor::block_on;
struct AppData {
    db: DatabaseConnection,
}
impl Clone for AppData {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
        }
    }
}
impl AppData {
    fn get_db(&self) -> &db::DatabaseConnection {
        &self.db
    }
    fn new(db: db::DatabaseConnection) -> AppData {
        AppData { db }
    }
    fn to_string(&self) -> String {
        "Hello".to_string()
    }
}
#[get("/api/test")]
async fn test(data: web::Data<AppData>, session: Session) {}
#[get("/api/autherization")]
async fn autherization(data: web::Data<AppData>, session: Session) -> Result<String> {
    session
        .insert("test", "testvalue")
        .expect("failed json serilisation");
    Ok(data.to_string())
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let secret_key = Key::generate();
    //gets database and
    let db = block_on(getdb()).expect("Could not create or find database:");
    env_logger::init_from_env(
        Env::default()
            .default_filter_or("info")
            .default_filter_or("debug"),
    );
    let app_data = AppData::new(db);
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
            .service(autherization)
    })
    .bind("127.0.0.1:5376")?
    .run()
    .await
}
