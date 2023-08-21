use leptos::*;

#[component]
pub fn header(cx: Scope)->impl IntoView{
    let styles=Style::new(style_macros::css_file!("credentials.css")); // injects the file durign compilation time :TADA:

    styled_macro::view! {
        cx,
        styles=styles,
        <header>
            <p>"Notifikattion tjänst för Färdtjänsten"</p>
        </header>
    }

}