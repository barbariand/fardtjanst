mod login;
pub use login::*;
mod home;
pub use home::*;
mod register;
pub use register::*;
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
            Self::Login => "/login",
            Self::Register => "/register",
        }
    }
}
