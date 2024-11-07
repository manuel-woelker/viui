use crate::types::Float;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionValue {
    Float(Float),
    String(String),
}

impl Display for ExpressionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionValue::Float(value) => write!(f, "{}", value),
            ExpressionValue::String(value) => write!(f, "{}", value),
        }
    }
}
