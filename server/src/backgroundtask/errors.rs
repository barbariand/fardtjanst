use actix_http::header::ToStrError;

#[derive(Debug)]
pub struct BackgroundTaskError {
    pub msg: String,
}

impl BackgroundTaskError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}
impl From<ToStrError> for BackgroundTaskError {
    fn from(value: ToStrError) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}
impl From<serde_json::Error> for BackgroundTaskError {
    fn from(value: serde_json::Error) -> Self {
        Self {
            msg: format!("serde_json error: {}", value),
        }
    }
}
impl From<reqwest::Error> for BackgroundTaskError {
    fn from(value: reqwest::Error) -> Self {
        Self {
            msg: format!("reqwest error: {}", value),
        }
    }
}