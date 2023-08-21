use api_structs::*;
use gloo_net::http::{Request, Response, RequestBuilder};
use gloo_storage::LocalStorage;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use leptos::*;
use gloo_storage::Storage;
use crate::API_TOKEN_STORAGE_KEY;
#[derive(Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
}


#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone)]
pub struct AuthorizedApi {
    url: &'static str,
    token: ApiToken,
}

impl UnauthorizedApi {
    pub const fn new(url: &'static str) -> Self {
        Self { url }
    }
    pub async fn register(&self, credentials: &User) -> Result<()> {
        let url = format!("{}/users", self.url);
        let response = Request::post(&url).json(credentials)?.send().await?;
        into_json(response).await
    }
    pub async fn login(&self, credentials: &User) -> Result<AuthorizedApi> {
        let url = format!("{}/login", self.url);
        let response = Request::post(&url).json(credentials)?.send().await?;
        if response.status()==401{
            Err(ApiError{message:String::from("username or password is incorrect")})?
        }
        let html_document = web_sys::window().unwrap().document().unwrap().dyn_into::<web_sys::HtmlDocument>().unwrap();
        log!("{:?}",html_document.cookie().unwrap());
        let token = html_document.cookie().unwrap();
        Ok(AuthorizedApi::new(self.url, ApiToken { token }))
    }
}

impl AuthorizedApi {
    pub const fn new(url: &'static str, token: ApiToken) -> Self {
        Self { url, token }
    }
    fn auth_header_value(&self) -> String {
        format!("{}", self.token.token)
    }
    async fn send(&self,req:RequestBuilder)->Result<Response>{
        req
            .header("id", &self.auth_header_value())
            .send()
            .await.map_err(Error::Fetch)
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

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Fetch(#[from] gloo_net::Error),
    #[error("{0:?}")]
    Api(ApiError),
}

impl From<ApiError> for Error {
    fn from(e: ApiError) -> Self {
        Self::Api(e)
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
