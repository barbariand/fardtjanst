use crate::backgroundtask::TripsRequest;
use crate::db;
use actix::fut;
use actix::Arbiter;
use actix::AsyncContext;
use actix::{Actor, Context};
use actix_http::header::ToStrError;
use chrono::Utc;
use db::{
    notification_info,
    sea_orm::{self, DbErr},
    users, NotificationInfo, Users,
};
use std::future::Future;
use log::debug;
use log::error;
use log::info;

use sea_orm::EntityTrait;
use serde::Serialize;


use std::pin::Pin;
use std::sync::atomic::Ordering;

use std::{
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
};

use super::notifier;
use super::tripsSerilizing::ReservationStatusEnum;
use super::tripsSerilizing::Trips;

#[derive(Clone)]
pub struct BackgroundActor {
    running: Arc<AtomicBool>,
    data_base: sea_orm::DatabaseConnection,
}
impl BackgroundActor {
    pub fn new(data_base: sea_orm::DatabaseConnection) -> BackgroundActor {
        BackgroundActor {
            data_base,
            running: Arc::new(AtomicBool::new(true)),
        }
    }
    async fn run(self, db: sea_orm::DatabaseConnection, running: Arc<AtomicBool>) {
        let sleep_duration = Duration::from_secs(10);
        let duration_to_sleep = Duration::from_secs(30 * 60);
        let mut i: u8 = 0;
        let loops: u8 = duration_to_sleep.as_secs() as u8 / sleep_duration.as_secs() as u8;
        while running.load(Ordering::SeqCst) {
            if i == 0 {
                let resp = self.clone().handle_users(&db).await;
                if let Err(err) = resp {
                    error!("Error opening DB: {}", err)
                }
            }
            i = (i + 1) % loops;
            thread::sleep(sleep_duration);
        }
    }
    async fn handle_users(self, db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
        let notification_infos = NotificationInfo::find().all(db).await.map(Arc::new)?;
        let mut users = Users::find().all(db).await?;
        let a = Arbiter::new();
        let length = users.len();
        for _ in 0..length {
            let ua = UserActor {
                notification_infos: Arc::clone(&notification_infos),
                user: users.remove(0),
            };
            a.spawn_fn(move || {
                ua.start();
            });
        }
        Ok(())
    }
}
impl Actor for BackgroundActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("actor is online");
        ctx.spawn(fut::wrap_future::<_, Self>(BackgroundActor::run(
            self.clone(),
            self.data_base.clone(),
            self.running.clone(),
        )));
        let running = self.running.clone();
        let _ = ctrlc::set_handler(move || {
            running.store(false, Ordering::SeqCst);
        });
    }
    fn stopped(&mut self, _: &mut Self::Context) {
        info!("actor is offline");
    }
    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        self.running.store(false, Ordering::SeqCst);
        actix::Running::Stop
    }
}

#[derive(Serialize, Debug, Default)]
struct AuthSession {
    username: i32,
    password: String,
}
impl AuthSession {
    fn new(user: users::Model) -> Self {
        Self {
            username: user.card_nummer,
            password: user.password,
        }
    }
}
struct BackgroundTaskError {
    msg: String,
}
impl BackgroundTaskError {
    fn new(msg: String) -> Self {
        Self { msg }
    }
}
impl From<ToStrError> for BackgroundTaskError {
    fn from(value: ToStrError) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}
impl From<serde_json::Error> for BackgroundTaskError {
    fn from(value: serde_json::Error) -> Self {
        Self {
            msg: format!("serde_json error: {}", value),
        }
    }
}
impl From<reqwest::Error> for BackgroundTaskError {
    fn from(value: reqwest::Error) -> Self {
        Self {
            msg: format!("reqwest error: {}", value),
        }
    }
}

