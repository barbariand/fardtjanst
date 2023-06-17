use leptos::*;

#[component]
pub fn header(cx: Scope)->impl IntoView{
    view!{
        cx,
        <header>
            <p>"Notifikattion tjänst för Färdtjänsten"</p>
        </header>
    }

}