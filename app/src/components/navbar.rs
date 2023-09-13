use leptos::*;
use stylist::Style;

use crate::Page;
use macros;
#[macros::enhance_with_style]
#[component]
pub fn NavBar<F>(
    cx: Scope,
    logged_in: Signal<bool>,
    on_logout: F,
) -> impl IntoView
where
    F: Fn() + 'static + Clone,
{
  
  styled_macro::view! { cx,
    styles=styles,
    <style>
            "nav {
              color: white;
          }
          "
            </style>
      <nav>
      
        <Show
          when = move || logged_in.get()
          fallback = |cx| view! { cx,
            <a href=Page::Login.path()>"Login"</a>
            " | "
            <a href=Page::Register.path()>"Register"</a>
          }
        >
          <a style="color: white;" href="javascript:;" on:click={
            let on_logout = on_logout.clone();
            move |_| on_logout()
          }>"Logout"</a>
        </Show>
      </nav>
    }
}