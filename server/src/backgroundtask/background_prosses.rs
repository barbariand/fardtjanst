use crate::db;
use actix::Handler;
use actix::dev::MessageResponse;
use actix::Addr;
use actix::AsyncContext;
use actix::Message;
use actix::{Actor, Context};

use db::{
    sea_orm::{self, DbErr}, NotificationInfo, Users,
};
use futures_util::future::join_all;
use log::error;
use log::info;
use sea_orm::EntityTrait;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use super::errors::BackgroundTaskError;
use super::notification_actor::NotificationActor;
use super::notifier::SendNotification;
use super::order_actor::Order;
use super::order_actor::OrderActor;
use super::user_actor::UserActor;
use super::user_actor::UserMessage;

#[derive(Clone)]
pub struct BackgroundActor {
    running: Arc<AtomicBool>,
    data_base: sea_orm::DatabaseConnection,
    user_actor: Addr<UserActor>,
    order_actor:Addr<OrderActor>,
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
        let not_addr=NotificationActor {}.start();
        let order_actor=OrderActor {data_base:data_base.to_owned(),notification_actor:not_addr.clone()};
        BackgroundActor {
            data_base,
            running: Arc::new(AtomicBool::new(true)),
            order_actor:order_actor.start(),
            user_actor: UserActor {}.start(),
            notification_actor: not_addr,
            sleep_duration,
            maxloops: loops,
            i: Arc::new(Mutex::new(0)),
        }
    }
    async fn run(self) {
        #![allow(clippy::await_holding_lock)] // this warning is fine beacuse we drop it later as repsonible programers
        let mut i = self.i.lock().unwrap();
        *i = (*i + 1) % self.maxloops;

        if *i == 1 {
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
impl MessageResponse<UserActor, UserMessage> for Result<(), BackgroundTaskError> {
    fn handle(
        self,
        _: &mut <UserActor as Actor>::Context,
        _: Option<actix::dev::OneshotSender<<UserMessage as Message>::Result>>,
    ) {
    }
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
impl Handler<Order> for BackgroundActor{
    type Result = ();
    fn handle(&mut self, msg: Order, _: &mut Self::Context) -> Self::Result {
        self.order_actor.send(msg);
    }
}