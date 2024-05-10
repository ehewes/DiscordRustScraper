use std::env;
use tracing::{subscriber, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut tracing_max_level = Level::INFO;

    let debug_env_var_key = format!("{}_DEBUG", env!("CARGO_PKG_NAME").to_uppercase());

    if let Ok(debug_var) = env::var(debug_env_var_key) {
        if debug_var == "1" {
            tracing_max_level = Level::TRACE;
        }
    }

    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing_max_level)
        .finish();

    subscriber::set_global_default(subscriber)
        .expect("Failed to set the global default tracing subscriber");

    spldcrdchlscrp::run_cli().await?;

    Ok(())
}
