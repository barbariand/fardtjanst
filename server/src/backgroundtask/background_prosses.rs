use crate::backgroundtask::TripsRequest;
use crate::db;
use actix::dev::MessageResponse;
use actix::fut;
use actix::Addr;
use actix::AsyncContext;
use actix::Handler;
use actix::Message;
use actix::SyncContext;
use actix::{Actor, Context};
use actix_http::header::ToStrError;
use actix_rt::Arbiter;
use chrono::Utc;
use db::{
    notification_info,
    sea_orm::{self, DbErr},
    users, NotificationInfo, Users,
};
use futures::executor::block_on;
use futures_util::future::join_all;
use log::debug;
use log::error;
use log::info;
use sea_orm::EntityTrait;
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

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
    user_actor: Addr<UserActor>,
    notification_actor: Addr<NotificationActor>,
    sleep_duration: Duration,
    maxloops: u64,
    i: Arc<Mutex<u64>>,
}
impl BackgroundActor {
    pub fn new(data_base: sea_orm::DatabaseConnection) -> BackgroundActor {
        let sleep_duration = Duration::from_secs(10);
        let duration_to_sleep = Duration::from_secs(30 * 60);
        let loops: u64 = duration_to_sleep.as_secs() / sleep_duration.as_secs();
        BackgroundActor {
            data_base,
            running: Arc::new(AtomicBool::new(true)),
            user_actor: UserActor {}.start(),
            notification_actor: NotificationActor {}.start(),
            sleep_duration,
            maxloops: loops,
            i: Arc::new(Mutex::new(0)),
        }
    }
    async fn run(self) {
        #![allow(clippy::await_holding_lock)] // this warning is fine beacuse we drop it later as repsonible programers
        let mut i = self.i.lock().unwrap();
        *i = (*i + 1) % self.maxloops;
        info!("loop: {}", *i);
        if *i == 1 {
            drop(i); // droped here as it is no longer needed
            if let Err(err) = self.handle_users().await {
                error!("Error opening DB: {}", err)
            }
        }
    }

    async fn handle_users(&self) -> Result<(), DbErr> {
        let res = Users::find().all(&self.data_base).await?;
        let notification_infos = Arc::new(NotificationInfo::find().all(&self.data_base).await?);
        let mut users = res;
        let length = users.len();
        join_all((0..length).map(move |_| {
            let ua = UserMessage {
                notification_infos: notification_infos.clone(),
                user: users.remove(0),
                notification_actor: self.notification_actor.clone(),
            };
            self.user_actor.send(ua)
        }))
        .await;
        Ok(())
    }
}
impl Actor for BackgroundActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("actor is online");
        ctx.run_interval(self.sleep_duration, move |actor, ctx| {
            let fut = actor.clone().run();
            ctx.spawn(actix::fut::wrap_future::<_, Self>(fut));
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
#[derive(Debug)]
struct BackgroundTaskError {
    msg: String,
}
impl MessageResponse<UserActor, UserMessage> for Result<(), BackgroundTaskError> {
    fn handle(
        self,
        _: &mut <UserActor as Actor>::Context,
        _: Option<actix::dev::OneshotSender<<UserMessage as Message>::Result>>,
    ) {
    }
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
struct UserActor {}

struct UserMessage {
    notification_infos: Arc<Vec<notification_info::Model>>,
    user: users::Model,
    notification_actor: Addr<NotificationActor>,
}
impl Message for UserMessage {
    type Result = ();
}
impl Handler<UserMessage> for UserActor {
    type Result = ();
    fn handle(&mut self, msg: UserMessage, ctx: &mut Self::Context) {
        let addr = msg.notification_actor.clone();

        ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
            info!("hello from user");
            let tripsrequest = get_trip_request(&msg.user).await.unwrap();
            if let Some(trips) = tripsrequest.customerTransportReservation {
                let after = Utc::now();
                let before = Utc::now()
                    .checked_add_signed(chrono::Duration::minutes(30))
                    .unwrap();

                for trip in trips {
                    if let Some(time) =trip.get_departure().get_time() &&time>=after&&time<=before{
                        let na=StartChecking{
                            notification_infos:msg.notification_infos.clone(),
                            user:msg.user.clone(),
                            time,
                            trip,
                            notification_actor:addr.clone()
                        };
                        match addr.send(na).await{
                            Ok(_)=>{}
                            Err(e)=>{
                                error!("could not send {}",e)
                            }
                        };
                    }
                }
            }
        }));
    }
}
impl Actor for UserActor {
    type Context = Context<Self>;
}
#[derive(Clone)]
struct NotificationActor {}

