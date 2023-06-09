
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
    resor,
    sea_orm::{self, sea_query::IntoCondition, EntityTrait, QuerySelect, RelationTrait,ActiveValue::Set},
    tempsessions, users, Resor,
};
use futures::StreamExt;
use log::error;
use macros;
use serde::{Deserialize, Serialize};
use std::time;
#[derive(Deserialize, Serialize)]
struct Resa {
    pub id: i32,
    pub to_addres: String,
    pub from_addres: String,
    pub user_id: Option<i32>,
    pub time: i64,
    pub passagers: i32,
    pub child_passagers: i32,
    pub transport: String,
    pub from_id: i32,
    pub to_id: i32,
}
#[post("/order")]
#[macros::restricted_route]
async fn order(
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
    let resa = match serde_json::from_slice::<Resa>(&body) {
        Ok(au) => au,
        Err(e) => {
            error!(target:"/api/autherization",
                "faild to serilze {:?}, got error {}",
                body,
                e
            );
            return Err(error::ErrorBadRequest("Invalid json"));
        }
    };
    let mingap = 20 * 60 * 60;
    if time::Duration::from_millis((resa.time -mingap) as u64)
        > time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("time went backwards idk what you should do lmao")
    {
        return Ok(HttpResponse::Conflict().finish());
    }

    let insertresa = resor::ActiveModel {
        id: Set(resa.id),
        to_addres: Set(resa.to_addres),
        from_addres: Set(resa.from_addres),
        by_addres: Set(None),
        user_id: Set(Some(user.id)),
        time: Set(resa.time),
        passagers: Set(resa.passagers),
        child_passagers: Set(resa.child_passagers),
        is_shared: Set(false),
        can_be_new_trip_template: Set(true),
        cancelleable: Set(true),
        transport: Set(resa.transport),
        by_id: Set(None),
        from_id: Set(resa.from_id),
        to_id: Set(resa.to_id),
        company_name: Set(None),
        status: Set(None),
    };
    match Resor::insert(insertresa).exec(data.get_db()).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => {Ok(HttpResponse::InternalServerError().finish())}
    }
}
