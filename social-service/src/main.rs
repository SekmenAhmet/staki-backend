mod config;
use config::Config;
mod handlers;
mod models;
mod services;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::from_env();
}
