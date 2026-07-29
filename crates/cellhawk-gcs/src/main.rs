mod api;
mod danger_grid;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, instrument};
use tracing_subscriber;

use cellhawk_core::types::TelemetrySnapshot;
use danger_grid::DangerGrid;

pub struct AppState {
    pub danger_grid: Arc<DangerGrid>,
    pub tx: broadcast::Sender<String>,
}

#[tokio::main]
#[instrument]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting CELLHAWK GCS Server (Enterprise Fleet Management Scale)");

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/0".to_string());

    let danger_grid = match DangerGrid::new(&redis_url) {
        Ok(dg) => Arc::new(dg),
        Err(e) => {
            error!("Failed to connect to Redis: {}", e);
            std::process::exit(1);
        }
    };

    let (tx, _rx) = broadcast::channel(100);

    let app_state = Arc::new(AppState { danger_grid, tx });

    let app = Router::new()
        .nest("/api/v2", api::fleet_router(app_state.clone()))
        .route("/api/v1/health", get(|| async { "OK" }))
        .route("/ws/telemetry", get(ws_handler))
        .route("/api/v1/threats", get(get_threats))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    // In production, we loop and forward telemetry broadcasts to the connected websocket
    // We also listen for incoming commands (ping/pong, manual overrides)

    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Ok(text) = msg {
                    if socket.send(Message::Text(text)).await.is_err() {
                        break;
                    }
                }
            }
            msg = socket.recv() => {
                if let Some(Ok(Message::Text(text))) = msg {
                    if text == "ping" {
                        let _ = socket.send(Message::Text("pong".to_string())).await;
                    }
                } else {
                    break;
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct ThreatQuery {
    lon: f64,
    lat: f64,
    radius_m: f64,
}

async fn get_threats(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<ThreatQuery>,
) -> Json<serde_json::Value> {
    match state
        .danger_grid
        .get_threats_in_radius(query.lon, query.lat, query.radius_m)
        .await
    {
        Ok(threats) => Json(serde_json::json!({
            "status": "success",
            "count": threats.len(),
            "threats": threats
        })),
        Err(e) => {
            error!("Failed to fetch threats: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "message": e.to_string()
            }))
        }
    }
}
