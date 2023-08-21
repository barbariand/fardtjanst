
pub mod api;

use client::*;
use leptos::*;
use leptos_router::AnimatedRoutes;
fn main() {
    mount_to_body(|cx| view! { cx,  <Start/>})
}