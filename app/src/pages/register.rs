use leptos::*;
use leptos_router::*;

use api_structs::*;

use crate::{
    api::{self, AuthorizedApi, UnauthorizedApi},
    components::register_credentials::*,
    Page,
};

#[component]
pub fn Register<F>(cx: Scope, api: UnauthorizedApi, on_success: F) -> impl IntoView
where
    F: Fn(AuthorizedApi) + 'static + Clone,
{
    let (register_error, set_login_error) = create_signal(cx, None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(cx, false);

    let register_action = create_action(
        cx,
        move |(username, password, färdtjänst_kort, färtjänst_lösenord): &(
            i32,
            String,
            i32,
            String,
        )| {
            log::debug!("Try to login with {username}");
            let password = password.to_string();
            let username = username.to_string();
            let färdtjänst_kort = färdtjänst_kort.to_owned();
            let färtjänst_lösenord = färtjänst_lösenord.to_string();
            let credentials = RegestringUser {
                card_nummer: färdtjänst_kort,
                name: username,
                password,
                färtjänst_password: färtjänst_lösenord,
            };
            let on_success = on_success.clone();
            async move {
                set_wait_for_response.update(|w| *w = true);
                let result = api.register(&credentials).await;
                set_wait_for_response.update(|w| *w = false);
                match result {
                    Ok(res) => {
                        set_login_error.update(|e| *e = None);
                        on_success(res);
                    }
                    Err(err) => {
                        let msg = match err {
                            api::Error::Fetch(js_err) => {
                                format!("{js_err:?}")
                            }
                            api::Error::Api(err) => err.message,
                        };
                        error!("Unable to login with {}: {msg}", credentials.name);
                        set_login_error.update(|e| *e = Some(msg));
                    }
                }
            }
        },
    );

    let disabled = Signal::derive(cx, move || wait_for_response.get());

    view! { cx,
      <RegisterCredentialsForm
        title = "Logga in till färdtjänsten"
        action_label = "Registrera"
        action = register_action
        error = register_error.into()
        disabled
      />
      <p>"Don't have an account?"</p>
      <A href=Page::Register.path()>"Register"</A>
    }
}
