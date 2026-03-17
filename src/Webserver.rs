use crate::Drivetrain::Drivetrain;
use crate::LidarLocalizer::Line;
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::{any, post};
use axum::{Json, Router};
use crossbeam_channel::Receiver;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    rx: Arc<Receiver<SendData>>,
    dt: Arc<Mutex<Drivetrain>>
}
#[derive(Serialize, Deserialize)]
pub struct SendData {
    pub data: Vec<SmallData>,
    pub lines: Vec<Line>,
    pub full_lines: Vec<Line>,
    pub x: f32,
    pub y: f32,
    pub heading: f32,
}
#[derive(Serialize, Deserialize)]
pub struct RecData {
    x: f64,
    y: f64,
    turn: f64
}
#[derive(Serialize, Deserialize)]
pub struct SmallData {
    pub x: f32,
    pub y : f32
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
            .route("/script.js", any(Webserver::script))
            .route("/data", any(Webserver::data))
            .route("/motorcontrol", post(Webserver::motorcontrol))
            .with_state(state)
            .layer(cors);
        let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        Webserver {router: app, tcp_listener: listener}
    }
    pub async fn serve(self) {
        axum::serve(self.tcp_listener, self.router.into_make_service()).await.expect("Axum failed");
    }
    async fn motorcontrol(State(state): State<AppState>, Json(rd): Json<RecData>) -> Result<(HeaderMap, Bytes), (StatusCode, String)> {
        println!("got message");
        {
            let mut dt = state.dt.lock().unwrap();
            dt.x = rd.x as f32;
            dt.y = rd.y as f32;
            dt.turn = rd.turn as f32;
            dt.power().unwrap();
        }
        Ok((HeaderMap::new(), Bytes::from("{}")))
    }
    async fn root() -> impl IntoResponse {
        Html(include_str!("../html/index.html"))
    }
    async fn script() -> impl IntoResponse {
        Html(include_str!("../html/script.js"))
    }
    async fn data(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
        ws.on_upgrade(move |socket| Webserver::handle_socket(state, socket))
    }
    async fn handle_socket(state: AppState, ws: WebSocket) {
        let (sender, receiver) = ws.split();
        tokio::spawn(Self::receive(state.clone(), receiver));
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
                                    dt.power().unwrap();
                                    println!("got message");
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
            //println!("writing data! ({} points)", x.data.len());
            let data = serde_json::to_string(&x).unwrap().into();
            match sender.send(Message::Text(data)).await {
                Ok(_) => {}
                Err(it) => { println!("Error: {}", it); }
            };
        }
    }
}