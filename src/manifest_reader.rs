use std::{fs::read_to_string, path::PathBuf};

use crate::{error::ManifestError, manifest::Manifest};

pub trait ManifestReader {
    fn read_from(path: PathBuf) -> Result<Self, ManifestError>
    where
        Self: serde::de::DeserializeOwned,
    {
        let content = read_to_string(&path)
            .map_err(|e| crate::error::ManifestError::FileRead(path.clone(), e))?;

        let manifest: Self = serde_yml::from_str(&content)
            .map_err(|e| crate::error::ManifestError::Parse(path, e))?;

        Ok(manifest)
    }
}

impl ManifestReader for Manifest {}
