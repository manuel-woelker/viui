use crate::result::ViuiResult;
use crate::types::Float;
use bevy_reflect::Reflect;
use std::fmt::{Debug, Display};
use std::sync::Arc;

pub trait Function:
    Fn(&[ExpressionValue]) -> ViuiResult<ExpressionValue> + Send + Sync + 'static
{
}
impl<T> Function for T where
    T: Fn(&[ExpressionValue]) -> ViuiResult<ExpressionValue> + Send + Sync + 'static
{
}
pub type ArcFunction = Arc<dyn Function>;

#[derive(Debug, Clone)]
pub enum ExpressionValue {
    Float(Float),
    String(String),
    Reflect(Arc<dyn Reflect>),
    Function(FunctionValue),
}

impl ExpressionValue {
    pub fn function(name: String, fun: impl Function) -> Self {
        ExpressionValue::Function(FunctionValue {
            name,
            fun: Arc::new(fun),
        })
    }
}

pub struct FunctionValue {
    name: String,
    fun: ArcFunction,
}

impl FunctionValue {
    pub(crate) fn invoke(&self, arguments: Vec<ExpressionValue>) -> ViuiResult<ExpressionValue> {
        (self.fun)(&arguments)
    }
}

impl Debug for FunctionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function {}", self.name)
    }
}

impl Clone for FunctionValue {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            fun: self.fun.clone(),
        }
    }
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
            ExpressionValue::Function(_) => todo!("Function as reflect"),
        }
    }
    pub(crate) fn as_reflect_box(&self) -> Box<dyn Reflect> {
        match self {
            ExpressionValue::Float(value) => Box::new(*value),
            ExpressionValue::String(value) => Box::new(value.clone()),
            ExpressionValue::Reflect(reflect) => todo!("Reflect clone"),
            ExpressionValue::Function(_) => todo!("Function as reflect"),
        }
    }
}

impl Display for ExpressionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionValue::Float(value) => write!(f, "{}", value),
            ExpressionValue::String(value) => write!(f, "{}", value),
            ExpressionValue::Reflect(reflect) => write!(f, "{:?}", reflect.as_ref()),
            ExpressionValue::Function(_) => write!(f, "Function"),
        }
    }
}