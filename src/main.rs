mod commands;

use anyhow::Result;
use commands::Commands;
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> Result<()> {
    let service = Commands::new().serve(stdio()).await.inspect_err(|e| {
        println!("Error starting server: {}", e);
    })?;

    service.waiting().await?;

    Ok(())
}
