#![feature(let_chains)]
#![feature(lint_reasons)]
use core::panic;

use actix::{Actor, Arbiter, SyncArbiter};
use actix_files::Files;
use actix_rt::System;
use actix_session::config::{PersistentSession, SessionLifecycle};
use actix_session::storage::CookieSessionStore;
use actix_session::{Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::error::ErrorInternalServerError;
use actix_web::{
    get, middleware::Logger, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
    Result,
};
mod backgroundtask;
mod routes;
use backgroundtask::BackgroundActor;
pub mod db;
use actix_web::error;
use db::notification_info::ActiveModel;
use db::{
    sea_orm::{
        self, sea_query::IntoCondition, ActiveModelTrait, EntityTrait, QuerySelect, RelationTrait,
    },
    tempsessions, users,
};
use futures::executor::block_on;
use futures::StreamExt;
use log::{error, info};
pub const MOCK_SERVER_URL: &str = "http://127.0.0.1:5376";


#[macros::restricted_route]
#[post("/registerNotifier")]
async fn register_notifier(
    data: web::Data<AppData>,
    mut payload: web::Payload,
    session: Session,
) -> Result<impl Responder> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > MAX_PAYLOAD_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
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
    let notification_info: ActiveModel =
        db::LinkedSubscriptionInfo::new(user, subscription_info).into();
    let res = notification_info.insert(&data.data_base).await;
    if res.is_err() {
        return Err(ErrorInternalServerError("Database error"));
    };
    Ok(HttpResponse::Ok().finish())
}
pub const MAX_PAYLOAD_SIZE: usize = 262_144;
#[derive(Clone)]
struct AppData {
    data_base: sea_orm::DatabaseConnection,
    addr:actix::Addr<BackgroundActor>,
}

impl AppData {
    fn get_db(&self) -> &db::DatabaseConnection {
        &self.data_base
    }
    fn new(data_base: db::DatabaseConnection, addr:actix::Addr<BackgroundActor>) -> AppData {
        AppData { data_base ,addr}
    }
}
#[actix::main]
async fn main() -> std::io::Result<()> {
    let secret_key = Key::generate();
    //gets database
    let data_base = block_on(db::getdb()).expect("failed to create connection to db");
    std::env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();
    let bg = BackgroundActor::new(data_base.clone());
    let addr: actix::Addr<BackgroundActor>=bg.start();
    
    let app_data = AppData::new(data_base.clone(),addr);
    let server = HttpServer::new(move || {
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
            .service(routes::api::trips::trips)
            .service(Files::new("/static", "./static").prefer_utf8(true))
            .service(register_notifier)
            .service(routes::api::login::login)
            .service(routes::api::logout::logout)
    })
    .bind("127.0.0.1:5377")?
    .run();
    let serverarbiter = Arbiter::new();
    serverarbiter.spawn_fn(move || block_on(async { server.await.unwrap() }));
    // Use tokio::select to handle whichever future completes first.
    tokio::signal::ctrl_c().await.expect("could not wait for signal");
    println!("shutting down");
    System::current().stop();
    Ok(())
}
