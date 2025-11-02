use std::path::PathBuf;

use anyhow::{Context, Result};
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use clap::{Parser, Subcommand};
use notify::Watcher;
use regex::RegexBuilder;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

/// ⚡ Simple static site generator.
#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

/// Common arguments.
#[derive(Clone, Parser)]
struct CommonArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
}

/// Command.
#[derive(Subcommand)]
enum Cmd {
    /// Build site.
    Build(CommonArgs),

    /// Serve site.
    Serve(CommonArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.cmd {
        Cmd::Build(args) => build(args),
        Cmd::Serve(args) => serve(args).await,
    }
}

fn build(args: CommonArgs) -> Result<()> {
    let markdown = RegexBuilder::new("---\n+(.*)\n+---\n+(.*)")
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    std::env::set_current_dir(&args.path.join("content"))?;
    let out: PathBuf = "../out".into();
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out)?;

    let mut tera = tera::Tera::new("../template/**/*")?;
    for src in glob::glob("**/*")? {
        let src = src?;
        let dst = out.join(&src);
        let dst = match dst
            .extension()
            .map(|e| e.to_str().unwrap_or(""))
            .unwrap_or("")
        {
            "md" => dst.with_extension("html"),
            _ => dst,
        };

        match src
            .extension()
            .map(|e| e.to_str().unwrap_or(""))
            .unwrap_or("")
        {
            "md" => {
                let src = std::fs::read_to_string(src)?;
                let captures = markdown.captures(&src).context("Missing metadata")?;
                let meta = captures.get(1).unwrap().as_str();
                let meta: tera::Map<String, tera::Value> = serde_yaml_ng::from_str(meta)?;
                let typ = meta["type"].as_str().expect("Type must be a string");
                let content = captures.get(2).unwrap().as_str();
                let content = comrak::markdown_to_html(&content, &comrak::Options::default());
                let content = format!(
                    "{{% extends \"{}.html\" %}}\n{{% block content %}}\n{}\n{{% endblock content %}}",
                    typ, content
                );
                let context = tera::Context::from_serialize(meta)?;
                let content = tera.render_str(&content, &context)?;
                std::fs::write(dst, content)?;
            }
            "html" => {
                let src = std::fs::read_to_string(src)?;
                let context = tera::Context::new();
                let content = tera.render_str(&src, &context)?;
                std::fs::write(dst, content)?;
            }
            _ => {
                std::fs::copy(&src, &dst)?;
            }
        }
    }

    Ok(())
}

async fn serve(args: CommonArgs) -> Result<()> {
    // Start static web server.
    std::thread::spawn(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let app = Router::new()
                    .route("/{*path}", get(serve_html))
                    .fallback_service(ServeDir::new("../out"));
                let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
                axum::serve(listener, app).await.unwrap();
            });
    });

    // Rebuild whenever content changes.
    build(args.clone()).unwrap();
    let content = args.path.join("content");
    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            let res = res.unwrap();
            match res.kind {
                notify::EventKind::Access(_) => {}
                _ => build(args.clone()).unwrap(),
            }
        })?;
    watcher.watch(&content, notify::RecursiveMode::Recursive)?;
    loop {
        std::thread::park();
    }
}

async fn serve_html(Path(path): Path<String>) -> impl IntoResponse {
    let mut path = PathBuf::from("../out").join(path);
    if path.extension().is_none() {
        path.set_extension("html");
    }

    match std::fs::read(path) {
        Ok(content) => Html(content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}
