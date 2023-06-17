use crate::{AppData, MAX_PAYLOAD_SIZE, MOCK_SERVER_URL};
use actix_session::Session;
use actix_web::{error, post, web};
use actix_web::{HttpResponse, Responder,HttpRequest};
use actix_web::Result;
use std::fmt::Display;
use crate::db;
use db::{
    sea_orm::{ColumnTrait, EntityTrait, QueryFilter},
    tempsessions, users, TempSession, Users,
};
use futures::StreamExt;
use log::error;
use super::{User,RequestError};
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
    let user = match serde_json::from_slice::<User>(&body) {
        Ok(au) => au,
        Err(e) => {
            error!(target:"/api/autherization",
                "faild to serilze {:?} into {:?}, got error {}",
                body,
                User {
                    ..Default::default()
                },
                e
            );
            return Err(error::ErrorBadRequest("Invalid json"));
        }
    };
    if let Err(e)=super::execute_request_for(user, reqwest::Method::POST, MOCK_SERVER_URL.to_string()+"/").await{
        match e{
            RequestError::Reqwest(_) => todo!(),
            RequestError::SerdeJson(_) => Ok(HttpResponse::ServiceUnavailable()),
            RequestError::Header => Ok(HttpResponse::BadGateway()),
        }
    }
    Ok("not implimented")
}