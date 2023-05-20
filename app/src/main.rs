use app::*;
use leptos::view;
fn main() {
    use leptos::mount_to_body;
    mount_to_body(|cx| view! { cx,  <Start /> })
}