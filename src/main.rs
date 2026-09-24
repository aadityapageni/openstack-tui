use anyhow::Result;
use clap::Parser;

mod auth;
mod cli;
mod client;
mod config;
mod events;
mod state;
mod ui;

use cli::Args;
use config::clouds::load_clouds_yaml;

fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // Bootstrap async runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()?;

    runtime.block_on(async_main(args))
}

async fn async_main(args: Args) -> Result<()> {
    // Set up logging to file
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("os9s");
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "os9s.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("openstack_tui=debug".parse()?),
        )
        .init();

    tracing::info!("os9s starting up");

    // Load clouds.yaml
    let clouds_path = args.clouds.as_deref();
    let clouds = load_clouds_yaml(clouds_path)?;

    let cloud_name = args.cloud.as_deref().unwrap_or("admin");
    let cloud_config = clouds
        .clouds
        .get(cloud_name)
        .ok_or_else(|| anyhow::anyhow!("Cloud '{}' not found in clouds.yaml", cloud_name))?
        .clone();

    tracing::info!("Using cloud: {}", cloud_name);

    // Authenticate
    let token = auth::keystone::authenticate(&cloud_config).await?;
    tracing::info!("Authenticated successfully, token expires at {}", token.expires_at);

    // Build shared app state
    let app_state = state::AppState::new(cloud_name.to_string(), cloud_config.clone(), token);
    let shared_state = std::sync::Arc::new(tokio::sync::Mutex::new(app_state));

    // Spawn background pollers
    state::poller::spawn_pollers(shared_state.clone()).await;

    // Run TUI
    ui::app::run(shared_state).await?;

    Ok(())
}
