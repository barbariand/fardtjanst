use crate::AppData;
use actix_session::Session;
use actix_web::{error, post, web};
use actix_web::{HttpResponse, Responder,HttpRequest};
use actix_web::Result;
use crate::db;
use db::{
    sea_orm::{ColumnTrait, EntityTrait, QueryFilter},
    tempsessions, users, TempSession, Users,
};
use futures::StreamExt;
use log::error;
use super::User;
use actix_web::http::header;

#[post("/api/login")]
async fn login(
    data: web::Data<AppData>,
    session: Session,
    req: HttpRequest,
    mut payload: web::Payload,
) -> Result<impl Responder> {
    log::info!("test");
    let auth:User=match req.headers().get(header::CONTENT_TYPE) {
        Some(content_type) if content_type == header::HeaderValue::from_static("application/x-www-form-urlencoded") => {
            let body = payload.next().await.ok_or_else(|| error::ErrorBadRequest("Bad Request Could not get form data"))??;
            serde_urlencoded::from_bytes(&body)
                .map_err(|e| error::ErrorBadRequest(format!("Bad Request {}",e)))?
        },
        Some(content_type) if content_type == header::HeaderValue::from_static("application/json") => {
            let body = payload.next().await.ok_or_else(|| error::ErrorBadRequest("Bad Request"))??;
            serde_json::from_slice(&body)
                .map_err(|e| error::ErrorBadRequest(format!("Bad Request {}",e)))?
            
        },
        _ => return Err(error::ErrorBadRequest("Invalid Content-Type")),
    };
    log::info!("{:?}",&auth);
    let userres = Users::find()
        .filter(users::Column::Name.eq(auth.username))
        .one(data.get_db())
        .await;
    let httpresponse: HttpResponse = match userres {
        Ok(user) => match user {
            Some(u) => {
                if auth.password!=u.password{
                    return Err(error::ErrorUnauthorized(
                        "{\"Error\":\"Username or password is wrong\"}",
                    ));
                }
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
