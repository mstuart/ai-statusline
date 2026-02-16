use std::io::{self, Read};
use std::process;

use clap::Parser;

mod cli;

use claudeline::config::Config;
use claudeline::layout::LayoutEngine;
use claudeline::render::Renderer;
use claudeline::widgets::{SessionData, WidgetRegistry};

#[derive(Parser)]
#[command(name = "claudeline", version, about = "A high-performance status line for Claude Code")]
struct Cli {
    #[command(subcommand)]
    command: Option<cli::Commands>,

    /// Path to config file
    #[arg(long)]
    config: Option<String>,

    /// Color level override: auto, none, 16, 256, truecolor
    #[arg(long, default_value = "auto")]
    color_level: String,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => cli::handle_command(cmd),
        None => render_statusline(&cli),
    }
}

fn render_statusline(cli: &Cli) {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        process::exit(1);
    }

    let data: SessionData = match serde_json::from_str(&input) {
        Ok(d) => d,
        Err(_) => process::exit(1),
    };

    let config = Config::load(cli.config.as_deref());
    let renderer = Renderer::detect(&cli.color_level);
    let registry = WidgetRegistry::new();
    let engine = LayoutEngine::new(&config, &renderer);

    let lines = engine.render(&data, &config, &registry);
    for line in &lines {
        println!("{line}");
    }
}
