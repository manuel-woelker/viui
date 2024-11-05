use crate::nodes::data::NodeData;
use crate::nodes::elements::kind::EventTrigger;
use crate::nodes::events::NodeEvent;
use crate::render::command::RenderCommand;
use crate::result::ViuiResult;
use bevy_reflect::Reflect;

pub trait NodeState: Reflect + 'static {}

pub trait NodeProps: Reflect + 'static {}

pub type StateBox = Box<dyn NodeState>;
pub type PropsBox = Box<dyn NodeProps>;

pub trait NodeEvents: Reflect + 'static {}
pub type NodeEventHandler =
    Box<dyn Fn(NodeEvent, &mut NodeData, &mut EventTrigger) -> ViuiResult<()> + Send>;
pub type NodeRenderFn = Box<dyn Fn(&mut Vec<RenderCommand>, &NodeData) -> ViuiResult<()> + Send>;

pub type EventList = Vec<String>;

impl NodeState for () {}
