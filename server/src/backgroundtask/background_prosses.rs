use actix::fut;
use actix::AsyncContext;
use actix::{Actor, Context};
use crate::db as db;
use db::sea_orm;
use log::info;
use std::sync::atomic::Ordering;
use std::{
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
};
trait FromMin {
    fn from_min(min:u64)->Duration;
}
impl FromMin for Duration{
    fn from_min(min:u64)->Duration {
        Duration::from_secs(min*60)
    }
}

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
    async fn run(running: Arc<AtomicBool>) {
        while running.load(Ordering::SeqCst) {
            info!("hello");
            thread::sleep(Duration::from_min(30));// this is the general loop for running everything that is needed if there is a resa close it should spawn other async functiosn to update more in the time interval
        }
    }
}
impl Actor for BackgroundActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("actor is online");
        ctx.spawn(fut::wrap_future::<_, Self>(BackgroundActor::run(
            self.running.clone(),
        )));
        let running=self.running.clone();
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


