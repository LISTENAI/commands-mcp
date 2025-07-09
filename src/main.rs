mod commands;
mod commands_flash;
mod commands_serial;
mod error;
mod manifest;
mod manifest_executor;
mod manifest_reader;
mod manifest_schema;

use std::{env::current_dir, path::PathBuf};

use anyhow::Result;
use clap::{Parser, crate_description};
use commands::Commands;
use rmcp::{ServiceExt, transport::stdio};

#[derive(Parser)]
#[command(author, version, about = crate_description!())]
struct Args {
    /// Name of manifest file
    #[arg(short, long, default_value = "commands.yaml", value_name = "MANIFEST")]
    manifest: String,

    /// Path to the working directory
    #[arg(default_value = ".", value_name = "WORKING_DIRECTORY")]
    working_directory: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let working_directory = args
        .working_directory
        .unwrap_or_else(|| current_dir().expect("Failed to get current directory"));

    let manifest_path = working_directory.join(&args.manifest);

    let manifest = manifest::Manifest::from(manifest_path)
        .map_err(|e| anyhow::anyhow!("Failed to load manifest: {}", e))?;

    let service = Commands::new(working_directory, manifest)
        .serve(stdio())
        .await
        .inspect_err(|e| {
            println!("Error starting server: {}", e);
        })?;

    service.waiting().await?;

    Ok(())
}
