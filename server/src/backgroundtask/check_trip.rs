use std::sync::Arc;

use actix::{Addr, Message};

use crate::db::{users,notification_info};
use super::{tripsSerilizing::Trips,notification_actor::NotificationActor};
#[derive(Clone)]
pub struct CheckOnItsWay {
    pub user: users::Model,
    pub trip: Trips,
    pub notification_actor: Addr<NotificationActor>,
    pub notification_infos: Arc<Vec<notification_info::Model>>,
}
impl Message for CheckOnItsWay {
    type Result = ();
}