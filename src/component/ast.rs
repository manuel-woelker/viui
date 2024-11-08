use crate::component::span::Span;
use crate::component::value::ExpressionValue;
use termtree::Tree;

#[derive(Debug)]
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
pub type UIAst = AstNode<UIDefinition>;

#[derive(Debug)]
pub struct UIDefinition {
    pub components: Vec<ComponentAst>,
}
pub type ComponentAst = AstNode<ComponentDefinition>;

#[derive(Debug)]
pub struct ComponentDefinition {
    pub name: String,
    pub children: Vec<NodeAst>,
}

pub type NodeAst = AstNode<NodeDefinition>;

#[derive(Debug)]
pub struct NodeDefinition {
    pub tag: String,
    pub props: Vec<PropAst>,
    //pub children: Vec<UIAst>,
    pub events: Vec<PropAst>,
}

pub type PropAst = AstNode<PropDefinition>;

#[derive(Debug)]
pub struct PropDefinition {
    pub name: String,
    pub expression: ExpressionAst,
    //pub children: Vec<UIAst>,
}

#[derive(Debug)]
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
    for component in &ast.data.components {
        tree.push(component_ast_to_tree(component));
    }
    tree
}

fn component_ast_to_tree(component: &ComponentAst) -> Tree<String> {
    let mut tree = Tree::new(format!("Component {}", component.data.name));
    for child in &component.data.children {
        tree.push(node_ast_to_tree(child));
    }
    tree
}

fn node_ast_to_tree(node_definition: &NodeAst) -> Tree<String> {
    let mut tree = Tree::new(format!("Node {}", node_definition.data.tag));
    for prop in &node_definition.data.props {
        let mut prop_tree = prop_ast_to_tree(prop);
        prop_tree.root += "=";
        tree.push(prop_tree);
    }
    for event in &node_definition.data.events {
        let mut event_tree = prop_ast_to_tree(event);
        event_tree.root.insert(0, '@');
        event_tree.root += "=";
        tree.push(event_tree);
    }
    tree
}

fn prop_ast_to_tree(prop_definition: &PropAst) -> Tree<String> {
    let mut tree = Tree::new(prop_definition.data.name.clone());
    tree.push(expression_ast_to_tree(&prop_definition.data.expression));
    tree
}
