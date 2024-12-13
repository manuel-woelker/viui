use crate::result::ViuiResult;
use log::info;
use std::io::{BufRead, BufReader, Seek};
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

pub trait BufreadSeek: BufRead + Seek {}
impl<T: BufRead + Seek> BufreadSeek for T {}

impl Resource {
    pub fn from_path<S: Into<PathBuf>>(path: S) -> Self {
        Self {
            inner: Arc::new(ResourceInner { path: path.into() }),
        }
    }

    pub fn as_bytes(&self) -> ViuiResult<Box<[u8]>> {
        info!("Loading resource: '{}'", self.inner.path.display());
        Ok(std::fs::read(&self.inner.path)?.into_boxed_slice())
    }

    pub fn as_path(&self) -> ViuiResult<String> {
        Ok(self.inner.path.display().to_string())
    }

    pub fn buf_reader(&self) -> ViuiResult<Box<dyn BufreadSeek>> {
        Ok(Box::new(BufReader::new(std::fs::File::open(
            &self.inner.path,
        )?)))
    }
}

impl<T: Into<PathBuf>> From<T> for Resource {
    fn from(value: T) -> Self {
        Self::from_path(value)
    }
}
