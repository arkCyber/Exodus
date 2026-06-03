//! Exodus Hub Server - Standalone Binary
//!
//! Run the Exodus Hub Server independently.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber;

use exodus_hub_server::{manager::ImManager, server::create_im_router, server::ImServerState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create data storage directory
    let data_dir = PathBuf::from("im_data");
    std::fs::create_dir_all(&data_dir)?;
    info!("Created data storage directory: {:?}", data_dir);

    // Database path
    let db_path = data_dir.join("im_server.db");
    info!("Initializing IM manager with database: {:?}", db_path);

    // Create IM manager
    let im_manager = Arc::new(ImManager::new(db_path)?);

    // Create server state
    let state = ImServerState::new(im_manager);

    // Create router
    let app = create_im_router(state);

    // Bind to address
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!("Exodus Hub Server listening on http://0.0.0.0:8080");

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}
