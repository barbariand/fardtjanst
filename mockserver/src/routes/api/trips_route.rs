use super::tripsSerilizing::TripsRequest;
use crate::AppData;
use actix_session::Session;
use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use mockserverdb as db;
use db::{
    sea_orm::{self, sea_query::IntoCondition, EntityTrait, QuerySelect, RelationTrait},
    tempsessions, users,
};
use macros;

#[get("/trips")]
#[macros::restricted_route]
pub(crate) async fn trips(
    data: web::Data<AppData>,
    session: Session,
    query: web::Query<TripsRequest>,
) -> Result<impl Responder> {
    let trips = query.0;
    let resorquery = trips.generate_query();

    let mayberesor = resorquery.all(data.get_db()).await;
    let resorna = match mayberesor {
        Ok(resorna) => resorna,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };
    let tr = match trips.addTrips(&user, &resorna) {
        Ok(s) => s,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    Ok(HttpResponse::Ok().json(tr))
}
