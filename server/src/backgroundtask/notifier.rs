

use serde::Serialize;
use serde_json;
use std::fs::File;
use web_push;
//Actions still need a
#[derive(Serialize,Debug,Clone)]
pub struct NotificationAction {
    action: String,       //action name
    title: String,        //tiltle of action
    icon: Option<String>, //icon URL
}
impl NotificationAction {
    #[expect(dead_code)]
    fn new(action: String, title: String, icon: Option<String>) -> Self {
        Self {
            action,
            title,
            icon,
        }
    }
}
/// this struct is a Rust implementation for the options https://developer.mozilla.org/en-US/docs/Web/API/Notification/Notification 
#[derive(Serialize,Debug,Clone)]
struct NotificationOptions {
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
    /// the body to be used
    fn add_body(&mut self, body: String) {
        self.body = Some(body);
    }
    fn add_icon(&mut self, icon: String) {
        self.icon = Some(icon);
    }
    #[expect(dead_code)]
    fn add_image(&mut self, image: String) {
        self.image = Some(image);
    }
    fn add_badge(&mut self, badge: String) {
        self.badge = Some(badge);
    }
    fn add_timestamp(&mut self, timestamp: i64) {
        self.timestamp = Some(timestamp);
    }
    fn add_tag(&mut self, tag: String) {
        self.tag = Some(tag);
    }
    fn add_data(&mut self, data: String) {
        self.data = Some(data);
    }
    fn add_vibrate(&mut self, vibrate: Vec<u64>) {
        self.vibrate = Some(vibrate);
    }
    fn renotify(&mut self, renotify: bool) {
        self.renotify = Some(renotify);
    }
    fn add_action(&mut self, action: NotificationAction) {
        if self.action.is_none() {
            self.action = Some(Vec::new())
        }
        if let Some(ref mut v) = self.action {
            v.push(action);
        }
    }
}
/// this struct is a Rust implementation for the 
/// [notification api](https://developer.mozilla.org/en-US/docs/Web/API/Notification/Notification)
/// and in js it should later be resived as <br>
/// ``` js
/// self.addEventListener('push', function(event) {
/// let not=event.data.json();
/// const promiseChain = self.registration.showNotification(not.title,not.options)
/// event.waitUntil(promiseChain);
/// });
/// ```
/// is not Deserialize as it shold not be resived
#[derive(Serialize,Debug,Clone)]
pub struct Notification {
    title: String,
    options: NotificationOptions,
}
impl Notification {
    pub fn new(title: String) -> Self {
        Self {
            title,
            options: NotificationOptions::new(),
        }
    }
    pub fn add_body(mut self, body: String)->Self {
        self.options.add_body(body);
        self
    }
    pub fn add_icon(mut self, icon: String)->Self {
        self.options.add_icon(icon);
        self
    }
    #[expect(dead_code)]
    pub fn add_image(mut self, image: String)->Self {
        self.options.add_image(image);
        self
    }
    pub fn add_badge(mut self, badge: String)->Self {
        self.options.add_badge(badge);
        self
    }
    pub fn add_timestamp(mut self, timestamp: i64)->Self {
        self.options.add_timestamp(timestamp);
        self
    }
    fn add_tag(mut self, tag: String)->Self {
        self.options.add_tag(tag);
        self
    }
    fn add_data(mut self, data: String)->Self {
        self.options.add_data(data);
        self
    }
    fn add_vibrate(mut self, vibrate: Vec<u64>)->Self {
        self.options.add_vibrate(vibrate);
        self
    }
    fn renotify(mut self, renotify: bool)->Self {
        self.options.renotify(renotify);
        self
    }
    fn add_action(mut self, action: NotificationAction)->Self {
        self.options.add_action(action);
        self
    }
}

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
