use anyhow::Error;
use client::{CountRequest, CountResponse, Direction};
use gloo_console::log;
use gloo_net::http::Request;
use js_sys::Date;
use wasm_bindgen_futures::spawn_local;
use yew::{html, Callback, Component, Context, Html};
use yew_websocket::{
    macros::Json,
    websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
};

pub enum WsAction {
    SendData,
    Disconnect,
    Lost,
}

pub enum Msg {
    WsAction(WsAction),
    WsReady(Result<CountResponse, Error>),
}

impl From<WsAction> for Msg {
    fn from(action: WsAction) -> Self {
        Msg::WsAction(action)
    }
}

pub struct Counter {
    pub data: Option<CountResponse>,
    pub ws: Option<WebSocketTask>,
}

impl Counter {
    fn view_count(&self) -> Html {
        if let Some(value) = &self.data {
            html! {
                <p>{ format!("The counter value is: {}", value.count) }</p>
            }
        } else {
            html! {
                <p>{ "The count hasn't loaded yet." }</p>
            }
        }
    }
}

impl Component for Counter {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        log!("creating");
        log!("connecting");
        let callback = ctx.link().callback(|Json(data)| Msg::WsReady(data));
        let notification = ctx.link().batch_callback(|status| match status {
            WebSocketStatus::Opened => None,
            WebSocketStatus::Closed | WebSocketStatus::Error => Some(WsAction::Lost.into()),
        });
        let task =
            WebSocketService::connect("ws://127.0.0.1:8081/ws/count", callback, notification)
                .unwrap();
        log!("connected");
        Self {
            data: None,
            ws: Some(task),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::WsAction(action) => match action {
                WsAction::SendData => {
                    log!("sending data");
                    false
                }
                WsAction::Disconnect => {
                    self.ws.take();
                    log!("diconnecting");
                    true
                }
                WsAction::Lost => {
                    log!("lost");
                    self.ws = None;
                    true
                }
            },
            Msg::WsReady(response) => {
                log!("response received");
                match response {
                    Ok(wsr) => {
                        log!(format!("response was {:?}", wsr.count));
                        self.data = Some(wsr);
                        true
                    }
                    Err(e) => {
                        log!(format!("error was {:?}", e));
                        false
                    }
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let incr = Callback::from(move |_| {
            spawn_local(async move {
                post_count_update(&CountRequest {
                    direction: Direction::Increment,
                })
                .await;
            });
        });

        let decr = Callback::from(move |_| {
            spawn_local(async move {
                post_count_update(&CountRequest {
                    direction: Direction::Decrement,
                })
                .await;
            });
        });

        html! {
            <div>
                <nav class="menu">
                    { self.view_count() }
                </nav>
                <button onclick={ incr }> {"+"} </button>
                <button onclick={ decr }> {"-"} </button>

                // Display the current date and time the page was rendered
                <p class="footer">
                    { "Rendered: " }
                    { String::from(Date::new_0().to_string()) }
                </p>
            </div>
        }
    }
}

async fn post_count_update(count_request: &CountRequest) {
    log!("post called");
    let j = serde_json::to_string(count_request);
    match j {
        Ok(data) => {
            Request::post(format!("/api/count/{}", count_request.to_string()).as_str())
                .send()
                .await
                .map_err(|err| err.to_string());
            ()
        }

        Err(_) => (),
    }
}
