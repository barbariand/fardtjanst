mod autherization_route;
mod order_route;
mod registernewuser;
use std::ops::Deref;

use crate::MOCK_SERVER_URL;
use actix_web::http::header;
pub use autherization_route::autherization;
use futures_util::Future;
pub use registernewuser::register_user;
use reqwest::{Client, IntoUrl, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct User {
    pub username: i32,
    pub password: String,
}

pub enum RequestError {
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    Header,
}
impl From<reqwest::Error> for RequestError {
    fn from(value: reqwest::Error) -> Self {
        RequestError::Reqwest(value)
    }
}
impl From<serde_json::Error> for RequestError {
    fn from(value: serde_json::Error) -> Self {
        RequestError::SerdeJson(value)
    }
}
pub async fn execute_request_for<U>(
    user: User,
    method: reqwest::Method,
    url: U,
) -> Result<RequestBuilder, RequestError>
where
    U: IntoUrl,
{
    let client = reqwest::Client::new();
    let cookie = get_correct_headers(user).await?;
    Ok(client.request(method, url).header("Cookie", cookie))
}
pub async fn get_correct_headers(
    user: User,
) -> Result<actix_http::header::HeaderValue, RequestError> {
    let client = reqwest::Client::new();
    let auth_response = client
        .post(MOCK_SERVER_URL.to_string() + "/api/autherization")
        .body(serde_json::to_string(&user)?)
        .send()
        .await?;
    Ok(auth_response
        .headers()
        .get("id")
        .ok_or_else(|| RequestError::Header)?
        .clone())
}
