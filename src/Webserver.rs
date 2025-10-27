use std::sync::Arc;
use axum::extract::{State, WebSocketUpgrade};
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use axum::response::IntoResponse;
use axum::{Error, Router};
use axum::routing::{any, get, post};
use tokio::net::TcpListener;
use crossbeam_channel::{unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    rx: Arc<Receiver<SendData>>
}
#[derive(Serialize, Deserialize)]
pub struct SendData {
    pub data: Vec<SmallData>
}
#[derive(Serialize, Deserialize)]
pub struct SmallData {
    pub x: i32,
    pub y : i32
}
pub struct Webserver {
    router: Router,
    tcp_listener: TcpListener
}
impl Webserver {
    pub async fn new(r: Receiver<SendData>) -> Webserver {
        let state = AppState {rx: Arc::new(r)};
        // build our application with a route
        let cors = CorsLayer::new().allow_origin(Any);
        let app = Router::new()
            .route("/", any(Webserver::root))
            .route("/data", any(Webserver::data))
            .with_state(state)
            .layer(cors);
        let listener = TcpListener::bind("0.0.0.0:8081").await.unwrap();
        Webserver {router: app, tcp_listener: listener}
    }
    pub async fn serve(self) {
        axum::serve(self.tcp_listener, self.router.into_make_service()).await.expect("Axum failed");
    }
    async fn root() -> &'static str {
        "hello, world"
    }
    async fn data(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
        ws.on_upgrade(move |socket| Webserver::handle_socket(state, socket))
    }
    async fn handle_socket(state: AppState, mut ws: WebSocket) {
        println!("received connection!");
        for x in state.rx.iter() {
            println!("writing data! ({} points)", x.data.len());
            let data = serde_json::to_string(&x).unwrap();
            match ws.send(Message::Text(Utf8Bytes::from(data))).await {
                Ok(_) => {}
                Err(it) => {println!("Error: {}", it);}
            };
        }
    }
}