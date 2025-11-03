use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use glob::glob;
use notify::Watcher;
use regex::{Regex, RegexBuilder};
use yansi::Paint;

/// Static site generator.
pub struct Builder {}

impl Builder {
    pub fn build() -> Result<()> {
        eprintln!("{} Building site", "INFO:".green().bold());
        let start = Instant::now();

        let out = Path::new("out");
        let _ = std::fs::remove_dir_all(out);
        std::fs::create_dir_all(out)?;

        let mut tera = tera::Tera::new("template/**/*")?;
        for src in glob("content/*")? {
            let src = src?;
            let dst = out.join(src.components().skip(1).collect::<PathBuf>());
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
                    let captures = MARKDOWN.captures(&src).context("Missing metadata")?;
                    let meta = captures.get(1).unwrap().as_str();
                    let meta: tera::Map<String, tera::Value> = serde_yaml_ng::from_str(meta)?;
                    let typ = meta["type"].as_str().expect("Type must be a string");
                    let content = captures.get(2).unwrap().as_str();
                    let mut options = comrak::Options::default();
                    options.extension.footnotes = true;
                    let content = comrak::markdown_to_html(content, &options);
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
        eprintln!(
            "{} Built site in {}",
            "INFO:".green().bold(),
            format!("{:0.03}s", start.elapsed().as_secs_f64()).cyan()
        );

        Ok(())
    }

    pub fn watch() -> ! {
        Builder::build().expect("Error building site");

        let mut watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                let res = res.expect("Error getting filesystem watch event");
                match res.kind {
                    notify::EventKind::Access(_) => {}
                    _ => Builder::build().expect("Error building site"),
                }
            })
            .expect("Error creating filesystem watcher");
        watcher
            .watch(Path::new("content"), notify::RecursiveMode::Recursive)
            .expect("Error watching content/");
        watcher
            .watch(Path::new("template"), notify::RecursiveMode::Recursive)
            .expect("Error watching templates/");

        loop {
            std::thread::park();
        }
    }
}

#[static_init::dynamic]
static MARKDOWN: Regex = RegexBuilder::new("---\n+(.*)\n+---\n+(.*)")
    .dot_matches_new_line(true)
    .build()
    .unwrap();
