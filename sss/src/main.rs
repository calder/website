use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use sss::{Builder, Server};

/// ⚡ Simple static site generator.
#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

/// Build args.
#[derive(Clone, Parser)]
struct BuildArgs {
    /// Path containing content and output directories.
    #[arg(default_value = ".")]
    path: PathBuf,
}

/// Serve args.
#[derive(Clone, Parser)]
struct ServeArgs {
    /// Build args.
    #[command(flatten)]
    build: BuildArgs,

    /// Address to listen on.
    #[arg(long, default_value = "localhost:8080")]
    addr: String,
}

/// Command.
#[derive(Subcommand)]
enum Cmd {
    /// Build site.
    Build(BuildArgs),

    /// Serve site.
    Serve(ServeArgs),
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = match &args.cmd {
        Cmd::Build(args) => &args.path,
        Cmd::Serve(args) => &args.build.path,
    };
    std::env::set_current_dir(path)?;

    match args.cmd {
        Cmd::Build(_) => build(),
        Cmd::Serve(args) => serve(args.addr),
    }
}

fn build() -> Result<()> {
    Builder::build()
}

fn serve(addr: String) -> ! {
    Server::start(addr);
    Builder::watch();
}
