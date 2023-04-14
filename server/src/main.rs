use actix_web::{rt,
    get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder,Result
};
use reqwest;
use actix_session::{
    config::{PersistentSession, SessionLifecycle},
    storage::CookieSessionStore,
    SessionMiddleware,Session
};
use std::{time::Duration, sync::Arc};
use db::{self,
    sea_orm::{self, EntityTrait,QuerySelect,RelationTrait,sea_query::IntoCondition},
    tempsessions, users, 
};
use actix_web::cookie::Key;
use macros;
use futures::executor::block_on;
const MOCK_SERVER_URL:&str="http://127.0.0.1:5376";
#[get("/start")]
async fn hello() -> impl Responder {
    
    "start"
}
#[get("/stop")]
async fn stop()-> impl Responder{
    "hII"
}
#[get("/trips")]
#[macros::restricted_route]
async fn trips(
    data: web::Data<AppData>,
    session: Session,
) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().finish())
}
struct BackgroundProsesor{
    data_base: sea_orm::DatabaseConnection,
}
impl BackgroundProsesor{
    async fn run(){
        loop{
            
        }
    }
}
const MAX_PAYLOAD_SIZE: usize = 262_144;
#[derive(Clone)]
struct AppData {
    data_base: sea_orm::DatabaseConnection,
    backgroundprossesor:Arc<BackgroundProsesor>
}


impl AppData {
    fn get_db(&self) -> &db::DatabaseConnection {
        &self.data_base
    }
    fn new(data_base: db::DatabaseConnection) -> AppData {
        AppData {
            data_base:data_base.clone(),
            backgroundprossesor:Arc::new(BackgroundProsesor{
                data_base,
            }),
        }
    }
}

#[actix_web::main]
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
            .service(hello)
            .service(trips)
    })
    .bind("127.0.0.1:5377")?
    .run()
    .await
}
