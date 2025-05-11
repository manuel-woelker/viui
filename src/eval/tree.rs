use crate::ast::value::ExpressionValue;
use crate::ir::node::{IrComponent, IrExpression, IrNode, NodeKind};
use crate::result::ViuiResult;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

#[derive(Debug)]
pub struct EvalNode {
    tag: String,
    children: Vec<EvalNode>,
    props: Properties,
}

impl EvalNode {
    pub fn tag(&self) -> &str {
        &self.tag
    }
    pub fn children(&self) -> &[EvalNode] {
        &self.children
    }
    pub fn props(&self) -> &Properties {
        &self.props
    }
}

#[derive(Debug)]
pub struct Properties {
    values: HashMap<String, EvalValue>,
}

impl Properties {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&EvalValue> {
        self.values.get(name)
    }
}

impl FromIterator<(String, EvalValue)> for Properties {
    fn from_iter<T: IntoIterator<Item = (String, EvalValue)>>(iter: T) -> Self {
        let mut map = HashMap::new();
        map.extend(iter);
        Self { values: map }
    }
}

#[derive(Debug)]
pub enum EvalValue {
    String(String),
}

impl EvalValue {
    pub fn as_str(&self) -> &str {
        match self {
            EvalValue::String(value) => value,
        }
    }
}

impl From<String> for EvalValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for EvalValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<&ExpressionValue> for EvalValue {
    fn from(value: &ExpressionValue) -> Self {
        match value {
            ExpressionValue::String(value) => Self::String(value.clone()),
            _ => todo!(),
        }
    }
}

pub fn eval_component(ir_component: &IrComponent) -> ViuiResult<EvalNode> {
    eval_node(&ir_component.root)
}

fn eval_node(ir_node: &IrNode) -> ViuiResult<EvalNode> {
    match ir_node.kind() {
        NodeKind::Block(block) => {
            let children = block
                .children
                .iter()
                .map(eval_node)
                .collect::<ViuiResult<Vec<EvalNode>>>()?;
            Ok(EvalNode {
                tag: "div".to_string(),
                children,
                props: Properties::new(),
            })
        }
        NodeKind::Element(element) => Ok(EvalNode {
            tag: element.name.clone(),
            children: vec![],
            props: element
                .props
                .iter()
                .map(|prop| Ok((prop.name.clone(), eval_expression(&prop.expression)?)))
                .collect::<ViuiResult<_>>()?,
        }),
        _ => todo!(),
    }
}

fn eval_expression(ir_expression: &IrExpression) -> ViuiResult<EvalValue> {
    Ok(match ir_expression {
        IrExpression::Literal(value) => value.into(),
        _ => todo!(),
    })
}

#[cfg(test)]
mod tests {
    use crate::ast::parser::parse_ui;
    use crate::ir::node::ast_to_ir;

    #[test]
    fn test_eval_label() {
        // read source from file
        let source = std::fs::read_to_string("examples/simple/label.viui-component").unwrap();
        let ast = parse_ui(&source).unwrap();
        let ir = ast_to_ir(&ast).unwrap();
        let evaled = super::eval_component(&ir[0]).unwrap();
        dbg!(evaled);
    }
}
