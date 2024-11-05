use crate::nodes::data::NodeData;
use crate::nodes::descriptor::NodeDescriptor;
use crate::nodes::elements::kind::{Element, EventTrigger};
use crate::nodes::events::NodeEvent;
use crate::nodes::types::{EventList, NodeEventHandler, NodeProps, NodeRenderFn, NodeState};
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
        event_handler: impl Fn(NodeEvent, &mut NodeData, &mut EventTrigger) -> ViuiResult<()>
            + Send
            + 'static,
        render_fn: impl Fn(&mut Vec<RenderCommand>, &NodeData) -> ViuiResult<()> + Send + 'static,
        emitted_events: EventList,
    ) {
        self.register_internal(
            name.into(),
            Box::new(make_state),
            Box::new(make_props),
            Box::new(event_handler),
            Box::new(render_fn),
            emitted_events,
        );
    }

    fn register_internal(
        &mut self,
        name: String,
        make_state: Box<dyn Fn() -> ViuiResult<Box<dyn NodeState>> + Send>,
        make_props: Box<dyn Fn() -> ViuiResult<Box<dyn NodeProps>> + Send>,
        event_handler: NodeEventHandler,
        render_fn: NodeRenderFn,
        emitted_events: EventList,
    ) {
        let kind_index = self.nodes.len();
        self.nodes.push(NodeDescriptor {
            kind_index,
            event_handler,
            render_fn,
            make_state,
            make_props,
            emitted_events,
        });
        self.node_map.insert(name, kind_index);
    }

    pub fn register_node<T: Element>(&mut self, emitted_events: EventList) {
        self.register(
            T::NAME,
            || Ok(Box::new(T::State::default())),
            || Ok(Box::new(T::Props::default())),
            Box::new(
                |event: NodeEvent, node_data: &mut NodeData, event_trigger: &mut EventTrigger| {
                    let (state, props) =
                        node_data.cast_state_mut_and_props::<T::State, T::Props>()?;
                    T::handle_event(&event, state, props, event_trigger);
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
            emitted_events,
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
        event: NodeEvent,
        node_data: &mut NodeData,
        event_trigger: &mut EventTrigger,
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
}
