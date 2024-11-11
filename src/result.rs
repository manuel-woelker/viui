use error_stack::Report;
use ron::de::SpannedError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;

#[derive(thiserror::Error, Debug)]
pub enum ViuiErrorKind {
    #[error("General Error: {0}")]
    General(String),
}

#[derive(Debug)]
pub struct ViuiError(pub Report<ViuiErrorKind>);

impl Display for ViuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ViuiError {
    #[track_caller]
    pub fn change_context<S: Into<String>>(self, message: S) -> Self {
        Self(
            self.0
                .change_context(ViuiErrorKind::General(message.into())),
        )
    }
}

impl ViuiError {
    #[track_caller]
    pub fn new(error: ViuiErrorKind) -> ViuiError {
        ViuiError(Report::new(error))
    }
}

pub type ViuiResult<T> = Result<T, ViuiError>;

impl<T> From<T> for ViuiError
where
    for<'a> &'a T: Into<ViuiErrorKind>,
    T: Error + Send + Sync + 'static,
{
    #[track_caller]
    fn from(error: T) -> Self {
        let kind: ViuiErrorKind = (&error).into();
        let report = Report::new(error);
        let report = report.change_context(kind);
        Self(report)
    }
}
impl From<&notify::Error> for ViuiErrorKind {
    #[track_caller]
    fn from(error: &notify::Error) -> Self {
        Self::General(error.to_string())
    }
}

impl From<&std::io::Error> for ViuiErrorKind {
    #[track_caller]
    fn from(error: &std::io::Error) -> Self {
        Self::General(error.to_string())
    }
}

impl From<&serde_yml::Error> for ViuiErrorKind {
    #[track_caller]
    fn from(error: &serde_yml::Error) -> Self {
        Self::General(error.to_string())
    }
}

impl From<&regex_lite::Error> for ViuiErrorKind {
    #[track_caller]
    fn from(error: &regex_lite::Error) -> Self {
        Self::General(error.to_string())
    }
}

impl From<&crossbeam_channel::RecvError> for ViuiErrorKind {
    #[track_caller]
    fn from(error: &crossbeam_channel::RecvError) -> Self {
        Self::General(error.to_string())
    }
}

impl From<bevy_reflect::ReflectPathError<'_>> for ViuiError {
    #[track_caller]
    fn from(error: bevy_reflect::ReflectPathError<'_>) -> Self {
        Self::new(ViuiErrorKind::General(error.to_string()))
    }
}

impl From<taffy::TaffyError> for ViuiError {
    #[track_caller]
    fn from(error: taffy::TaffyError) -> Self {
        Self::new(ViuiErrorKind::General(error.to_string()))
    }
}

impl From<String> for ViuiErrorKind {
    #[track_caller]
    fn from(error: String) -> Self {
        Self::General(error)
    }
}

impl From<&ParseFloatError> for ViuiErrorKind {
    #[track_caller]
    fn from(error: &ParseFloatError) -> Self {
        Self::General(format!("Failed to parse float value: {}", error))
    }
}

impl From<&SpannedError> for ViuiErrorKind {
    #[track_caller]
    fn from(error: &SpannedError) -> Self {
        Self::General(format!("RON Error: {}", error))
    }
}

impl From<&str> for ViuiError {
    #[track_caller]
    fn from(error: &str) -> Self {
        Self(Report::new(ViuiErrorKind::General(error.to_string())))
    }
}

#[macro_export]
macro_rules! bail {
    ($($args:tt)+) => {
        return Err($crate::result::ViuiError::new($crate::result::ViuiErrorKind::General(format!($($args)+).into())))
    }
}

#[macro_export]
macro_rules! err {
    ($($args:tt)+) => {
        $crate::result::ViuiError::new($crate::result::ViuiErrorKind::General(format!($($args)+).into()))
    };
}

#[macro_export]
macro_rules! context {
    ($fmt:expr $(, $($args:expr),+)? => $block:block) => {
        {
            $block
        }.map_err(|e: $crate::result::ViuiError| e.change_context(format!(concat!("Failed to ",$fmt) $(, $($args)+)?)))
    };
}
pub use context;

#[cfg(test)]
mod tests {
    use crate::result::ViuiResult;

    #[test]
    fn test_context_macro_ok() {
        let _result = {
            context!("grok stuff for {}", "bar" => {
                Ok(0)
            })
        }
        .unwrap();
    }

    #[test]
    fn test_context_macro_err() {
        fn my_broken_function() -> ViuiResult<u32> {
            Err("ungrokkable")?
        }
        let result = {
            context!("grok stuff for {}", "bar" => {
                my_broken_function()
            })
        }
        .expect_err("Should have errored, but was");
        assert_eq!(
            "General Error: Failed to grok stuff for bar",
            result.to_string()
        );
        assert!(format!("{:?}", result).contains("my_broken_function"));
    }
}
