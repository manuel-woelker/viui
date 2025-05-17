use crate::arenal::{Arenal, Idx};
use crate::ast::value::ExpressionValue;
use crate::ir::node::{IrComponent, IrExpression, IrNode, NodeKind};
use crate::nodes::data::LayoutInfo;
use crate::result::ViuiResult;
use crate::widget::Widget;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};

pub struct EvalNode {
    tag: String,
    children: Vec<EvalNodeIdx>,
    props: Properties,
    pub layout: LayoutInfo,
    pub widget: Box<dyn Widget>,
}

impl Debug for EvalNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvalNode")
            .field("tag", &self.tag)
            .field("children", &self.children)
            .field("props", &self.props)
            .finish()
    }
}

impl EvalNode {
    pub fn tag(&self) -> &str {
        &self.tag
    }
    pub fn children(&self) -> &[EvalNodeIdx] {
        &self.children
    }
    pub fn props(&self) -> &Properties {
        &self.props
    }
}

pub type EvalNodeIdx = Idx<EvalNode>;

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

pub fn eval_component(
    ir_component: &IrComponent,
    arenal: &mut Arenal<EvalNode>,
) -> ViuiResult<EvalNodeIdx> {
    eval_node(&ir_component.root, arenal)
}

fn eval_node(ir_node: &IrNode, arenal: &mut Arenal<EvalNode>) -> ViuiResult<EvalNodeIdx> {
    match ir_node.kind() {
        NodeKind::Block(block) => {
            let children = block
                .children
                .iter()
                .map(|ir_child| eval_node(ir_child, arenal))
                .collect::<ViuiResult<Vec<EvalNodeIdx>>>()?;
            Ok(arenal.insert(EvalNode {
                tag: "div".to_string(),
                children,
                props: Properties::new(),
                layout: Default::default(),
                widget: ir_node.create_widget()?,
            }))
        }
        NodeKind::Element(element) => Ok(arenal.insert(EvalNode {
            tag: element.name.clone(),
            children: vec![],
            props: element
                .props
                .iter()
                .map(|prop| Ok((prop.name.clone(), eval_expression(&prop.expression)?)))
                .collect::<ViuiResult<_>>()?,
            layout: Default::default(),
            widget: ir_node.create_widget()?,
        })),
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
    use crate::arenal::Arenal;
    use crate::ast::parser::parse_ui;
    use crate::ir::node::ast_to_ir;

    #[test]
    fn test_eval_label() {
        // read source from file
        let source = std::fs::read_to_string("examples/simple/label.viui-component").unwrap();
        let ast = parse_ui(&source).unwrap();
        let ir = ast_to_ir(&ast).unwrap();
        let mut arenal = Arenal::new();
        let evaled = super::eval_component(&ir[0], &mut arenal).unwrap();
        dbg!(evaled);
    }
}
