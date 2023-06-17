use super::tripsSerilizing::TripsRequest;
use crate::AppData;
use crate::routes::api::tripsSerilizing::ReservationStatus;
use crate::routes::api::tripsSerilizing::ReservationStatusEnum;
use actix_session::Session;
use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Result;
use chrono::Duration;
use chrono::Utc;
use rand;
use crate::db as db;
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
    let trips = query.into_inner();
    let resorquery = trips.generate_query();

    let mayberesor = resorquery.all(data.get_db()).await;
    let resorna = match mayberesor {
        Ok(resorna) => resorna,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };
    let mut tr = match trips.addTrips(&user, &resorna) {
        Ok(s) => s,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e)),
    };
    if let Some(trip_list)=&mut tr.customerTransportReservation{
        for trip in trip_list{
            if let Some(departure_time)=trip.departure.get_time(){
                let after = Utc::now();
                let time_untill=after-departure_time;
                if time_untill<Duration::minutes(30)&&time_untill>Duration::minutes(-30){
                    let adjusted_seconds_until_departure=-time_untill.num_seconds()+30*60;
                    if (rand::random::<f64>()*60.0*60.0)>adjusted_seconds_until_departure as f64||trip.departure.customerInfo.reservationStatus.as_ref().is_some_and(|v|{v.status==ReservationStatusEnum::BilPåväg}){
                        trip.departure.set_status(ReservationStatus{status:ReservationStatusEnum::BilPåväg});
                    }else{
                        trip.departure.set_status(ReservationStatus{status:ReservationStatusEnum::LetarEfterBil});
                    }

                }
            }
        }
    }
    Ok(HttpResponse::Ok().json(tr))
}