#[derive(Clone)]
struct UserActor {
    notification_infos: Arc<Vec<notification_info::Model>>,
    user: users::Model,
}
impl UserActor {
    async fn handle_user(self) {
        match self.handle_user_with_error().await {
            Ok(_) => {}
            Err(e) => {
                debug!("got an error rn");
                error!("{}", e.msg); //nicer to do this than like 5 match statements inside
            }
        }
    }
    async fn handle_user_with_error(&self) -> Result<(), BackgroundTaskError> {
        let tripsrequest = get_trip_request(&self.user).await?;
        if let Some(trips) = tripsrequest.customerTransportReservation {
            let after = Utc::now();
            let before = Utc::now()
                .checked_add_signed(chrono::Duration::minutes(30))
                .ok_or(BackgroundTaskError::new(
                    "failed to create duration beacuse overflow".to_string(),
                ))?;
            for trip in trips {
                if let Some(time) =trip.get_departure().get_time() &&time>=after&&time<=before{
                    let na=NotificationActor{
                        notification_infos:self.notification_infos.clone(),
                        user:self.user.clone(),
                        time,
                        trip,
                    };
                    actix::Arbiter::current().spawn_fn(move|| {
                        na.start();
                    });
                }
            }
        }
        Ok(())
    }
}
impl Actor for UserActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.spawn(fut::wrap_future::<_, Self>(Self::handle_user(self.clone())));
    }
}
#[derive(Clone)]
struct NotificationActor {
    notification_infos: Arc<Vec<notification_info::Model>>,
    user: users::Model,
    time: chrono::DateTime<Utc>,
    trip: Trips,
}
impl NotificationActor {
    async fn start_notification(self) {
        let notify: Vec<&notification_info::Model> = self
            .notification_infos
            .as_ref()
            .iter()
            .filter(|not| not.user_id == self.user.id)
            .collect();
        if notify.is_empty() {
            return;
        }
        let after = Utc::now();
        let time_untill = after - self.time;
        let std_time_untill_5 = time_untill
            .checked_sub(&chrono::Duration::minutes(5))
            .expect("")
            .to_std()
            .expect("could not create std duration");
        actix_rt::time::sleep(std_time_untill_5).await;
        let tripres = get_trip_request(&self.user).await;
        let trips = match tripres {
            Ok(trip) => match trip.customerTransportReservation {
                Some(s) => s,
                None => {
                    error!("trip/s was not found but was here before");
                    return;
                }
            },
            Err(e) => {
                error!("could not get trip beacuse {}", e.msg);
                return;
            }
        };
        let current_trip=match trips
            .iter()
            .filter(|t| t.id == self.trip.id)
            .collect::<Vec<&Trips>>()
            .first()
        {
            Some(s) => *s,
            None => {
                error!("trip was not found but was here before");
                return;
            }
        };
        if let Some(status) = current_trip.get_departure().get_status() {
            if status == ReservationStatusEnum::BilPåväg {
                self.send_notification(current_trip);
                return;
            }
        };
        self.recursivley_check_for_status_chamge().await;
    }
    fn recursive_check(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(self.recursivley_check_for_status_chamge())
    }
    
    async fn recursivley_check_for_status_chamge(&self) {
        actix_rt::time::sleep(Duration::from_secs(60)).await;
        let tripres = get_trip_request(&self.user).await;
        let trips = match tripres {
            Ok(trip) => match trip.customerTransportReservation {
                Some(s) => s,
                None => {
                    error!("trip/s was not found but was here before");
                    return;
                }
            },
            Err(e) => {
                error!("could not get trip because {}", e.msg);
                return;
            }
        };
        let current_trip=match trips
            .iter()
            .filter(|t| t.id == self.trip.id)
            .collect::<Vec<&Trips>>()
            .first()
        {
            Some(s) => *s,
            None => {
                error!("trip was not found but was here before");
                return;
            }
        };
        if let Some(status) = current_trip.get_departure().get_status() {
            if status == ReservationStatusEnum::BilPåväg {
                self.send_notification(current_trip);
                return;
            }
        };
        self.recursive_check().await;
    }

    fn send_notification(&self, current_trip: &Trips) {
        notifier::Notification::new("Färdtjänst Notifier".to_string())
            .add_icon(
                "https://fardtjansten.regionstockholm.se/Resources/img/ft_icon_iPad.png"
                    .to_string(),
            ).add_badge("https://fardtjansten.regionstockholm.se/Resources/img/ft_icon_iPad.png"
            .to_string())
            .add_body(format!(
                "Resa från {} till {} är påväg {}",
                current_trip.from.address,
                current_trip.to.address,
                (if current_trip.isShared {
                    "är delad"
                } else {
                    ""
                })
                .to_string()
            ));
    }
}
impl Actor for NotificationActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.spawn(fut::wrap_future::<_, Self>(Self::start_notification(
            self.clone(),
        )));
    }
}

async fn get_trip_request(user: &users::Model) -> Result<TripsRequest, BackgroundTaskError> {
    let client = reqwest::Client::new();
    debug!("user: {:?}", user);
    let authresponse = client
        .post(crate::MOCK_SERVER_URL.to_owned() + "/api/autherization")
        .json(&AuthSession::new(user.clone()))
        .send()
        .await?;
    info!("got code {}", authresponse.status());
    let headers = authresponse.headers();
    debug!("{:?}", headers);
    let cookieheader = headers.get("set-cookie").ok_or(BackgroundTaskError::new(
        "could not find headers".to_string(),
    ))?;
    let tripresponse = client
        .get(crate::MOCK_SERVER_URL.to_owned() + "/trips")
        .query(&TripsRequest::new(
            "".to_string(),
            "".to_string(),
            "".to_string(),
            0,
            10,
            None,
        ))
        .header(
            "Cookie",
            *cookieheader
                .to_str()?
                .split(';')
                .collect::<Vec<&str>>().first()
                .ok_or(BackgroundTaskError {
                    msg: "string is empty".to_string(),
                })?,
        )
        .send()
        .await?;

    tripresponse
        .json::<TripsRequest>()
        .await
        .map_err(|err| BackgroundTaskError {
            msg: format!("serde_json error : {}", err),
        })
}
