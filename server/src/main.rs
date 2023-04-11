use std::sync::Mutex;
use actix_web::{
    get, middleware::Logger, web, web::Data, App, HttpResponse, HttpServer, Responder,Result
};
use env_logger::Env;
use log::info;
use my_web_app::MyTestStruct;
use actix_rt;
use actix_session::{
    config::{PersistentSession, SessionLifecycle},
    storage::CookieSessionStore,
    SessionMiddleware,Session
};
use db::{
    sea_orm::{self,ColumnTrait, EntityTrait, QueryFilter,QuerySelect,RelationTrait,sea_query::IntoCondition},
    tempsessions, users, TempSession, Users,
};
use actix_web::cookie::Key;
use serverdb as db;
use macros;
use futures::executor::block_on;
#[get("/hello")]
async fn hello() -> impl Responder {
    info!("Sending a String.");
    "Hello Cindy"
}

#[get("/trips")]
#[macros::restricted_route]
pub(crate) async fn trips(
    data: web::Data<AppData>,
    session: Session,
) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().finish())
}

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        Env::default()
            .default_filter_or("debug"),
    );
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
            .service(hello)
            .service(jsondata)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
