use crate::component::ast::NodeAst;
use crate::nodes::data::NodeData;
use crate::nodes::elements::kind::LayoutConstraints;
use crate::nodes::types::{NodeEventHandler, NodeEvents, NodeProps, NodeRenderFn, NodeState};
use crate::result::ViuiResult;

pub type LayoutFn = Box<dyn Fn(&NodeData) -> ViuiResult<LayoutConstraints> + Send>;

pub struct NodeDescriptor {
    pub(crate) kind_index: usize,
    pub make_state: Box<dyn Fn() -> ViuiResult<Box<dyn NodeState>> + Send>,
    pub make_props: Box<dyn Fn() -> ViuiResult<Box<dyn NodeProps>> + Send>,
    pub event_handler: NodeEventHandler<Box<dyn NodeEvents>>,
    pub layout_fn: LayoutFn,
    pub render_fn: NodeRenderFn,
    pub children: Vec<NodeAst>,
}
