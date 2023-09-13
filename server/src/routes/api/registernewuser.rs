use super::{RequestError, User};
use crate::db;
use crate::{AppData, MAX_PAYLOAD_SIZE, MOCK_SERVER_URL};
use actix_web::Result;
use actix_web::{error, post, web};
use actix_web::{ HttpResponse, Responder};
use db::{
    sea_orm::EntityTrait,
     users, Users,
};
use futures::StreamExt;
use log::error;
use sea_orm::ActiveValue;

use api_structs::RegestringUser;
use serde::Deserialize;
#[derive(Deserialize)]
struct RegisterdUser{
    pub name: String,
    pub password: String,
    pub card_nummer: i32,
    pub phone_number: String,
}
#[post("/api/register")]
async fn register_user(
    data: web::Data<AppData>,
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
    let user = match serde_json::from_slice::<RegestringUser>(&body) {
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
    let res=match super::get_authorized_request(
        user.clone(),
        reqwest::Method::GET,
        MOCK_SERVER_URL.to_string() + "/profile",
    )
    .await
    {
        Err(e) => {
            return match e {
                RequestError::Reqwest(_) => Ok(HttpResponse::ServiceUnavailable()),
                RequestError::SerdeJson(_) => Ok(HttpResponse::ServiceUnavailable()),
                RequestError::Header => Ok(HttpResponse::BadGateway()),
            };
        },
        Ok(r)=>{
            match r.send().await{
                Ok(s)=>s,
                Err(_)=>return Ok(HttpResponse::ServiceUnavailable()),
            }
        }
    };
    let register_user=match res.json::<RegisterdUser>().await{
        Err(_)=>{
            return Ok(HttpResponse::BadGateway())
        }
        Ok(u)=>u,
    };
    let insert=users::ActiveModel{
        card_nummer:ActiveValue::Set(register_user.card_nummer),
        name:ActiveValue::Set(user.name),
        färdjänst_passsword:ActiveValue::Set(user.färtjänst_password),
        phone_number:ActiveValue::Set(register_user.phone_number),
        password:ActiveValue::Set(user.password),
        ..Default::default()
    };
    Users::insert(insert).exec(data.get_db()).await.expect("db is down");
    Ok(HttpResponse::Ok())
}
