mod project {
    use axum::{
        Router,
        extract::{
            State, WebSocketUpgrade,
            ws::{Message, WebSocket},
        },
        http::HeaderValue,
        response::IntoResponse,
        routing::get,
    };
    use futures_util::{SinkExt, StreamExt};
    use tokio::sync::broadcast::{Sender, channel};
    use tower_http::cors::{Any, CorsLayer};

    #[tokio::test]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let (tx, _) = channel(100);
        let app = app(tx);
        let web_listener = tokio::net::TcpListener::bind("127.0.0.1:8081").await?;

        // axum::serve(web_listener, app).await?;

        tokio::spawn(async move {
            axum::serve(web_listener, app).await.unwrap();
        });

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(())
    }

    fn app(tx: Sender<String>) -> Router {
        let cors_layer = CorsLayer::new()
            .allow_methods(Any)
            .allow_origin("http://127.0.0.1:12345".parse::<HeaderValue>().unwrap());
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
        ws.on_upgrade(|web_socket| handle_websocket(tx, web_socket))
    }

    async fn handle_websocket(tx: Sender<String>, websocket: WebSocket) {
        let (mut sender, mut receiver) = websocket.split();

        let mut rx = tx.subscribe();
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                sender.send(Message::from(msg)).await.unwrap();
            }
        });

        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(content) => {
                        tx.send(content.to_string()).unwrap();
                    }
                    _ => (),
                }
            }
        }
    }
}
