use std::path::Path;

use anyhow::Result;
use axum::Router;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

/// Static site server.
pub struct Server {}

impl Server {
    pub fn start(addr: String) {
        std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    Self::serve(&addr).await.unwrap();
                });
        });
    }

    pub async fn serve(addr: &str) -> Result<()> {
        let out = Path::new("out");
        let app = Router::new()
            .route(
                "/{*path}",
                get(
                    async move |axum::extract::Path(path): axum::extract::Path<String>| {
                        let mut path = out.join(path);
                        if path.extension().is_none() {
                            path.set_extension("html");
                        }

                        match std::fs::read(path) {
                            Ok(content) => Html(content).into_response(),
                            Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
                        }
                    },
                ),
            )
            .fallback_service(ServeDir::new(out));
        let listener = TcpListener::bind(addr).await.unwrap();

        axum::serve(listener, app).await?;

        Ok(())
    }
}
