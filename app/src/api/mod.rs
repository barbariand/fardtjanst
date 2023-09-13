use crate::API_TOKEN_STORAGE_KEY;
use api_structs::*;
use gloo_net::http::{Request, RequestBuilder, Response};
use gloo_storage::LocalStorage;
use gloo_storage::Storage;
use gloo_utils;
use leptos::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen;
use wasm_bindgen::prelude::*;
#[derive(Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
}

#[wasm_bindgen(inline_js = r#"function urlBase64ToUint8Array(base64String) {
    var padding = '='.repeat((4 - base64String.length % 4) % 4);
    var base64 = (base64String + padding)
        .replace(/\-/g, '+')
        .replace(/_/g, '/');

    var rawData = window.atob(base64);
    var outputArray = new Uint8Array(rawData.length);

    for (var i = 0; i < rawData.length; ++i) {
        outputArray[i] = rawData.charCodeAt(i);
    }
    return outputArray;
}
export function askForNotifications(str) {
    console.log(str);
    return new Promise(function (resolve, reject) {
        const permissionResult = Notification.requestPermission(function (result) {
            resolve(result);
        });

        if (permissionResult) {
            permissionResult.then(resolve, reject);
        }
    }).then(function (permissionResult) {
        if (permissionResult !== 'granted') {
            throw new Error("We weren't granted permission.");
        }
    });
    
}
export function subscribeUserToPush() {
    return navigator.serviceWorker
        .register('/resources/service-worker.js')
        .then(function (registration) {
            const subscribeOptions = {
                userVisibleOnly: true,
                applicationServerKey: urlBase64ToUint8Array(
                    'BBUBC-RSb16u6gyQvIo7ia1pf4cDQxoWrhCTyose3kC2UBg9u8-_I4hQJQxPsVVoIryJ7yLhUMcgjiOnoRs6dZc=',
                ),
            };

            return registration.pushManager.subscribe(subscribeOptions);
        })
        .then(function (pushSubscription) {
            
            console.log(
                'Received PushSubscription: ',
                JSON.stringify(pushSubscription),
            );
            return pushSubscription;
        });
}"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn subscribeUserToPush() -> std::result::Result<JsValue, JsValue>;
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiToken {
    pub token: String,
}
impl ApiToken {
    fn new(token: String) -> Self {
        Self { token }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}
#[derive(Clone, Copy)]
pub struct UnauthorizedApi {
    url: &'static str,
}

#[derive(Clone, PartialEq)]
pub struct AuthorizedApi {
    url: &'static str,
    token: ApiToken,
}

impl UnauthorizedApi {
    pub const fn new(url: &'static str) -> Self {
        Self { url }
    }
    pub async fn register(&self, credentials: &RegestringUser) -> Result<AuthorizedApi> {
        let url = format!("{}/register", self.url);
        let _ = Request::post(&url).json(credentials)?.send().await?;
        self.login(&(credentials.clone().into_user())).await
    }
    pub async fn login(&self, credentials: &User) -> Result<AuthorizedApi> {
        let url = format!("{}/login", self.url);
        let response = Request::post(&url).json(credentials)?.send().await?;
        if response.status() == 401 {
            Err(ApiError {
                message: String::from("username or password is incorrect"),
            })?
        }
        let html_document = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .dyn_into::<web_sys::HtmlDocument>()
            .unwrap();
        log!("{:?}", html_document.cookie().unwrap());
        let token = html_document.cookie().unwrap();
        Ok(AuthorizedApi::new(self.url, ApiToken::new(token)))
    }
}

impl AuthorizedApi {
    pub const fn new(url: &'static str, token: ApiToken) -> Self {
        Self { url, token }
    }
    fn auth_header_value(&self) -> String {
        format!("{}", self.token.token)
    }
    async fn send(&self, req: RequestBuilder) -> Result<Response> {
        req.header("id", &self.auth_header_value())
            .send()
            .await
            .map_err(Error::from)
    }
    async fn send_expect_json<T>(&self, req: RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        into_json(self.send(req).await?).await
    }
    pub async fn logout(&self) -> Result<()> {
        let url = format!("{}/logout", self.url);
        LocalStorage::delete(API_TOKEN_STORAGE_KEY);
        self.send(Request::post(&url)).await?;
        Ok(())
    }
    pub async fn user_info(&self) -> Result<UserInfo> {
        let url = format!("{}/users", self.url);
        self.send_expect_json(Request::get(&url)).await
    }
    pub async fn trips(&self) -> Result<TripsRequest> {
        let url = format!("{}/trips", self.url);
        self.send_expect_json(Request::get(&url)).await
    }
    pub fn token(&self) -> &ApiToken {
        &self.token
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum Error {
    #[error("{0:?}")]
    Fetch(String),
    #[error("{0:?}")]
    Api(ApiError),
}

impl From<ApiError> for Error {
    fn from(e: ApiError) -> Self {
        Self::Api(e)
    }
}
impl From<gloo_net::Error> for Error {
    fn from(value: gloo_net::Error) -> Self {
        Self::Fetch(value.to_string())
    }
}

async fn into_json<T>(response: Response) -> Result<T>
where
    T: DeserializeOwned,
{
    // ensure we've got 2xx status
    if response.ok() {
        Ok(response.json().await?)
    } else {
        Err(response.json::<ApiError>().await?.into())
    }
}
