use std::{fs::read_to_string, path::PathBuf};

use crate::manifest::Manifest;

impl Manifest {
    pub fn from(path: PathBuf) -> Result<Self, crate::error::ManifestError> {
        let content = read_to_string(&path)
            .map_err(|e| crate::error::ManifestError::FileRead(path.clone(), e))?;

        let manifest: Self = serde_yml::from_str(&content)
            .map_err(|e| crate::error::ManifestError::Parse(path, e))?;

        Ok(manifest)
    }
}
