mod background_prosses;
mod order_actor;
mod errors;
mod check_trip;
mod notification_actor;
mod user_actor;
use log::info;
use log::debug;
use errors::BackgroundTaskError;
use serde::Deserialize;
use serde::Serialize;
use crate::db::users;
pub mod notifier;
pub use order_actor::Order;
use api_structs::TripsRequest;
pub use background_prosses::{BackgroundActor};
trait IntoAuthUser{
    fn into_auth_user(self)->AuthUser;
}
#[derive(Serialize, Deserialize, Debug, Default)]
struct AuthUser {
    username: i32,
    password: String,
}
impl IntoAuthUser for &users::Model{
    fn into_auth_user(self)->AuthUser {
        AuthUser { username:self.card_nummer, password: self.password.clone() }
    }
}

pub async fn get_trip_request(user: &users::Model) -> Result<TripsRequest, BackgroundTaskError> {
    let client = reqwest::Client::new();
    debug!("user: {:?}", user);
    let authresponse = client
        .post(crate::MOCK_SERVER_URL.to_owned() + "/api/autherization")
        .json(&user.into_auth_user())
        .send()
        .await?;
    info!("got code {}", authresponse.status());
    let headers = authresponse.headers();
    debug!("{:?}", headers);
    let cookieheader = headers.get("set-cookie").ok_or(BackgroundTaskError::new(
        "could not find headers".to_string(),
    ))?;
    let tripresponse = client
        .get(crate::MOCK_SERVER_URL.to_owned() + "/trips")
        .query(&TripsRequest::new(
            "".to_string(),
            "".to_string(),
            "".to_string(),
            0,
            10,
            None,
        ))
        .header(
            "Cookie",
            *cookieheader
                .to_str()?
                .split(';')
                .collect::<Vec<&str>>()
                .first()
                .ok_or(BackgroundTaskError {
                    msg: "string is empty".to_string(),
                })?,
        )
        .send()
        .await?;

    tripresponse
        .json::<TripsRequest>()
        .await
        .map_err(|err| BackgroundTaskError {
            msg: format!("serde_json error : {}", err),
        })
}

