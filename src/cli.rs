use clap::{Args, Parser, Subcommand};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Inline(Config),
    File { path: String },
}

#[derive(Args, Debug, Deserialize, Clone)]
pub struct Config {
    #[arg(long)]
    pub dotenv: Option<String>,
    #[arg(long)]
    pub database_url: Option<String>,
    #[arg(short, long)]
    pub output: String,
    #[serde(default = "default_template_directory")]
    #[arg(long, default_value_t = String::from("templates"))]
    pub template_directory: String,
    #[arg(short, long)]
    pub templates: Vec<String>,
    #[serde(default = "bool_true")]
    #[arg(short, long, default_value_t = true)]
    pub render_folder: bool,
}

#[inline]
fn bool_true() -> bool {
    true
}
#[inline]
fn default_template_directory() -> String {
    "templates".to_string()
}
