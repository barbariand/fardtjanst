use actix_rt;
use actix_service::Service;
use actix_session::{
    config::{PersistentSession, SessionLifecycle},
    storage::CookieSessionStore,
    Session, SessionMiddleware,
};
use actix_web::{
    cookie::Key,
    dev::{ServiceRequest, ServiceResponse},
};
use actix_web::{
    error, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder, Result,
};
use bcrypt;
use colored::Colorize;
use db as DB;
use db::sea_orm::sea_query::IntoCondition;
use db::sea_orm::RelationTrait;
use futures::FutureExt;
use futures::{executor::block_on, StreamExt};
use log::{debug, error};
use macros;
use serde_json;
use DB::{
    resor,
    sea_orm::{self, sea_query::Expr, ColumnTrait, EntityTrait, QueryFilter, QuerySelect},
    tempsessions, users, Resor, TempSession, Users,
};
const MAX_PAYLOAD_SIZE: usize = 262_144;
use colored::*;
use futures::{future::ok, Future};
use pretty_env_logger::env_logger::Env;
use serde::{Deserialize, Serialize};
use slog;
use slog::{info, o, Drain};
use slog_async;
use slog_term;
use std::fmt::Write;
#[derive(Serialize)]
struct IsAuth {
    isauth: bool,
}
#[derive(Clone)]
struct AppData {
    db: sea_orm::DatabaseConnection,
}
/* impl Drop for AppData {
    fn drop(&mut self) {
        block_on(async {
            TempSession::delete_many().exec(&self.db).await.unwrap();
        });
        debug!("deleted sessions");
    }
} */
#[derive(Serialize, Deserialize, Debug, Default)]
struct AuthSession {
    username: i32,
    password: String,
}

impl AppData {
    fn get_db(&self) -> &db::DatabaseConnection {
        &self.db
    }
    fn new(db: DB::DatabaseConnection) -> AppData {
        AppData { db }
    }
}

#[post("/api/autherization")]
async fn autherization(
    data: web::Data<AppData>,
    session: Session,
    mut payload: web::Payload,
) -> Result<impl Responder> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > MAX_PAYLOAD_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    debug!("{:?}", body);
    let auth = match serde_json::from_slice::<AuthSession>(&body) {
        Ok(au) => au,
        Err(e) => {
            error!(target:"/api/autherization",
                "faild to serilze {:?} into {:?}, got error {}",
                body,
                AuthSession {
                    ..Default::default()
                },
                e
            );
            return Err(error::ErrorBadRequest("Invalid json"));
        }
    };
    debug!(target: "/api/autherization","AUTH: {:?}", auth);
    let userres = Users::find()
        .filter(users::Column::CardNummer.eq(auth.username))
        .one(data.get_db())
        .await;
    debug!("{:?}", userres);
    let httpresponse: HttpResponse = match userres {
        Ok(user) => match user {
            Some(u) => {
                //TODO encrypt password

                match bcrypt::verify(auth.password, u.password.as_str()) {
                    Ok(sucsess) => match sucsess {
                        true => (),
                        false => {
                            return Err(error::ErrorUnauthorized(
                                "{\"Error\":\"Username or password is wrong\"}",
                            ))
                        }
                    },
                    Err(e) => {
                        error!("failed to check hash: {}", e);
                        return Err(error::ErrorUnauthorized(
                            "{\"Error\":\"Username or password is wrong\"}",
                        ));
                    }
                };
                let newtempsession = tempsessions::ActiveModel {
                    user_id: db::sea_orm::ActiveValue::Set(Some(u.id)),
                    ..Default::default()
                };
                TempSession::insert(newtempsession);
                session
                    .insert("id", u.id)
                    .expect("failed json serilisation");
                HttpResponse::Ok().into()
            }
            None => {
                return Err(error::ErrorUnauthorized(
                    "{\"Error\":\"Username or password is wrong\"}",
                ))
            }
        },
        Err(e) => {
            error!("Database error{}", e);
            HttpResponse::InternalServerError().body("{\"Error\":\"Something went wrong\"}")
        }
    };
    Ok(httpresponse)
}

#[post("/testifauth")]
#[macros::restricted_route]
async fn testifauth(data: web::Data<AppData>, session: Session) -> Result<impl Responder> {
    //debug!("{:?} ", user);
    Ok(HttpResponse::Ok().json(IsAuth { isauth: true }))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let secret_key = Key::generate();
    //gets database
    let db = block_on(DB::getdb()).expect("Could not create or find database:");
    std::env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();

    let app_data = AppData::new(db);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .wrap(Logger::default())
            .wrap_fn(|req, srv| {
                srv.call(req).map(|res| {
                    println!("Hi from response");
                    res
                })
            })
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .session_lifecycle(SessionLifecycle::PersistentSession(
                        PersistentSession::default(),
                    ))
                    .build(),
            )
            .service(testifauth)
            .service(autherization)
    })
    .bind("127.0.0.1:5376")?
    .run()
    .await
}
