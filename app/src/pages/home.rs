use crate::api::*;
use leptos::*;
use leptos_router::*;
use super::components::*;
#[component]
fn home(cx: Scope, user_info: Signal<Option<UserInfo>>) -> impl IntoView {
    view! {
        cx,
        <div></div>
    }
}
