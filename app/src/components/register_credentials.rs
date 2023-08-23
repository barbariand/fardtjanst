use leptos::*;
use stylist::Style;
use style_macros;
use macros::enhance_with_style;
#[enhance_with_style]// injects the file witch is the rust file name + css so smth.rs -> smth.css durign compilation time then puts it to styles:TADA:
#[component]
pub fn CredentialsForm(
    cx: Scope,
    title: &'static str,
    action_label: &'static str,
    action: Action<(String, String,i32,String), ()>,
    error: Signal<Option<String>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    Result
    let (password, set_password) = create_signal(cx, String::new());
    let (email, set_username) = create_signal(cx, String::new());
    let (färtjänst_password, set_färtjänst_password) = create_signal(cx, String::new());
    let (card_number, set_card_number) = create_signal(cx, 0);
    let dispatch_action = move || action.dispatch((email.get(), password.get(),card_number.get(),färtjänst_password.get()));
    let button_is_disabled =
        Signal::derive(cx, move || disabled.get() || password.get().is_empty() || card_number.get().lt(&100_000_000));
    styled_macro::view! { cx,
    styles=styles,
    <style>"::placeholder {
        /* Chrome, Firefox, Opera, Safari 10.1+ */
        color: white;
        opacity: 1; /* Firefox */
      }
      
      :-ms-input-placeholder {
        /* Internet Explorer 10-11 */
        color: white;
      }
      
      ::-ms-input-placeholder {
        /* Microsoft Edge */
        color: white;
      }
      "</style>
      <div class="outer-form">
      
        <form
         on:submit=|ev| ev.prevent_default()>
         
      
            <h2>{title}</h2>
            {move || {
                error
                    .get()
                    .map(|err| {
                        view! { cx, <p style="color:red;">{err}</p> }
                    })
            }}
            <div class="form-inline">
            <div class="input-group">
            <label for="username">"Användarnamn"</label>
            <input
                type="text"
                required
                name="username"
                prop:disabled=move || disabled.get()
                on:keyup=move |ev: ev::KeyboardEvent| {
                    let val = event_target_value(&ev);
                    set_username.update(|v| *v = val);
                }
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    set_username.update(|v| *v = val);
                }
            />
            </div>
            </div>
            <div class="form-inline">
            <div class="input-group">
            <label for="password">"Lösenord"</label>
            <input
                type="password"
                required
                name="password"
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
            </div>
            </div>
            <button
                prop:disabled=move || button_is_disabled.get()
                on:click=move |_| dispatch_action()
            >
                {action_label}
            </button>
            
            
        </form>
        </div>
    }
}
