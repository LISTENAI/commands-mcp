use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("Failed to read manifest file {0}: {1}")]
    FileRead(PathBuf, std::io::Error),

    #[error("Failed to parse manifest file {0}: {1}")]
    Parse(PathBuf, serde_yml::Error),
}
