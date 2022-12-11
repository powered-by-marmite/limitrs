use client::{CountRequest, CountResponse, Direction};
use gloo_console::log;
use gloo_net::http::Request;
use js_sys::Date;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use yew_router::prelude::*;

async fn get_count_value() -> i32 {
    log!("get called");
    let resp = Request::get("/api/count").send().await.unwrap();
    let result = {
        if !resp.ok() {
            Err(format!(
                "Error fetching data {} ({})",
                resp.status(),
                resp.status_text()
            ))
        } else {
            resp.text().await.map_err(|err| err.to_string())
        }
    }
    .unwrap();

    let count_resp: CountResponse = match serde_json::from_str(&result) {
        Ok(r) => r,
        Err(_) => return 0,
    };

    log!(format!("count was got as {}", count_resp.count));

    count_resp.count
}

async fn post_count_update(count_request: &CountRequest) {
    log!("post called");
    let j = serde_json::to_string(count_request);
    match j {
        Ok(data) => {
            Request::post("/api/count")
                .header("Content-Type", "application/json")
                .body(data)
                .send()
                .await
                .map_err(|err| err.to_string());
            ()
        }

        Err(_) => (),
    }
}

#[function_component(Counter)]
fn counter() -> Html {
    let state = use_state(|| 0);

    // Set up initial state
    {
        let state = state.clone();
        use_effect_with_deps(
            move |_| {
                let state = state.clone();
                spawn_local(async move {
                    let get_result = get_count_value().await;
                    state.set(get_result);
                });

                || ()
            },
            (),
        );
    }

    let incr_counter = {
        Callback::from(move |_| {
            spawn_local(async move {
                post_count_update(&CountRequest {
                    direction: Direction::Increment,
                })
                .await;
            });
        })
    };

    let decr_counter = {
        Callback::from(move |_| {
            spawn_local(async move {
                post_count_update(&CountRequest {
                    direction: Direction::Decrement,
                })
                .await;
            });
        })
    };

    html! {
        <div>
                // Display the current value of the counter
                <p class="counter">
                    { "The counter value is: "} { (*state).clone() }
                </p>

                <button onclick={incr_counter}> {"+"} </button>
                <button onclick={decr_counter}> {"-"} </button>

                // Display the current date and time the page was rendered
                <p class="footer">
                    { "Rendered: " }
                    { String::from(Date::new_0().to_string()) }
                </p>
        </div>
    }
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/counter")]
    Counter,
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! { <h1>{ "Hello Frontend" }</h1> },
        Route::Counter => html! { <Counter />},
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
}
