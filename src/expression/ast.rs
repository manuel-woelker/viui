use crate::expression::span::Span;
use crate::expression::value::ExpressionValue;
use treeline::Tree;

#[derive(Debug)]
pub struct AstNode<T> {
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
}

pub type ExpressionAst = AstNode<ExpressionKind>;

#[derive(Debug)]
pub enum ExpressionKind {
    Literal(ExpressionValue),
    VarUse(String),
    StringTemplate {
        strings: Vec<String>,
        expressions: Vec<ExpressionAst>,
    },
    //    Concatenation(Vec<AstNode>),
}

pub fn print_expression_ast(ast: &ExpressionAst) -> String {
    format!("{}", expression_ast_to_tree(ast))
}

pub fn expression_ast_to_tree(ast: &ExpressionAst) -> Tree<String> {
    match &ast.data {
        ExpressionKind::Literal(value) => Tree::root(format!("Literal {:?}", value)),
        ExpressionKind::StringTemplate {
            strings,
            expressions,
        } => {
            let mut tree = Tree::root("StringTemplate".to_string());
            for (string, expression) in strings.iter().zip(expressions.iter()) {
                tree.push(Tree::root(format!("String {}", string)));
                tree.push(expression_ast_to_tree(expression));
            }
            tree.push(Tree::root(format!(
                "String {}",
                strings.iter().last().unwrap()
            )));
            tree
        }
        ExpressionKind::VarUse(name) => Tree::root(format!("VarUse {}", name)),
    }
}
