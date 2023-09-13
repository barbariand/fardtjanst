use leptos::*;
use stylist::Style;
use style_macros;
use macros::enhance_with_style;
#[enhance_with_style]// injects the file witch is the rust file name + css so smth.rs -> smth.css durign compilation time then puts it to styles:TADA:
#[component]
pub fn LoginCredentials(
    cx: Scope,
    title: &'static str,
    action_label: &'static str,
    action: Action<(i32, String), ()>,
    error: Signal<Option<String>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let (password, set_password) = create_signal(cx, String::new());
    let (email, set_username) = create_signal(cx, 0);
    let dispatch_action = move || action.dispatch((email.get(), password.get()));
    let button_is_disabled =
        Signal::derive(cx, move || disabled.get() || password.get().is_empty());
    styled_macro::view! { cx,
    styles=styles,
    <style>r#"::placeholder {
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
      #login::after{
        content:"";
        position: absolute;
        z-index: -1;
        inset: 0;
        background-color: #351d4a;
        color:#ab79d6;
        scale: 0.2 1;
        border-radius: 8px;
        border-style: none;
        box-sizing: border-box;
        cursor: pointer;
        display: inline-block;
        padding:0.4rem calc(0.4rem - 1px);
        font-family: "Haas Grot Text R Web", "Helvetica Neue", Helvetica, Arial,
    sans-serif;
  font-size: 14px;
  font-weight: 500;
  height: 38px;
  line-height: 20px;
  list-style: none;
  margin: 0;
  outline: none;
  text-align: center;
  text-decoration: none;
  transition: color 100ms;
  vertical-align: baseline;
  user-select: none;
  -webkit-user-select: none;
  touch-action: manipulation;
  text-decoration: none;

    isolation: isolate;
      }
      "#</style>
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
                    set_username.update(|v| *v = val.parse::<i32>().unwrap());
                }
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    set_username.update(|v| *v = val.parse::<i32>().unwrap());
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
            
            id="login"
                prop:disabled=move || button_is_disabled.get()
                on:click=move |_| dispatch_action()
            >
            {action_label}
            </button>
            
            
        </form>
        </div>
    }
}
