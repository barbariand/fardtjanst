use crate::api::AuthorizedApi;
use leptos::prelude::*;
use leptos::*;
#[component]
pub fn Home(cx: Scope, api: RwSignal<Option<AuthorizedApi>>) -> impl IntoView {
    let trips = create_resource(
        cx,
        || (),
        move |_| async move { api.get().unwrap().trips().await },
    );
    trips.refetch();
    view! { cx,
      <Transition
            fallback=move || view! { cx, <p>"Loading..."</p> }
        >
        <ErrorBoundary
        // the fallback receives a signal containing current errors
        fallback=|cx, errors| view! { cx,}
        >
        {

        }
        </ErrorBoundary>
        </Transition>

    }
}
