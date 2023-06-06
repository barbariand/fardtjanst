use leptos::*;
use wasm_bindgen::{prelude::wasm_bindgen, prelude::Closure};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[component]
pub fn start(cx: Scope) -> impl IntoView {
    
    view! { cx,
        <main>
        </main>
    }
}
