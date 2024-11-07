use crate::types::Float;

#[derive(Debug)]
pub enum ExpressionValue {
    Float(Float),
    String(String),
}
