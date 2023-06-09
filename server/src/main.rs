#![feature(let_chains)]
#![feature(drain_filter)]
use actix::{Actor, Context, Arbiter};
use actix_session::config::{PersistentSession, SessionLifecycle};
use actix_session::storage::CookieSessionStore;
use actix_session::{Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_files::Files;
use actix_web::error::ErrorInternalServerError;
use actix_web::{
    get,post, middleware::Logger, web, App, HttpResponse, HttpServer, Responder, Result,HttpRequest
};
use web_push;
mod backgroundtask;
use backgroundtask::BackgroundActor;
pub mod db;
use db::{
    sea_orm::{self, sea_query::IntoCondition, EntityTrait, QuerySelect, RelationTrait,ActiveModelTrait},
    tempsessions, users,
};
use futures::StreamExt;
use actix_web::{error};
use log::{error, info};
use futures::executor::block_on;
use macros;

pub const MOCK_SERVER_URL: &str = "http://127.0.0.1:5376";

#[get("/trips")]
#[macros::restricted_route]
async fn trips(data: web::Data<AppData>, session: Session) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().finish())
}


#[macros::restricted_route]
#[post("/registerNotifier")]
async fn register_notifier(data: web::Data<AppData>,mut payload: web::Payload, session: Session) -> Result<impl Responder>{
    
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
    let notification_info=db::notification_info::ActiveModel{
        UserId:sea_orm::ActiveValue::Set(user.id),
        endpoint:sea_orm::ActiveValue::Set(subscription_info.endpoint),
        auth:sea_orm::ActiveValue::Set(subscription_info.keys.auth),
        p256dh:sea_orm::ActiveValue::Set(subscription_info.keys.p256dh),
        ..Default::default()
    };
    let res=notification_info.insert(&data.data_base).await;
    if let Err(_)= res{
        return Err(ErrorInternalServerError("Database error"));
    };
    Ok(HttpResponse::Ok().finish())
}

#[get("/checkheaders")]
async fn checkheaders(req: HttpRequest)->Result<impl Responder>{
    let cookies=match req.cookie("id") {
        Some(cookie) => format!("Cookie: {}", cookie.value()),
        None => "No cookie found".to_string(),
    };
    info!("{}",cookies);
    Ok("")
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
            .service(checkheaders)
            .service(trips)
            .service(Files::new("/static", "./static").prefer_utf8(true))
            .service(register_notifier)
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
