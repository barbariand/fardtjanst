
pub mod api;

use client::*;
use leptos::*;

fn main() {
    mount_to_body(|cx| view! { cx,  <Start/>})
}