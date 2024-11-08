use crate::types::Float;
use bevy_reflect::Reflect;
use std::fmt::Display;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum ExpressionValue {
    Float(Float),
    String(String),
    Reflect(Arc<dyn Reflect>),
}

impl PartialEq for ExpressionValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExpressionValue::Float(left), ExpressionValue::Float(right)) => left == right,
            (ExpressionValue::String(left), ExpressionValue::String(right)) => left == right,
            (ExpressionValue::Reflect(left), ExpressionValue::Reflect(right)) => false,
            _ => false,
        }
    }
}

impl ExpressionValue {
    pub(crate) fn as_reflect(&self) -> &dyn Reflect {
        match self {
            ExpressionValue::Float(value) => value,
            ExpressionValue::String(value) => value,
            ExpressionValue::Reflect(reflect) => &**reflect,
        }
    }
}

impl Display for ExpressionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionValue::Float(value) => write!(f, "{}", value),
            ExpressionValue::String(value) => write!(f, "{}", value),
            ExpressionValue::Reflect(reflect) => write!(f, "{:?}", reflect.as_ref()),
        }
    }
}
