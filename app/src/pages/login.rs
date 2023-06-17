use leptos::*;
use leptos_router::*;
use crate::api;
use super::components::*;
#[component]
pub fn Login<F>(cx: Scope, api: api::UnauthorizedApi, on_success: F) -> impl IntoView
where
    F: Fn(api::AuthorizedApi) + 'static + Clone,
{
    let styles = styled::style!(
        p{
            background-color:green;
        }
    );
    let (login_error, set_login_error) = create_signal(cx, None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(cx, false);
    let login_action = create_action(cx, move |(username, password): &(usize, String)| {
            let username=*username;
            let password = password.to_string();
            let credentials = api::Credentials { username, password };
            let on_success = on_success.clone();
            async move {
                set_wait_for_response.update(|w| *w = true);
                let result = api.login(&credentials).await;
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
                        error!(
                            "Unable to login with {}: {msg}",
                            credentials.password
                        );
                        set_login_error.update(|e| *e = Some(msg));
                    }
                }
            }
    });
    styled_macro::view! {
        cx,
        styles=styles,
        <div>"hello"</div>
    }
}
