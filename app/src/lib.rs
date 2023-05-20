use leptos::{leptos_dom::console_log, *};
use wasm_bindgen::{prelude::wasm_bindgen, prelude::Closure};
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cat {
    url: String,
}
#[wasm_bindgen(module = "/js/notifications.js")]
extern "C" {
	// fn closure_test(f: &mut Closure<dyn FnMut()>);
	fn askForNotifications(f: &str);
}

#[derive(Error, Clone, Debug)]
pub enum FetchError {
    #[error("Please request more than zero cats.")]
    NonZeroCats,    
    #[error("Error loading data from serving.")]
    Request,
    #[error("Error deserializaing cat data from request.")]
    Json,
}

async fn fetch_cats(count: u32) -> Result<Vec<String>, FetchError> {
    console_log("trying to print to js");
    askForNotifications("HELLO JS FROM WASAM");
    console_log("tried to print to js");
    console_log(format!("want {}", count).as_str());
    if count > 0 {
        // make the request
        let urlstr = &format!("https://api.thecatapi.com/v1/images/search?limit={count}",);
        console_log(format!("Url is {}", urlstr).as_str());
        let mut res = reqwasm::http::Request::get(urlstr)
            .send()
            .await
            .map_err(|_| FetchError::Request)?
            // convert it to JSON
            .json::<Vec<Cat>>()
            .await
            .map_err(|_| FetchError::Json)?
            // extract the URL field for each cat
            .into_iter()
            .map(|cat| cat.url)
            .collect::<Vec<_>>();
        console_log(format!("got {} amount of cats", res.len()).as_str());
        if res.len() > count as usize {
            for _ in 0..(res.len() - count as usize) {
                res.remove(0);
            }
        
        }
        Ok(res)
    } else {
        Err(FetchError::NonZeroCats)
    }
}
#[component]
pub fn start(cx: Scope) -> impl IntoView {
    let (cat_count, set_cat_count) = create_signal::<u32>(cx, 0);

    // we use local_resource here because
    // 1) our error type isn't serializable/deserializable
    // 2) we're not doing server-side rendering in this example anyway
    //    (during SSR, create_resource will begin loading on the server and resolve on the client)
    let cats = create_local_resource(cx, cat_count, fetch_cats);
    
    let fallback = move |cx, errors: RwSignal<Errors>| {
        let error_list = move || {
            errors.with(|errors| {
                errors
                    .iter()
                    .map(|(_, e)| view! { cx, <li>{e.to_string()}</li> })
                    .collect::<Vec<_>>()
            })
        };
        
        view! { cx,
            <div class="error">
                <h2>"Error"</h2>
                <ul>{error_list}</ul>
            </div>
        }
    };

    // the renderer can handle Option<_> and Result<_> states
    // by displaying nothing for None if the resource is still loading
    // and by using the ErrorBoundary fallback to catch Err(_)
    // so we'll just implement our happy path and let the framework handle the rest
    let cats_view = move || {
        cats.read(cx).map(|data| {
            data.map(|data| {
                data.iter()
                    .map(|s| view! { cx, <img width=100 height=100 src=s></img>})
                    .collect::<Vec<_>>()
            })
        })
    };
    view! { cx,
        <div>
        <script src="">
        </script>
            <label>
                "How many cats would you like?"
                <input
                    type="number"
                    prop:value=move || cat_count.get().to_string()
                    on:input=move |ev| {
                        let val = event_target_value(&ev).parse::<u32>().unwrap_or(0);
                        set_cat_count(val);
                    }
                />
            </label>
            <ErrorBoundary fallback>
                <Transition fallback=move || {
                    view! { cx, <div>"Loading (Suspense Fallback)..."</div> }
                }>
                    {cats_view}
                </Transition>
            </ErrorBoundary>
        </div>
    }
}
#[component]
pub fn header(cx: Scope)->impl IntoView{
    view!{
        cx,
        <header>
            <p>"notifikattions tjänst för Färdtjänsten"</p>
        </header>
    }

}