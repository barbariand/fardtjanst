use leptos::{ev, *};

#[component]
pub fn CredentialsForm(
    cx: Scope,
    title: &'static str,
    action_label: &'static str,
    action: Action<(usize, String), ()>,
    error: Signal<Option<String>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let (password, set_password) = create_signal(cx, String::new());
    let (email, set_email) = create_signal(cx, 0);

    let dispatch_action = move || action.dispatch((email.get(), password.get()));

    let button_is_disabled =
        Signal::derive(cx, move || disabled.get() || password.get().is_empty());

    view! { cx,
        <form on:submit=|ev| ev.prevent_default()>
            <p>{title}</p>
            {move || {
                error
                    .get()
                    .map(|err| {
                        view! { cx, <p style="color:red;">{err}</p> }
                    })
            }}
            <input
                type="number"
                required
                placeholder="username"
                prop:disabled=move || disabled.get()
                on:keyup=move |ev: ev::KeyboardEvent| {
                    let val = event_target_value(&ev);
                    if let Ok(value)= val.parse::<usize>(){
                        set_email.update(|v| *v = value);
                        
                    }
                }
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    if let Ok(value)= val.parse::<usize>(){
                        set_email.update(|v| *v = value);
                    }
                }
            />
            <input
                type="password"
                required
                placeholder="Password"
                prop:disabled=move || disabled.get()
                on:keyup=move |ev: ev::KeyboardEvent| {
                    match &*ev.key() {
                        "Enter" => {
                            dispatch_action();
                        }
                        _ => {
                            let val = event_target_value(&ev);
                            set_password.update(|p| *p = val);
                        }
                    }
                }
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    set_password.update(|p| *p = val);
                }
            />
            <button
                prop:disabled=move || button_is_disabled.get()
                on:click=move |_| dispatch_action()
            >
                {action_label}
            </button>
        </form>
    }
}
