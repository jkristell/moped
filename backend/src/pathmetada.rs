use std::collections::HashMap;
use crate::artwork::CaaState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct FolderInfoCache {
    inner: HashMap<String, FolderMetadata>
}

#[derive(Serialize, Deserialize, Default)]
pub struct FolderMetadata {
    /// Cover Art Archive state
    caa: CaaState,
}

impl FolderInfoCache {

    pub fn caa(&self, path: &str) -> Option<&CaaState> {
        self.inner.get(path).map(|fi| &fi.caa)
    }

    pub fn update_caa(&mut self, path: &str, caa: CaaState) {
        self.inner.insert(
            path.to_string(),
            FolderMetadata {caa}
        );
    }

}