pub mod login;
pub mod order;
use std::string::ToString;
pub mod registernewuser;
use crate::{MOCK_SERVER_URL, db};
use reqwest::{ IntoUrl, RequestBuilder};
pub mod trips;
pub mod logout;
use api_structs::{User,IntoUser, RegestringUser};
impl IntoUser for &db::users::Model{
    fn into_user(self) -> User {
        User{
            username:self.card_nummer,
            password:self.password.clone(),
        }
    }
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
pub async fn get_authorized_request<U,V>(
    user: V,
    method: reqwest::Method,
    url: U,
) -> Result<RequestBuilder, RequestError>
where
    V:IntoUser,
    U: IntoUrl,
{
    let inner_user=user.into_user();
    let client = reqwest::Client::new();
    let cookie = get_correct_headers(&inner_user).await?;
    Ok(client.request(method, url).header("Cookie", cookie))
}
pub async fn get_correct_headers(
    user: &User,
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
