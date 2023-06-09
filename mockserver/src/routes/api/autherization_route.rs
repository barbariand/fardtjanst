use crate::{AppData, MAX_PAYLOAD_SIZE};
use actix_session::Session;
use actix_web::error;
use actix_web::post;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use crate::db as db;
use db::{
    sea_orm::{ColumnTrait, EntityTrait, QueryFilter},
    tempsessions, users, TempSession, Users,
};
use futures::StreamExt;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct AuthSession {
    username: i32,
    password: String,
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
    let userres = Users::find()
        .filter(users::Column::CardNummer.eq(auth.username))
        .one(data.get_db())
        .await;
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
