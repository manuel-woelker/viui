use crate::component::span::Span;
use crate::component::value::ExpressionValue;
use std::ops::{Deref, DerefMut};
use termtree::Tree;

#[derive(Debug, Clone)]
pub struct AstNode<T> {
    #[allow(dead_code)]
    span: Span,
    data: T,
}

impl<T> AstNode<T> {
    pub fn new(span: Span, data: T) -> Self {
        Self { span, data }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn into_data(self) -> T {
        self.data
    }
}

impl<T> Deref for AstNode<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for AstNode<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub type UIAst = AstNode<UIDefinition>;

#[derive(Debug)]
pub struct UIDefinition {
    pub components: Vec<ComponentAst>,
}
pub type ComponentAst = AstNode<ComponentDefinition>;

#[derive(Debug)]
pub struct ComponentDefinition {
    pub name: String,
    pub children: Vec<ItemAst>,
}

pub type NodeAst = AstNode<NodeDefinition>;

#[derive(Debug, Clone)]
pub struct NodeDefinition {
    pub tag: String,
    pub props: Vec<PropAst>,
    pub children: Vec<ItemAst>,
    pub events: Vec<PropAst>,
}

pub type PropAst = AstNode<PropDefinition>;

#[derive(Debug, Clone)]
pub struct PropDefinition {
    pub name: String,
    pub expression: ExpressionAst,
    //pub children: Vec<UIAst>,
}

#[derive(Debug, Clone)]
pub enum ItemDefinition {
    Block { items: Vec<ItemAst> },
    Node { node: NodeAst },
    If(Box<IfItemDefinition>),
    For(Box<ForItemDefinition>),
}

#[derive(Debug, Clone)]
pub struct IfItemDefinition {
    pub condition: ExpressionAst,
    pub then_item: ItemAst,
    pub else_item: Option<ItemAst>,
}

#[derive(Debug, Clone)]
pub struct ForItemDefinition {
    pub expression: ExpressionAst,
    pub binding_name: String,
    pub each_item: ItemAst,
}

pub type ItemAst = AstNode<ItemDefinition>;

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Literal(ExpressionValue),
    VarUse(String),
    StringTemplate {
        strings: Vec<String>,
        expressions: Vec<ExpressionAst>,
    },
    Call {
        callee: Box<ExpressionAst>,
        arguments: Vec<ExpressionAst>,
    },
}
pub type ExpressionAst = AstNode<ExpressionKind>;

pub fn print_expression_ast(ast: &ExpressionAst) -> String {
    format!("{}", expression_ast_to_tree(ast))
}
pub fn print_ui_ast(ast: &UIAst) -> String {
    format!("{}", ui_ast_to_tree(ast))
}

fn expression_ast_to_tree(ast: &ExpressionAst) -> Tree<String> {
    match &ast.data {
        ExpressionKind::Literal(value) => Tree::new(format!("Literal {:?}", value)),
        ExpressionKind::StringTemplate {
            strings,
            expressions,
        } => {
            let mut tree = Tree::new("StringTemplate".to_string());
            for (string, expression) in strings.iter().zip(expressions.iter()) {
                tree.push(Tree::new(format!("String {}", string)));
                tree.push(expression_ast_to_tree(expression));
            }
            tree.push(Tree::new(format!(
                "String {}",
                strings.iter().last().unwrap()
            )));
            tree
        }
        ExpressionKind::VarUse(name) => Tree::new(format!("VarUse {}", name)),
        ExpressionKind::Call { callee, arguments } => {
            let mut tree = Tree::new("Call".to_string());
            tree.push(expression_ast_to_tree(callee));
            for argument in arguments {
                tree.push(expression_ast_to_tree(argument));
            }
            tree
        }
    }
}

fn ui_ast_to_tree(ast: &UIAst) -> Tree<String> {
    let mut tree = Tree::new("UIDefinition".to_string());
    for component in &ast.components {
        tree.push(component_ast_to_tree(component));
    }
    tree
}

fn component_ast_to_tree(component: &ComponentAst) -> Tree<String> {
    let mut tree = Tree::new(format!("Component {}", component.data.name));
    for child in &component.children {
        tree.push(item_ast_to_tree(child));
    }
    tree
}

fn node_ast_to_tree(node_definition: &NodeAst) -> Tree<String> {
    let mut tree = Tree::new(format!("Node {}", node_definition.data.tag));
    for prop in &node_definition.props {
        let mut prop_tree = prop_ast_to_tree(prop);
        prop_tree.root += "=";
        tree.push(prop_tree);
    }
    for event in &node_definition.events {
        let mut event_tree = prop_ast_to_tree(event);
        event_tree.root.insert(0, '@');
        event_tree.root += "=";
        tree.push(event_tree);
    }
    for child in &node_definition.children {
        let mut event_tree = item_ast_to_tree(child);
        event_tree.root.insert_str(0, "child: ");
        tree.push(event_tree);
    }
    tree
}

fn item_ast_to_tree(item_ast: &ItemAst) -> Tree<String> {
    match &item_ast.data {
        ItemDefinition::Node { node } => node_ast_to_tree(&node),
        ItemDefinition::If(if_item) => {
            let mut if_tree = expression_ast_to_tree(&if_item.condition);
            if_tree.root.insert_str(0, "if ");
            let mut then_tree = item_ast_to_tree(&if_item.then_item);
            then_tree.root = "then".to_string();
            if_tree.push(then_tree);
            if let Some(else_item) = &if_item.else_item {
                let mut else_tree = item_ast_to_tree(else_item);
                else_tree.root = "else".to_string();
                if_tree.push(else_tree);
            }
            if_tree
        }
        ItemDefinition::For(for_item) => {
            let mut for_tree = expression_ast_to_tree(&for_item.expression);
            for_tree
                .root
                .insert_str(0, &format!("for {} in ", for_item.binding_name));
            let mut each_tree = item_ast_to_tree(&for_item.each_item);
            each_tree.root += "each";
            for_tree.push(each_tree);
            for_tree
        }
        ItemDefinition::Block { items } => {
            let mut tree = Tree::new("".to_string());
            for item in items {
                tree.push(item_ast_to_tree(item));
            }
            tree
        }
    }
}

fn prop_ast_to_tree(prop_definition: &PropAst) -> Tree<String> {
    let mut tree = Tree::new(prop_definition.name.clone());
    tree.push(expression_ast_to_tree(&prop_definition.expression));
    tree
}
