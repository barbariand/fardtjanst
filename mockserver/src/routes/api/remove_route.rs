use crate::db;
use crate::{AppData, MAX_PAYLOAD_SIZE};
use actix_session::Session;
use actix_web::delete;
use actix_web::error;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use chrono::Utc;
use db::{

    sea_orm::{
        self, sea_query::IntoCondition, EntityTrait, QuerySelect, RelationTrait,
    },
    tempsessions, users, Resor,
};
use futures::StreamExt;
use log::error;
use macros;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
struct Resa {
    pub id: i32,
}
#[delete("/remove")]
#[macros::restricted_route]
async fn remove(
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
    match Resor::find_by_id(resa.id).one(&data.data_base).await {
        Ok(Some(foundresa)) => {
            if foundresa.time
                < Utc::now()
                    .checked_add_signed(chrono::Duration::minutes(20))
                    .expect("could not add time ").timestamp()
            {
                Ok(HttpResponse::Conflict().finish())
            } else {
                match Resor::delete_by_id(foundresa.id)
                    .exec(&data.data_base)
                    .await
                {
                    Ok(s) => {
                        if s.rows_affected > 1 {
                            error!("more rows where affected than expected");
                        }
                        Ok(HttpResponse::Ok().finish())
                    }
                    Err(e) => {
                        error!("could not acces db: {}", e);
                        Ok(HttpResponse::NotFound().finish())
                    }
                }
            }
        }
        Ok(None)=>{
            Ok(HttpResponse::NotFound().finish())
        }
        Err(e) => {
            error!("could not acces db: {}", e);
            Ok(HttpResponse::NotFound().finish())
        }
    }
}
