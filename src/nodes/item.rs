use crate::arenal::Idx;
use crate::component::ast::ExpressionAst;
use crate::nodes::data::NodeIdx;

pub struct NodeItem {
    pub kind: NodeItemKind,
}

pub type ItemIdx = Idx<NodeItem>;

pub enum NodeItemKind {
    Node(NodeIdx),
    If(IfItem),
    Block(BlockItem),
}

pub struct IfItem {
    pub condition: bool,
    pub condition_expression: ExpressionAst,
    pub then_item: ItemIdx,
    pub else_item: Option<ItemIdx>,
}

pub struct BlockItem {
    pub items: Vec<ItemIdx>,
}
