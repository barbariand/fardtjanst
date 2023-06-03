#![feature(let_chains)]
use actix::{Actor, Context, Arbiter};
use actix_session::config::{PersistentSession, SessionLifecycle};
use actix_session::storage::CookieSessionStore;
use actix_session::{Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_files::Files;
use actix_web::cookie::time::Duration;
use actix_web::{
    get,post, middleware::Logger, web, App, HttpResponse, HttpServer, Responder, Result,
};
use web_push;
mod backgroundtask;
use backgroundtask::BackgroundActor;
pub mod db;
use db::{
    sea_orm::{self, sea_query::IntoCondition, EntityTrait, QuerySelect, RelationTrait},
    tempsessions, users,
};
use futures::StreamExt;
use actix_web::{error, ResponseError};
use std::fs::File;
use log::error;
use futures::executor::block_on;
use macros;

pub const MOCK_SERVER_URL: &str = "http://127.0.0.1:5376";

#[get("/trips")]
#[macros::restricted_route]
async fn trips(data: web::Data<AppData>, session: Session) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().finish())
}
fn to_response_error(err: web_push::WebPushError)->actix_web::error::Error{
    let str=match err{
        web_push::WebPushError::BadRequest(s) => {
            "BadRequest".to_owned()+s.unwrap_or("No request".to_string()).as_str()

        },
        web_push::WebPushError::ServerError(s) => {
            "BadRequest".to_owned()+s.unwrap_or(std::time::Duration::new(0, 0)).as_millis().to_string().as_str()+" miliseconds"
        },
        web_push::WebPushError::Other(s) => {
            "Other ".to_string()+s.as_str()
        },
        s=>{
            s.to_string()
        }
    };
    error::ErrorInternalServerError(str)
}
#[post("/registerNotifier")]
async fn registerNotifier(data: web::Data<AppData>,mut payload: web::Payload) -> Result<impl Responder>{
    println!("HELLO FROM NOTIFICATIONS");
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > MAX_PAYLOAD_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    };
    let subscription_info = match serde_json::from_slice::<web_push::SubscriptionInfo>(&body) {
        Ok(au) => au,
        Err(e) => {
            error!(target:"/api/autherization",
                "faild to serilze {:?} into SubscriptionInfo, got error {}",
                body,
                e
            );
            return Err(error::ErrorBadRequest("Invalid json"));
        }
    };
    Ok("hello")
}

pub const MAX_PAYLOAD_SIZE: usize = 262_144;
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
            data_base: data_base.clone(),
        }
    }
    
}
struct SumActor {}

impl Actor for SumActor { 
    type Context = Context<Self>;
}
#[actix::main]
async fn main() -> std::io::Result<()> {
    let secret_key = Key::generate();
    //gets database
    let data_base = block_on(db::getdb()).expect("Could not create or find database:");
    std::env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();

    let app_data = AppData::new(data_base.clone());
    let server=HttpServer::new(move || {
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
            .service(trips)
            .service(Files::new("/static", "./static").prefer_utf8(true))
            .service(registerNotifier)
    })
    .bind("127.0.0.1:5377")?
    .run();
    let bg=BackgroundActor::new(data_base);
    Arbiter::new().spawn_fn(move|| {
        bg.start();
    });
    let serverarbiter=Arbiter::new();
    serverarbiter.spawn_fn(move || {
        block_on( async{
            server.await.unwrap()
        })
    });
    serverarbiter.join().expect("failed to run");
    Ok(())
}
