use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use comrak::Options;
use regex::RegexBuilder;
use tera::Tera;

/// ⚡ Simple static site generator.
#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

/// Common arguments.
#[derive(Parser)]
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

fn main() -> Result<()> {
    let args = Args::parse();

    match args.cmd {
        Cmd::Build(args) => build(args),
        Cmd::Serve(args) => serve(args),
    }
}

fn build(args: CommonArgs) -> Result<()> {
    let markdown = RegexBuilder::new("---\n+(.*)\n+---\n+(.*)").dot_matches_new_line(true).build().unwrap();

    std::env::set_current_dir(&args.path.join("content"))?;
    let out: PathBuf = "../out".into();
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out)?;

    let mut tera = Tera::new("../template/**/*")?;
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
                let meta = captures.get(1).unwrap();
                let content = captures.get(2).unwrap().as_str();
                let content = comrak::markdown_to_html(&content, &Options::default());
                let content = format!(r#"{{% extends "post.html" %}}
{{% block content %}}
{content}
{{% endblock content %}}
"#);

                let context = tera::Context::new();
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

fn serve(args: CommonArgs) -> Result<()> {
    build(args)
}
