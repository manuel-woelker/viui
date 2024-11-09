use crate::nodes::data::NodeData;
use crate::nodes::types::{NodeEventHandler, NodeEvents, NodeProps, NodeState};
use crate::render::command::RenderCommand;
use crate::result::ViuiResult;

pub type NodeRenderFn = Box<dyn Fn(&mut Vec<RenderCommand>, &NodeData) -> ViuiResult<()> + Send>;

pub struct NodeDescriptor {
    pub(crate) kind_index: usize,
    pub make_state: Box<dyn Fn() -> ViuiResult<Box<dyn NodeState>> + Send>,
    pub make_props: Box<dyn Fn() -> ViuiResult<Box<dyn NodeProps>> + Send>,
    pub event_handler: NodeEventHandler<Box<dyn NodeEvents>>,
    pub render_fn: NodeRenderFn,
}
