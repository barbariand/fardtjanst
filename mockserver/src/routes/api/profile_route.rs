use crate::AppData;
use actix_session::Session;
use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use crate::db as db;
use db::{
    sea_orm::{self, sea_query::IntoCondition, EntityTrait, QuerySelect, RelationTrait},
    tempsessions, users,
};
use macros;
#[get("/profile")]
#[macros::restricted_route]
pub(crate) async fn profile(
    data: web::Data<AppData>,
    session: Session,
) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(user))
}