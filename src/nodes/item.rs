use crate::arenal::Idx;
use crate::ast::nodes::ExpressionAst;
use crate::nodes::data::NodeIdx;

#[derive(Clone)]
pub struct NodeItem {
    pub kind: NodeItemKind,
}

pub type ItemIdx = Idx<NodeItem>;

#[derive(Clone)]
pub enum NodeItemKind {
    Node(NodeIdx),
    If(IfItem),
    Block(BlockItem),
    For(ForItem),
}

#[derive(Clone)]
pub struct IfItem {
    pub condition: bool,
    pub condition_expression: ExpressionAst,
    pub then_item: ItemIdx,
    pub else_item: Option<ItemIdx>,
}

#[derive(Clone)]
pub struct BlockItem {
    pub items: Vec<ItemIdx>,
}

#[derive(Clone)]
pub struct ForItem {
    pub expression: ExpressionAst,
    pub binding_name: String,
    pub item_template: ItemIdx,
    pub items: Vec<ItemIdx>,
}
