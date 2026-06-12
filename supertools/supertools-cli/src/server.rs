use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    ReviewData { content: String, mode: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    Decision {
        approved: bool,
        feedback: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionResult {
    pub approved: bool,
    pub feedback: Option<String>,
}

#[derive(rust_embed::RustEmbed)]
#[folder = "static/"]
struct Assets;

#[derive(Clone)]
struct AppState {
    content: String,
    mode: String,
    decision_tx: Arc<Mutex<Option<oneshot::Sender<DecisionResult>>>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

async fn serve_index() -> impl IntoResponse {
    match Assets::get("index.html") {
        Some(file) => {
            let bytes = file.data.to_vec();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html")
                .body(axum::body::Body::from(bytes))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("Not Found"))
            .unwrap(),
    }
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let review_msg = ServerMessage::ReviewData {
        content: state.content.clone(),
        mode: state.mode.clone(),
    };
    let review_json = match serde_json::to_string(&review_msg) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Failed to serialize ReviewData: {}", e);
            return;
        }
    };

    if let Err(e) = socket.send(Message::Text(review_json.into())).await {
        eprintln!("Failed to send ReviewData: {}", e);
        return;
    }

    while let Some(msg_result) = socket.recv().await {
        let msg = match msg_result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error receiving websocket message: {}", e);
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                if let Ok(ClientMessage::Decision { approved, feedback }) =
                    serde_json::from_str::<ClientMessage>(text.as_str())
                {
                    if let Some(decision_tx) = state.decision_tx.lock().await.take() {
                        let _ = decision_tx.send(DecisionResult { approved, feedback });
                    }
                    if let Some(shutdown_tx) = state.shutdown_tx.lock().await.take() {
                        let _ = shutdown_tx.send(());
                    }
                    break;
                }
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Start the Axum web server on the given port (or 0 for random).
/// Returns a JoinHandle that completes when the decision is made, and the URL to access the server.
pub async fn start_server(
    port: u16,
    content: String,
    mode: String,
) -> Result<
    (tokio::task::JoinHandle<DecisionResult>, String),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let local_addr = listener.local_addr()?;
    let bound_port = local_addr.port();
    let url = format!("http://127.0.0.1:{}", bound_port);

    let (decision_tx, decision_rx) = oneshot::channel::<DecisionResult>();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let state = AppState {
        content,
        mode,
        decision_tx: Arc::new(Mutex::new(Some(decision_tx))),
        shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
    };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let handle = tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        });

        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }

        decision_rx.await.unwrap_or(DecisionResult {
            approved: false,
            feedback: None,
        })
    });

    Ok((handle, url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};

    #[tokio::test]
    async fn test_server_ws_decision() {
        let content = "Test implementation plan".to_string();
        let mode = "plan".to_string();

        // Start server on random port
        let (server_handle, url) = start_server(0, content.clone(), mode.clone())
            .await
            .expect("Failed to start server");

        // The stub URL is port 0, which cannot be connected to. This should fail to connect.
        let ws_url = url.replace("http://", "ws://") + "/ws";
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Failed to connect to WebSocket");

        // Read ServerMessage::ReviewData
        let msg = ws_stream
            .next()
            .await
            .expect("No message from server")
            .expect("Error reading message");

        let text = msg.to_text().expect("Message is not text");
        let server_msg: ServerMessage =
            serde_json::from_str(text).expect("Failed to parse ServerMessage");
        assert_eq!(
            server_msg,
            ServerMessage::ReviewData {
                content: content.clone(),
                mode: mode.clone()
            }
        );

        // Send decision
        let decision_msg = ClientMessage::Decision {
            approved: true,
            feedback: Some("All looks great!".to_string()),
        };
        let decision_json = serde_json::to_string(&decision_msg).unwrap();
        ws_stream
            .send(tokio_tungstenite::tungstenite::Message::Text(
                decision_json.into(),
            ))
            .await
            .expect("Failed to send decision");

        // Await server decision result
        let result = server_handle.await.expect("Server task failed");
        assert_eq!(
            result,
            DecisionResult {
                approved: true,
                feedback: Some("All looks great!".to_string())
            }
        );
    }
}
