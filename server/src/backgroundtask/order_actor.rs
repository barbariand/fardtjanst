use actix::fut::wrap_future;
use actix::{Message, Actor, Context, Handler, Addr, AsyncContext};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use actix_web::web::Bytes;
use actix::WrapFuture;
use log::error;
use crate::{MOCK_SERVER_URL, db::notification_info};
use crate::db::NotificationInfo;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use crate::routes::api::{get_authorized_request, RequestError};

use super::BackgroundActor;
use super::{notification_actor::NotificationActor, notifier::{SendNotification, self}};
#[derive(Clone)]
pub struct Order{
    pub data:Bytes,
    pub tries:usize,
    pub user:crate::routes::api::User,
    pub background_actor:Addr<BackgroundActor>,
}
impl Order{
    pub fn new(data:Bytes,user:impl crate::routes::api::IntoUser,background_actor:Addr<BackgroundActor>)->Self{
        Self{
            data,
            tries:0,
            user:user.into_user(),
            background_actor,
        }
    }
}
impl Message for Order{
    type Result = ();
}
pub struct OrderActor {
    pub data_base: sea_orm::DatabaseConnection,
    pub notification_actor: Addr<NotificationActor>,
}

impl Actor for OrderActor{
    type Context = Context<Self>;

}
#[derive(Deserialize, Serialize)]
struct Resa {
    pub time: i64,
    pub to_addres: String,
    pub from_addres: String,
}
const MAX_TRIES:usize=10;
const SECONDS_BETWEEN_TRIES:i64=60;
impl Handler<Order> for OrderActor {
    type Result = ();

    fn handle(&mut self, msg: Order, ctx: &mut Self::Context) {
        // Clone or copy necessary data
        let mut retry_msg=msg.clone();
        let user_id = msg.user.id;
        let data_base = self.data_base.clone();  // Assuming you can clone this
        let notification_actor = self.notification_actor.clone();

        // Create the future directly
        let fut = async move {
            let optinal_request = get_authorized_request(msg.user, reqwest::Method::GET, MOCK_SERVER_URL.to_string() + "/order").await;
            let optinal_prosesed_request = match optinal_request {
                Ok(r) => r.body(msg.data).send().await.map_err(RequestError::from),
                Err(e) => Err(e),
            };
            let mut notification = notifier::Notification::new("Färdtjänst Notifier".to_string())
            .add_icon("https://fardtjansten.regionstockholm.se/Resources/img/ft_icon_iPad.png".to_string())
            .add_badge("https://fardtjansten.regionstockholm.se/Resources/img/ft_icon_iPad.png".to_string());
            match optinal_prosesed_request {
                Ok(_r)=>{
                    serde_json::from_slice::<Resa>(&retry_msg.data);
                    notification=notification.add_body(format!("Resa bestäld från {} till {}, kl: {}",));
                }
                Err(e)=>{
                    retry_msg.tries+=1;
                    if msg.tries>MAX_TRIES{
                        notification=notification.add_body("Kunde inte beställa färdtjänst resa".to_string());
                    }else {
                        actix_rt::time::sleep(Duration::new(60,0)).await;
                        msg.background_actor.send(retry_msg);
                        return;
                    }
                }
            }
            
            for noti_info in NotificationInfo::find().filter(notification_info::Column::Id.eq(user_id)).all(&data_base).await.expect("it does not work") {
                let not=notification.clone();
                let message = SendNotification {
                    notification: not,
                    notification_info: noti_info.clone(),
                };

                match notification_actor.send(message).await {
                    Ok(_) => {},
                    Err(e) => error!("could not send {}", e),
                };
            }
        };

        // Spawn the future directly into the actor's context
        ctx.spawn(wrap_future(fut));
    }
}