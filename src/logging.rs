use crate::result::ViuiResult;
use tracing::Level;

pub fn init_logging() -> ViuiResult<()> {
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(Level::DEBUG)
        .init();
    Ok(())
}
