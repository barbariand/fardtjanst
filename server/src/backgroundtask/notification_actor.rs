use std::{sync::Arc, time::Duration};
use log::error;
use actix::AsyncContext;
use actix::{Actor, Context, Addr, Message, Handler};
use crate::db::{users,notification_info};
use super::{tripsSerilizing::{Trips, ReservationStatusEnum}, get_trip_request, notifier::{self, SendNotification},check_trip::CheckOnItsWay};
use chrono::Utc;
#[derive(Clone)]
pub struct NotificationActor {}

impl Actor for NotificationActor {
    type Context = Context<Self>;
}
pub struct StartChecking {
    pub time: chrono::DateTime<Utc>,
    pub user: users::Model,
    pub trip: Trips,
    pub notification_actor: Addr<NotificationActor>,
    pub notification_infos: Arc<Vec<notification_info::Model>>,
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
                            " och är samåkning"
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
