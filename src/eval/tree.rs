use crate::ast::value::ExpressionValue;
use crate::ir::node::{IrComponent, IrExpression, IrNode, NodeKind};
use crate::result::ViuiResult;

#[derive(Debug)]
pub struct EvalNode {
    tag: String,
    children: Vec<EvalNode>,
    props: Vec<EvalProp>,
}

#[derive(Debug)]
pub struct EvalProp {
    pub name: String,
    pub value: EvalValue,
}

#[derive(Debug)]
pub enum EvalValue {
    String(String),
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
                props: vec![],
            })
        }
        NodeKind::Element(element) => Ok(EvalNode {
            tag: element.name.clone(),
            children: vec![],
            props: element
                .props
                .iter()
                .map(|prop| {
                    Ok(EvalProp {
                        name: prop.name.clone(),
                        value: eval_expression(&prop.expression)?,
                    })
                })
                .collect::<ViuiResult<Vec<_>>>()?,
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
