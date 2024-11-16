use crate::result::ViuiResult;
use log::info;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Resource {
    inner: Arc<ResourceInner>,
}

#[derive(Debug)]
struct ResourceInner {
    path: PathBuf,
}

impl Resource {
    pub fn new<S: Into<PathBuf>>(path: S) -> Self {
        Self {
            inner: Arc::new(ResourceInner { path: path.into() }),
        }
    }

    pub fn as_bytes(&self) -> ViuiResult<Vec<u8>> {
        info!("Loading resource: '{}'", self.inner.path.display());
        Ok(std::fs::read(self.inner.path.clone())?)
    }
}
