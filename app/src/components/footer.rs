use leptos::*;
use leptos_dom::IntoView;
#[component]
pub fn Footer(cx: Scope) -> impl IntoView {
    
    let styles = styled::style!(
        footer {
            height:10%;
        }
        
    );

    styled_macro::view! {
        cx,
        styles=styles,
        <footer>
            "Made by Dante Nilsson, contact on mail dante.a.nilsson+fardtjanstnotifikation@gmail.com"
        </footer>
    }
}
