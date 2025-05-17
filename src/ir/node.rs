use crate::ast::nodes::{ComponentAst, ItemDefinition, PropAst, UIAst};
use crate::ast::value::ExpressionValue;
use crate::result::ViuiResult;
use crate::widget::div::DivWidget;
use crate::widget::Widget;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

pub trait WidgetFactory {
    fn create_widget(&self) -> ViuiResult<Box<dyn Widget>>;
}

pub type WidgetFactoryRef = Rc<dyn WidgetFactory>;

#[derive(Clone)]
pub struct DefaultWidgetFactory {}

impl WidgetFactory for DefaultWidgetFactory {
    fn create_widget(&self) -> ViuiResult<Box<dyn Widget>> {
        Ok(Box::new(DivWidget::default()))
    }
}

fn default_widget_factory() -> WidgetFactoryRef {
    Rc::new(DefaultWidgetFactory {})
}

#[derive(Default)]
pub struct WidgetRegistry {
    widgets: HashMap<String, WidgetFactoryRef>,
}

impl WidgetRegistry {
    pub fn get_widget_factory(&self, tag: &str) -> WidgetFactoryRef {
        self.widgets
            .get(tag)
            .cloned()
            .unwrap_or_else(default_widget_factory)
    }
}

impl WidgetRegistry {
    pub fn register_widget<T: WidgetFactory + 'static>(&mut self, name: String, factory: T) {
        self.widgets.insert(name, Rc::new(factory));
    }
}

#[derive(Debug)]
pub struct IrComponent {
    pub name: String,
    pub root: IrNode,
}

pub struct IrNode {
    kind: NodeKind,
    widget_factory: WidgetFactoryRef,
}

impl Debug for IrNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IrNode").field("kind", &self.kind).finish()
    }
}

impl IrNode {
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    pub fn create_widget(&self) -> ViuiResult<Box<dyn Widget>> {
        self.widget_factory.create_widget()
    }
}
#[derive(Debug)]
pub enum NodeKind {
    Block(BlockNode),
    Element(ElementNode),
    If(IfNode),
    For(ForNode),
}

#[derive(Debug)]
pub struct BlockNode {
    pub children: Vec<IrNode>,
}

#[derive(Debug)]
pub struct ElementNode {
    pub name: String,
    pub props: Vec<IrProp>,
}

#[derive(Debug)]
pub struct IrProp {
    pub name: String,
    pub expression: IrExpression,
}

#[derive(Debug)]
pub struct IfNode {
    pub condition: IrExpression,
    pub then: Box<IrNode>,
    pub else_: Option<Box<IrNode>>,
}

#[derive(Debug)]
pub struct ForNode {
    pub variable: String,
    pub iterable: IrExpression,
    pub body: Box<IrNode>,
}

#[derive(Debug, Clone)]
pub enum IrExpression {
    Literal(ExpressionValue),
    Variable(String),
    StringTemplate {
        strings: Vec<String>,
        expressions: Vec<IrExpression>,
    },
    Call {
        callee: Box<IrExpression>,
        arguments: Vec<IrExpression>,
    },
}

pub fn ast_to_ir(ui_ast: &UIAst, widget_registry: &WidgetRegistry) -> ViuiResult<Vec<IrComponent>> {
    let mut ir = vec![];
    for component in &ui_ast.components {
        ir.push(ast_component_to_ir(component, widget_registry)?);
    }
    Ok(ir)
}

fn ast_component_to_ir(
    component: &ComponentAst,
    widget_registry: &WidgetRegistry,
) -> ViuiResult<IrComponent> {
    Ok(IrComponent {
        name: component.name.clone(),
        root: IrNode {
            widget_factory: default_widget_factory(),
            kind: NodeKind::Block(BlockNode {
                children: component
                    .children
                    .iter()
                    .map(|child| ast_item_to_ir(child, widget_registry))
                    .collect::<ViuiResult<_>>()?,
            }),
        },
    })
}

fn ast_item_to_ir(
    item: &crate::ast::nodes::ItemAst,
    widget_registry: &WidgetRegistry,
) -> ViuiResult<IrNode> {
    Ok(match item.data() {
        ItemDefinition::Block { items } => {
            todo!("ast_item_to_ir");
        }
        ItemDefinition::Node { node } => {
            let widget_factory = widget_registry.get_widget_factory(&node.tag);
            IrNode {
                widget_factory,
                kind: NodeKind::Element(ElementNode {
                    name: node.tag.clone(),
                    props: node
                        .props
                        .iter()
                        .map(ast_prop_to_ir)
                        .collect::<ViuiResult<_>>()?,
                }),
            }
        }
        ItemDefinition::If(_) => {
            todo!("ast_item_to_ir");
        }
        ItemDefinition::For(_) => {
            todo!("ast_item_to_ir");
        }
    })
}

fn ast_prop_to_ir(prop: &PropAst) -> ViuiResult<IrProp> {
    Ok(IrProp {
        name: prop.name.clone(),
        expression: ast_expression_to_ir(&prop.expression)?,
    })
}

fn ast_expression_to_ir(expression: &crate::ast::nodes::ExpressionAst) -> ViuiResult<IrExpression> {
    Ok(match expression.data() {
        crate::ast::nodes::ExpressionKind::Literal(value) => IrExpression::Literal(value.clone()),
        crate::ast::nodes::ExpressionKind::VarUse(name) => IrExpression::Variable(name.clone()),
        crate::ast::nodes::ExpressionKind::StringTemplate {
            strings,
            expressions,
        } => IrExpression::StringTemplate {
            strings: strings.clone(),
            expressions: expressions
                .iter()
                .map(ast_expression_to_ir)
                .collect::<ViuiResult<_>>()?,
        },
        crate::ast::nodes::ExpressionKind::Call { callee, arguments } => todo!(),
    })
}

#[cfg(test)]
mod tests {
    use super::{ast_to_ir, WidgetRegistry};
    use crate::ast::parser::parse_ui;

    #[test]
    fn test_ir_expression() {
        // read source from file
        let source = std::fs::read_to_string("examples/simple/label.viui-component").unwrap();
        let ast = parse_ui(&source).unwrap();
        let ir = ast_to_ir(&ast, &WidgetRegistry::default());
        dbg!(ir);
    }
}
