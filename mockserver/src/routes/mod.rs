pub mod api;

use crate::AppData;
use actix_session::Session;
use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use db::{
    resor,
    sea_orm::{
        self, sea_query::IntoCondition, ColumnTrait, EntityTrait, QueryFilter, QuerySelect,
        RelationTrait,
    },
    tempsessions, users,
};
use log::debug;
use macros;
use serde::Serialize;
mod trips;
pub use trips::TripsRequest;
#[derive(Serialize)]
struct IsAuth {
    isauth: bool,
}

#[get("/trips")]
#[macros::restricted_route]
pub(crate) async fn testifauth(
    data: web::Data<AppData>,
    session: Session,
) -> Result<impl Responder> {
    debug!("{:?} ", user);
    let mayberesor = resor::Entity::find()
        .filter(resor::Column::UserId.eq(user.id))
        .all(data.get_db())
        .await;
    let resorna = match mayberesor {
        Ok(resorna) => resorna,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };
    let sort_order = String::new();
    let group = String::new();
    let filter = String::new();
    let skip = 0;
    let take = 100;
    let remaning = 0;
    let tr = match TripsRequest::new(
        group, sort_order, filter, skip, take, remaning, &user, &resorna,
    ) {
        Ok(s) => s,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };
    Ok(HttpResponse::Ok().json(tr))
}
