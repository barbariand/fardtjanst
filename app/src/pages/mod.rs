mod components;
use components::*;
mod login;
pub use login::*;
mod home;
use home::*;
use leptos::*;
use leptos_router::*;

#[derive(Debug, Clone, Copy)]
pub enum Page {
    Home,
    Login,
    Register,
}

impl Page {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Login => "/api/autherization",
            Self::Register => "/api/register",
        }
    }
}