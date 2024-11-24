use crate::bail;
use crate::result::{ViuiError, ViuiResult};
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
    Bool(bool),
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

impl TryFrom<&dyn Reflect> for ExpressionValue {
    type Error = ViuiError;

    fn try_from(value: &dyn Reflect) -> ViuiResult<Self> {
        Ok(if let Some(value) = value.downcast_ref::<Float>() {
            ExpressionValue::Float(*value)
        } else if let Some(value) = value.downcast_ref::<String>() {
            ExpressionValue::String(value.clone())
        } else {
            bail!(
                "Could not convert value to expression value: {:?} {}",
                value,
                value.reflect_short_type_path()
            );
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
            (ExpressionValue::Reflect(_left), ExpressionValue::Reflect(_right)) => false,
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
            ExpressionValue::Bool(value) => value,
            ExpressionValue::Function(_) => todo!("Function as reflect"),
        }
    }
    pub(crate) fn as_reflect_box(&self) -> Box<dyn Reflect> {
        match self {
            ExpressionValue::Float(value) => Box::new(*value),
            ExpressionValue::String(value) => Box::new(value.clone()),
            ExpressionValue::Bool(value) => Box::new(*value),
            ExpressionValue::Reflect(_reflect) => todo!("Reflect clone"),
            ExpressionValue::Function(_) => todo!("Function as reflect"),
        }
    }
}

impl Display for ExpressionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionValue::Bool(value) => write!(f, "{}", value),
            ExpressionValue::Float(value) => write!(f, "{}", value),
            ExpressionValue::String(value) => write!(f, "{}", value),
            ExpressionValue::Reflect(reflect) => write!(f, "{:?}", reflect.as_ref()),
            ExpressionValue::Function(_) => write!(f, "Function"),
        }
    }
}
