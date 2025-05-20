mod project {
    use axum::{
        Router,
        extract::{
            State, WebSocketUpgrade,
            ws::{Message, WebSocket},
        },
        http::{HeaderValue, StatusCode},
        response::IntoResponse,
        routing::get,
    };
    use futures_util::{SinkExt, StreamExt};
    use tokio::sync::broadcast::Sender;
    use tower_http::cors::{Any, CorsLayer};

    #[tokio::test]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn app(tx: Sender<String>) -> Router {
        let cors_layer = CorsLayer::new()
            .allow_methods(Any)
            .allow_origin("http://127.0.0.1:8080".parse::<HeaderValue>().unwrap());
        Router::new()
            .route("/", get(|| async { "Home" }))
            .route("/chat", get(chat_handler))
            .with_state(tx)
            .layer(cors_layer)
    }

    async fn chat_handler(
        State(tx): State<Sender<String>>,
        ws: WebSocketUpgrade,
    ) -> impl IntoResponse {
        ws.on_upgrade(|web_socket| handle_websocket(tx, web_socket));
        (StatusCode::OK, "Hello").into_response()
    }

    async fn handle_websocket(tx: Sender<String>, websocket: WebSocket) {
        let (mut sender, mut receiver) = websocket.split();

        let mut rx = tx.subscribe();
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                sender.send(Message::from(msg)).await.unwrap();
            }
        });
    }
}
