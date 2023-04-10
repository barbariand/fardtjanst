use super::tripsSerilizing::Trips;
use crate::{AppData, MAX_PAYLOAD_SIZE};
use actix_session::Session;
use actix_web::error;
use actix_web::post;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use db::{resor,
    sea_orm::{self, sea_query::IntoCondition, EntityTrait, QuerySelect, RelationTrait},
    tempsessions, users,
};

use futures::StreamExt;
use log::debug;
use log::error;
use macros;
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
    let resor = match serde_json::from_slice::<resor::ActiveModel>(&body) {
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
    Ok(HttpResponse::Ok().into())
}
