use leptos::*;
use stylist::Style;
use macros;
use super::navbar::*;
#[macros::enhance_with_style]
#[component]
pub fn header<F>(cx: Scope,
    logged_in: Signal<bool>,
    on_logout: F,)->impl IntoView
    where
    F: Fn() + 'static + Clone,{
    

    styled_macro::view! {
        cx,
        styles=styles,
        <header>
            <p>"Notifikattion tjänst för Färdtjänsten"</p>
        </header>
        <NavBar logged_in on_logout/>
    }

}