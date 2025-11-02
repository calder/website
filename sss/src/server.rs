use std::path::Path;

use anyhow::Result;
use axum::Router;
use axum::http::{StatusCode, Uri};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use yansi::Paint;

/// Static site server.
pub struct Server {}

impl Server {
    pub fn start(addr: String) {
        eprintln!(
            "{} Serving on {}",
            "LIVE:".green().bright().bold(),
            format!("http://{addr}").cyan().bold()
        );

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
        let app = Router::new().fallback_service(ServeDir::new(out).fallback(get(
            async move |uri: Uri| {
                let path = out
                    .join(uri.path().trim_start_matches('/'))
                    .with_extension("html");
                match std::fs::read(path) {
                    Ok(content) => Html(content).into_response(),
                    Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
                }
            },
        )));
        let listener = TcpListener::bind(addr).await.unwrap();

        axum::serve(listener, app).await?;

        Ok(())
    }
}