impl Actor for NotificationActor {
    type Context = Context<Self>;
}
struct StartChecking {
    time: chrono::DateTime<Utc>,
    user: users::Model,
    trip: Trips,
    notification_actor: Addr<NotificationActor>,
    notification_infos: Arc<Vec<notification_info::Model>>,
}
impl Message for StartChecking {
    type Result = ();
}
impl Handler<StartChecking> for NotificationActor {
    type Result = ();
    fn handle(&mut self, msg: StartChecking, ctx: &mut Self::Context) -> Self::Result {
        let notify: Vec<&notification_info::Model> = msg
            .notification_infos
            .as_ref()
            .iter()
            .filter(|not| not.UserId == msg.user.id)
            .collect();
        if notify.is_empty() {
            return;
        }
        ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
            let after = Utc::now();
            let time_untill = after - msg.time;
            let std_time_untill_5 = time_untill
                .checked_sub(&chrono::Duration::minutes(5))
                .expect("")
                .to_std()
                .expect("could not create std duration");
            actix_rt::time::sleep(std_time_untill_5).await;
            let tripres = get_trip_request(&msg.user).await;
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
            let current_trip = match trips
                .iter()
                .filter(|t| t.id == msg.trip.id)
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
                    let notification = notifier::Notification::new(
                        "Färdtjänst Notifier".to_string(),
                    )
                    .add_icon(
                        "https://fardtjansten.regionstockholm.se/Resources/img/ft_icon_iPad.png"
                            .to_string(),
                    )
                    .add_badge(
                        "https://fardtjansten.regionstockholm.se/Resources/img/ft_icon_iPad.png"
                            .to_string(),
                    )
                    .add_body(format!(
                        "Resa från {} till {} är påväg {}",
                        current_trip.from.address,
                        current_trip.to.address,
                        (if current_trip.isShared {
                            " och är delad"
                        } else {
                            ""
                        })
                        .to_string()
                    ));
                    for notification_info in msg.notification_infos.as_ref() {
                        let message = SendNotification {
                            notification: notification.clone(),
                            notification_info: notification_info.clone(),
                        };
                        match msg.notification_actor.send(message).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("could not send beacuse {}", e)
                            }
                        }
                    }
                    return;
                }
            };
            match msg
                .notification_actor
                .send(CheckOnItsWay {
                    user: msg.user,
                    trip: msg.trip,
                    notification_actor: msg.notification_actor.clone(),
                    notification_infos: msg.notification_infos.clone(),
                })
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!("could not send beacuse {}", e)
                }
            };
        }));
    }
}
#[derive(Clone)]
struct CheckOnItsWay {
    user: users::Model,
    trip: Trips,
    notification_actor: Addr<NotificationActor>,
    notification_infos: Arc<Vec<notification_info::Model>>,
}
impl Message for CheckOnItsWay {
    type Result = ();
}
impl Handler<CheckOnItsWay> for NotificationActor {
    type Result = ();
    fn handle(&mut self, msg: CheckOnItsWay, ctx: &mut Self::Context) -> Self::Result {
        ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
            actix_rt::time::sleep(Duration::from_secs(60)).await;
            let tripres = get_trip_request(&msg.user).await;
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
            let current_trip = match trips
                .iter()
                .filter(|t| t.id == msg.trip.id)
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
                    let notification = notifier::Notification::new(
                        "Färdtjänst Notifier".to_string(),
                    )
                    .add_icon(
                        "https://fardtjansten.regionstockholm.se/Resources/img/ft_icon_iPad.png"
                            .to_string(),
                    )
                    .add_badge(
                        "https://fardtjansten.regionstockholm.se/Resources/img/ft_icon_iPad.png"
                            .to_string(),
                    )
                    .add_body(format!(
                        "Resa från {} till {} är påväg {}",
                        current_trip.from.address,
                        current_trip.to.address,
                        (if current_trip.isShared {
                            " och är delad"
                        } else {
                            ""
                        })
                        .to_string()
                    ));
                    for notification_info in msg.notification_infos.as_ref() {
                        let message = SendNotification {
                            notification: notification.clone(),
                            notification_info: notification_info.clone(),
                        };
                        match msg.notification_actor.send(message).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("could not send {}", e)
                            }
                        };
                    }
                    return;
                }
            };
            match msg.notification_actor.send(msg.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    error!("could not send {}", e)
                }
            };
        }));
    }
}
struct SendNotification {
    notification: notifier::Notification,
    notification_info: notification_info::Model,
}
impl Message for SendNotification {
    type Result = ();
}
impl Handler<SendNotification> for BackgroundActor {
    type Result = ();
    fn handle(&mut self, msg: SendNotification, ctx: &mut Self::Context) -> Self::Result {
        let notification_actor=self.notification_actor.clone();
        ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
            match notification_actor.send(msg).await {
                Ok(_) => {}
                Err(e) => {
                    error!("could not send {}", e)
                }
            };
        }));
    }
}
impl Handler<SendNotification> for NotificationActor {
    type Result = ();
    fn handle(&mut self, msg: SendNotification, ctx: &mut Self::Context) -> Self::Result {
        ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
            if let Err(e) =
                notifier::send_notification(msg.notification_info.clone().into(), &msg.notification)
                    .await
            {
                use web_push::WebPushError::*;
                match e {
                    Unauthorized | InvalidUri | EndpointNotValid | EndpointNotFound => {
                        // should be removed
                    }
                    BadRequest(s) => {
                        error!("request failed beacuse: {:?}", s);
                    }
                    PayloadTooLarge => {
                        error!("payload to big: {:?}", &msg.notification)
                    }
                    _ => {}
                }
            }
        }));
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
                .collect::<Vec<&str>>()
                .first()
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
struct OrderActor {
    data_base: sea_orm::DatabaseConnection,
}
