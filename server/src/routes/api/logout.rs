use actix_http::error::HttpError;
use actix_session::Session;
use actix_web::{ post, web, Responder};
use actix_web::{HttpResponse};
use crate::AppData;
use crate::db;
use actix_http::Error;
use db::{
    sea_orm::{ EntityTrait},
    tempsessions, users, 
};
use sea_orm::sea_query::IntoCondition;
use sea_orm::RelationTrait;
use sea_orm::QuerySelect;
#[post("/api/logout")]
#[macros::restricted_route]
async fn logout(
    data:web::Data<AppData>,
    session: Session)->impl Responder{
        session.purge();
    
        Ok::<HttpResponse, Error>(HttpResponse::Ok().json(()))
}