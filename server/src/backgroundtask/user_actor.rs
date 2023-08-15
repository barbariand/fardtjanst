use std::sync::Arc;
use actix::{Addr, Message, Handler, Actor, Context};
use chrono::Utc;
use actix::AsyncContext;
use crate::{db::{users,notification_info}, backgroundtask::get_trip_request};
use log::{info,error};
use super::{notification_actor::NotificationActor,notification_actor::StartChecking, notifier::SendNotification, };

#[derive(Clone)]
pub struct UserActor {}

pub struct UserMessage {
    pub notification_infos: Arc<Vec<notification_info::Model>>,
    pub user: users::Model,
    pub notification_actor: Addr<NotificationActor>,
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

