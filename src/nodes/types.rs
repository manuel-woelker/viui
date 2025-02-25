use crate::nodes::data::NodeData;
use crate::nodes::elements::kind::EventTrigger;
use crate::nodes::events::InputEvent;
use crate::render::context::RenderContext;
use crate::render::parameters::RenderParameters;
use crate::result::ViuiResult;
use bevy_reflect::{Enum, Reflect};
use std::fmt::Debug;

pub trait NodeState: Reflect + 'static {}

pub trait NodeProps: Reflect + 'static {}

pub type StateBox = Box<dyn NodeState>;
pub type PropsBox = Box<dyn NodeProps>;

pub trait NodeEvents: Enum + Reflect + Debug + 'static {}
pub type NodeEventHandler<E> =
    Box<dyn Fn(InputEvent, &mut NodeData, &mut EventTrigger<'_, E>) -> ViuiResult<()> + Send>;
pub type NodeRenderFn =
    Box<dyn Fn(&mut RenderContext, &RenderParameters, &NodeData) -> ViuiResult<()> + Send>;

impl NodeState for () {}
impl NodeProps for () {}
