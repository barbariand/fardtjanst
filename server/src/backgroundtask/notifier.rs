use crate::db;
use actix_web::cookie::time::serde::timestamp;
use db::notification_info;
use serde::Serialize;
use serde_json;
use std::fs::File;
use web_push;
//Actions still need a
#[derive(Serialize)]
pub struct NotificationAction {
    action: String,       //action name
    title: String,        //tiltle of action
    icon: Option<String>, //icon URL
}
impl NotificationAction {
    fn new(action: String, title: String, icon: Option<String>) -> Self {
        Self {
            action,
            title,
            icon,
        }
    }
}
#[derive(Serialize)]
pub struct NotificationOptions {
    body: Option<String>,
    icon: Option<String>,
    image: Option<String>,
    badge: Option<String>,
    timestamp: Option<i64>,
    tag: Option<String>,
    data: Option<String>,
    vibrate: Option<Vec<u64>>,
    renotify: Option<bool>,
    action: Option<Vec<NotificationAction>>,
}
impl NotificationOptions {
    fn new() -> Self {
        Self {
            body: None,
            icon: None,
            image: None,
            badge: None,
            timestamp: None,
            tag: None,
            data: None,
            vibrate: None,
            renotify: None,
            action: None,
        }
    }
    fn add_body(&mut self,body:String){
        self.body=Some(body);
    }
    fn add_icon(&mut self,icon:String){
        self.icon=Some(icon);
    }
    fn add_image(&mut self,image:String){
        self.image=Some(image);
    }
    fn add_badge(&mut self,badge:String){
        self.badge=Some(badge);
    }
    fn add_timestamp(&mut self,timestamp:i64){
        self.timestamp=Some(timestamp);
    }
    fn add_tag(&mut self,tag:String){
        self.tag=Some(tag);
    }
    fn add_data(&mut self,data:String){
        self.data=Some(data);
    }
    fn add_vibrate(&mut self,vibrate:Vec<u64>){
        self.vibrate=Some(vibrate);
    }
    fn add_renotify(&mut self,renotify:bool){
        self.renotify=Some(renotify);
    }
    fn add_action(&mut self,action:NotificationAction){
        if self.action.is_none(){
            self.action=Some(Vec::new())
        }
        if let Some(ref mut v) = self.action {
            v.push(action);
        }
    }
}
#[derive(Serialize)]
pub struct Notification {
    title: String,
    options: NotificationOptions,
}
impl Notification{
    fn new(title:String)->Self{
        Self{
            title,
            options:NotificationOptions::new()
        }
    }
    fn add_body(&mut self,body:String){
        self.options.body=Some(body);
    }
    fn add_icon(&mut self,icon:String){
        self.options.icon=Some(icon);
    }
    fn add_image(&mut self,image:String){
        self.options.image=Some(image);
    }
    fn add_badge(&mut self,badge:String){
        self.options.badge=Some(badge);
    }
    fn add_timestamp(&mut self,timestamp:i64){
        self.options.timestamp=Some(timestamp);
    }
    fn add_tag(&mut self,tag:String){
        self.options.tag=Some(tag);
    }
    fn add_data(&mut self,data:String){
        self.options.data=Some(data);
    }
    fn add_vibrate(&mut self,vibrate:Vec<u64>){
        self.options.vibrate=Some(vibrate);
    }
    fn add_renotify(&mut self,renotify:bool){
        self.options.renotify=Some(renotify);
    }
    fn add_action(&mut self,action:NotificationAction){
        if self.options.action.is_none(){
            self.options.action=Some(Vec::new())
        }
        if let Some(ref mut v) = self.options.action {
            v.push(action);
        }
    }
}

pub async fn sendNotification(
    subscription_info: web_push::SubscriptionInfo,
    notification: Notification,
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
