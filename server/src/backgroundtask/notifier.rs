
use log::error;
use actix::{Message, Handler};
use serde_json;
use std::fs::File;
use web_push;
use actix::AsyncContext;
use crate::db::notification_info;
use api_structs::Notification;
use super::notification_actor::NotificationActor;
/// sends the notification usint the subscription_info 
/// will return error if it fails
pub async fn send_notification(
    subscription_info: web_push::SubscriptionInfo,
    notification: &Notification,
) -> Result<(), web_push::WebPushError> {
    //Read signing material for payload.
    let file = File::open("private_key.pem").unwrap();
    let sig_builder =
        web_push::VapidSignatureBuilder::from_pem(file, &subscription_info)?.build()?;

    //Now add payload and encrypt.
    let mut builder = web_push::WebPushMessageBuilder::new(&subscription_info)?;
    let notification_string = serde_json::to_string(&notification)?;
    let content: &[u8] = notification_string.as_bytes();
    builder.set_payload(web_push::ContentEncoding::Aes128Gcm, content);
    builder.set_vapid_signature(sig_builder);

    let client = web_push::WebPushClient::new()?;

    //Finally, send the notification!
    client.send(builder.build()?).await?;
    Ok(())
}
pub struct SendNotification {
    pub notification: Notification,
    pub notification_info: notification_info::Model,
}
impl Message for SendNotification {
    type Result = ();
}


impl Handler<SendNotification> for NotificationActor {
    type Result = ();
    fn handle(&mut self, msg: SendNotification, ctx: &mut Self::Context) -> Self::Result {
        ctx.spawn(actix::fut::wrap_future::<_, Self>(async move {
            if let Err(e) =
                send_notification(msg.notification_info.clone().into(), &msg.notification)
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