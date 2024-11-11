use crate::nodes::data::NodeData;
use crate::nodes::descriptor::{LayoutFn, NodeDescriptor};
use crate::nodes::elements::kind::{Element, EventTrigger, LayoutConstraints};
use crate::nodes::events::InputEvent;
use crate::nodes::types::{NodeEventHandler, NodeEvents, NodeProps, NodeRenderFn, NodeState};
use crate::render::command::RenderCommand;
use crate::result::ViuiResult;
use std::collections::HashMap;

pub struct NodeRegistry {
    pub nodes: Vec<NodeDescriptor>,
    pub node_map: HashMap<String, usize>,
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        name: impl Into<String>,
        make_state: impl Fn() -> ViuiResult<Box<dyn NodeState>> + Send + 'static,
        make_props: impl Fn() -> ViuiResult<Box<dyn NodeProps>> + Send + 'static,
        event_handler: impl Fn(InputEvent, &mut NodeData, &mut EventTrigger<Box<dyn NodeEvents>>) -> ViuiResult<()>
            + Send
            + 'static,
        render_fn: impl Fn(&mut Vec<RenderCommand>, &NodeData) -> ViuiResult<()> + Send + 'static,
        layout_fn: impl Fn(&NodeData) -> ViuiResult<LayoutConstraints> + Send + 'static,
    ) {
        self.register_internal(
            name.into(),
            Box::new(make_state),
            Box::new(make_props),
            Box::new(event_handler),
            Box::new(render_fn),
            Box::new(layout_fn),
        );
    }

    fn register_internal(
        &mut self,
        name: String,
        make_state: Box<dyn Fn() -> ViuiResult<Box<dyn NodeState>> + Send>,
        make_props: Box<dyn Fn() -> ViuiResult<Box<dyn NodeProps>> + Send>,
        event_handler: NodeEventHandler<Box<dyn NodeEvents>>,
        render_fn: NodeRenderFn,
        layout_fn: LayoutFn,
    ) {
        let kind_index = self.nodes.len();
        self.nodes.push(NodeDescriptor {
            kind_index,
            event_handler,
            render_fn,
            make_state,
            make_props,
            layout_fn,
        });
        self.node_map.insert(name, kind_index);
    }

    pub fn register_node<T: Element>(&mut self) {
        self.register(
            T::NAME,
            || Ok(Box::new(T::State::default())),
            || Ok(Box::new(T::Props::default())),
            Box::new(
                |event: InputEvent,
                 node_data: &mut NodeData,
                 event_trigger: &mut EventTrigger<Box<dyn NodeEvents>>| {
                    let (state, props) =
                        node_data.cast_state_mut_and_props::<T::State, T::Props>()?;
                    let mut event_handler = |e: T::Events| event_trigger(Box::new(e));
                    T::handle_event(&event, state, props, &mut event_handler);
                    Ok(())
                },
            ),
            Box::new(
                |render_queue: &mut Vec<RenderCommand>, node_data: &NodeData| {
                    let (state, props) = node_data.cast_state_and_props::<T::State, T::Props>()?;
                    T::render_element(render_queue, state, props);
                    Ok(())
                },
            ),
            Box::new(|node_data: &NodeData| {
                let (state, props) = node_data.cast_state_and_props::<T::State, T::Props>()?;
                T::layout_element(state, props)
            }),
        );
    }

    pub fn get_node_by_name(&self, name: &str) -> &NodeDescriptor {
        &self.nodes[*self.node_map.get(name).unwrap()]
    }

    pub fn make_node_props(&self, name: &str) -> ViuiResult<Box<dyn NodeProps>> {
        (self.get_node_by_name(name).make_props)()
    }

    pub fn handle_event(
        &self,
        node_index: usize,
        event: InputEvent,
        node_data: &mut NodeData,
        event_trigger: &mut EventTrigger<Box<dyn NodeEvents>>,
    ) -> ViuiResult<()> {
        (self.nodes[node_index].event_handler)(event, node_data, event_trigger)
    }

    pub fn get_node_index(&self, kind: &str) -> usize {
        *self.node_map.get(kind).unwrap()
    }

    pub fn render_node(
        &self,
        render_queue: &mut Vec<RenderCommand>,
        node_data: &NodeData,
    ) -> ViuiResult<()> {
        (self.nodes[node_data.kind_index()].render_fn)(render_queue, node_data)
    }
    pub fn layout_node(&self, node_data: &NodeData) -> ViuiResult<LayoutConstraints> {
        (self.nodes[node_data.kind_index()].layout_fn)(node_data)
    }
}
