use super::{RequestError,get_authorized_request};
use crate::db;
use crate::{AppData, MAX_PAYLOAD_SIZE, MOCK_SERVER_URL};
use actix_session::Session;
use actix_web::Result;
use actix_web::{error, post, web};
use actix_web::{ HttpResponse, Responder};
use db::{
    sea_orm::EntityTrait,
    tempsessions, users
};
use sea_orm::QuerySelect;
use sea_orm::RelationTrait;
use sea_orm::sea_query::IntoCondition;
use futures::StreamExt;

#[post("/api/trips")]
#[macros::restricted_route]
async fn trips(
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
    match get_authorized_request(&user,reqwest::Method::GET,MOCK_SERVER_URL.to_string()+"/trips").await.map(|r|r.body(body.freeze()).send()){
        Ok(r)=>{
            match r.await.map(|res|res.bytes()){
                Ok(res) => {
                    match res.await{
                        Ok(bytes)=>{
                            Ok(HttpResponse::Ok().body(bytes))
                        },Err(e)=>{Err(error::ErrorInternalServerError(e))}
                    }
                },
                Err(e) => Err(error::ErrorBadGateway(e))
            }
        },
        Err(e) => {
            match e {
                RequestError::Reqwest(_) => Ok(HttpResponse::ServiceUnavailable().finish()),
                RequestError::SerdeJson(_) => Ok(HttpResponse::ServiceUnavailable().finish()),
                RequestError::Header => Ok(HttpResponse::BadGateway().finish()),
            }
        },
    }
}