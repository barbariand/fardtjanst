
use crate::backgroundtask::TripsRequest;
use crate::db;
use actix::fut;
use actix::Arbiter;
use actix::AsyncContext;
use actix::{Actor, Context};
use actix_http::header::ToStrError;
use chrono::DateTime;
use chrono::Utc;
use db::{
    notification_info,
    sea_orm::{self, DbErr},
    users, NotificationInfo, Users,
};
use log::debug;
use log::error;
use log::info;
use reqwest;
use sea_orm::EntityTrait;
use serde::Serialize;
use serde_json;
use std::fmt::format;
use std::sync::atomic::Ordering;

use std::{
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
};

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
    async fn run(self,db: sea_orm::DatabaseConnection, running: Arc<AtomicBool>) {
        let sleep_duration = Duration::from_secs(10);
        let duration_to_sleep = Duration::from_secs(30 * 60);
        let mut i: u8 = 0;
        let loops: u8 = duration_to_sleep.as_secs() as u8 / sleep_duration.as_secs() as u8;
        while running.load(Ordering::SeqCst) {
            if i == 0 {
            
                let resp =  self.clone().handle_users(&db).await;
                match resp {
                    Err(err) => {
                        error!("Error opening DB: {}", err)
                    }
                    Ok(_) => {}
                }
            }
            i = (i + 1) % loops;
            thread::sleep(sleep_duration);
        }
    }
    async fn handle_users(self,db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
        let notification_infos = NotificationInfo::find().all(db).await.map(Arc::new)?;
        let mut users = Users::find().all(db).await?;
        let a = Arbiter::new();
        let length=users.len();
        for i in 0..length {
            a.spawn(handle_user(self.clone(),Arc::clone(&notification_infos), users.remove(0)));
        }
        Ok(())
    }
    
    
    async fn handle_user_with_error(&self,
        notification_infos: Arc<Vec<notification_info::Model>>,
        user: users::Model,
    ) -> Result<(), BackgroundTaskError> {
        let tripsrequest=self.get_trip_request(&user).await?;
        if let Some(trips) = tripsrequest.customerTransportReservation {
            let after = Utc::now();
            let before = Utc::now()
                .checked_add_signed(chrono::Duration::minutes(30))
                .ok_or(BackgroundTaskError::new(
                    "failed to create duration beacuse overflow".to_string(),
                ))?;
            for trip in trips {
                if let Some(time) =trip.get_departure().get_time() &&time>=after&&time<=before{
                    let time_untill=after-time;
                        
                }
            }
        }
        Ok(())
    }
    async fn get_trip_request(&self,user: &users::Model)->Result<TripsRequest,BackgroundTaskError>{
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
        let cookieheader = headers
            .get("set-cookie")
            .ok_or(BackgroundTaskError::new("could not find headers".to_string()))?;
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
            .header("Cookie", (*cookieheader.to_str()?.split(";").collect::<Vec<&str>>().get(0).ok_or(BackgroundTaskError{msg:"string is empty".to_string()})?)).send().await?;
        tripresponse.json::<TripsRequest>().await.map_err(|err|{
            BackgroundTaskError { msg: format!("serde_json error : {}",err) }
        })
    }
    async fn notify_about_trip(&self,time_untill:Duration, notification_infos: Arc<Vec<notification_info::Model>>,user:users::Model,){
        
    }
}

async fn handle_user(myself:BackgroundActor,notification_infos: Arc<Vec<notification_info::Model>>, user: users::Model) {
    match myself.handle_user_with_error(notification_infos, user).await {
        Ok(_) => {}
        Err(e) => {
            debug!("got an error rn");
            error!("{}", e.msg); //nicer to do this than like 5 match statements inside
        }
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
impl From<ToStrError> for BackgroundTaskError{
    fn from(value: ToStrError) -> Self {
        Self{
            msg:value.to_string()
        }
    }
}
impl From<serde_json::Error> for BackgroundTaskError {
    fn from(value: serde_json::Error) -> Self {
        Self {
            msg: format!("serde_json error: {}",value),
        }
    }
}
impl From<reqwest::Error> for BackgroundTaskError {
    fn from(value: reqwest::Error) -> Self {
        Self {
            msg: format!("reqwest error: {}",value),
        }
    }
}


impl Actor for BackgroundActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("actor is online");
        ctx.spawn(fut::wrap_future::<_, Self>(
            BackgroundActor::run(
                self.clone(),
                self.data_base.clone(),
                self.running.clone(),
            )
        ));
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
