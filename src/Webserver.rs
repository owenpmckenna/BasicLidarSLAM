use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use axum::extract::{State, WebSocketUpgrade};
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use axum::response::IntoResponse;
use axum::{Error, Router};
use axum::routing::{any, get, post};
use tokio::net::TcpListener;
use crossbeam_channel::{unbounded, Receiver, Sender};
use futures_util::stream::{SplitSink, SplitStream};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use crate::Drivetrain::Drivetrain;

#[derive(Clone)]
struct AppState {
    rx: Arc<Receiver<SendData>>,
    dt: Arc<Mutex<Drivetrain>>
}
#[derive(Serialize, Deserialize)]
pub struct SendData {
    pub data: Vec<SmallData>
}
#[derive(Serialize, Deserialize)]
pub struct RecData {
    x: f64,
    y: f64,
    turn: f64
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
        let state = AppState {rx: Arc::new(r), dt: Arc::new(Mutex::new(Drivetrain::new()))};
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
        let (mut sender, mut reciever) = ws.split();
        tokio::spawn(Self::receive(state.clone(), reciever));
        println!("received connection!");
        tokio::spawn(Self::send(state.clone(), sender));
    }
    async fn receive(state: AppState, mut reciever: SplitStream<WebSocket>) {
        loop {
            match reciever.next().await {
                None => {return;}
                Some(it) => {
                    match it {
                        Ok(it) => {
                            match serde_json::from_str::<RecData>(it.to_text().unwrap()) {
                                Ok(it) => {
                                    let mut dt = state.dt.lock().unwrap();
                                    dt.x = it.x as f32;
                                    dt.y = it.y as f32;
                                    dt.turn = it.turn as f32;
                                    dt.power().unwrap()
                                }
                                Err(it) => {println!("error: {}", it); return;}
                            }
                        }
                        Err(it) => {println!("error: {}", it); return;}
                    }
                }
            }
        }
    }
    async fn send(state: AppState, mut sender: SplitSink<WebSocket, Message>) {
        for x in state.rx.iter() {
            println!("writing data! ({} points)", x.data.len());
            let data = serde_json::to_string(&x).unwrap().into();
            match sender.send(Message::Text(data)).await {
                Ok(_) => {}
                Err(it) => { println!("Error: {}", it); }
            };
        }
    }
}