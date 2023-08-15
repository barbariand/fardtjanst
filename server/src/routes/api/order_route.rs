use super::{RequestError, User,get_authorized_request};
use crate::backgroundtask::Order;
use crate::db;
use crate::{AppData, MAX_PAYLOAD_SIZE, MOCK_SERVER_URL};
use actix_session::Session;
use actix_web::Result;
use actix_web::{error, post, web};
use actix_web::{ HttpResponse, Responder};
use db::{
    sea_orm::{ColumnTrait, EntityTrait, QueryFilter},
    tempsessions, users, TempSession, Users,
};
use sea_orm::QuerySelect;
use sea_orm::RelationTrait;
use sea_orm::sea_query::IntoCondition;
use serde::Deserialize;
use futures::StreamExt;
#[post("/api/order")]
#[macros::restricted_route]
async fn register_user(
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
    data.addr.send(Order::new(body.freeze(), &user,data.addr.clone()));
    Ok(HttpResponse::Ok().finish())
}