#[derive(Debug)]
pub struct IrNode {
    kind: NodeKind,
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
    pub props: Vec<(String, IrExpression)>,
    pub children: Vec<IrNode>,
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
    Literal(String),
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
