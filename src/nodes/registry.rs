use crate::component::ast::ItemAst;
use crate::err;
use crate::infrastructure::layout_context::LayoutContext;
use crate::nodes::data::NodeData;
use crate::nodes::descriptor::{LayoutFn, NodeDescriptor};
use crate::nodes::elements::kind::{Element, EventTrigger, LayoutConstraints};
use crate::nodes::events::InputEvent;
use crate::nodes::types::{NodeEventHandler, NodeEvents, NodeProps, NodeRenderFn, NodeState};
use crate::render::context::RenderContext;
use crate::render::parameters::RenderParameters;
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

    #[allow(clippy::too_many_arguments)]
    pub fn register(
        &mut self,
        name: impl Into<String>,
        make_state: impl Fn() -> ViuiResult<Box<dyn NodeState>> + Send + 'static,
        make_props: impl Fn() -> ViuiResult<Box<dyn NodeProps>> + Send + 'static,
        event_handler: impl Fn(InputEvent, &mut NodeData, &mut EventTrigger<Box<dyn NodeEvents>>) -> ViuiResult<()>
            + Send
            + 'static,
        render_fn: impl Fn(&mut RenderContext, &RenderParameters, &NodeData) -> ViuiResult<()>
            + Send
            + 'static,
        layout_fn: impl Fn(&mut LayoutContext, &mut NodeData) -> ViuiResult<LayoutConstraints>
            + Send
            + 'static,
        children: Vec<ItemAst>,
    ) {
        self.register_internal(
            name.into(),
            Box::new(make_state),
            Box::new(make_props),
            Box::new(event_handler),
            Box::new(render_fn),
            Box::new(layout_fn),
            children,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn register_internal(
        &mut self,
        name: String,
        make_state: Box<dyn Fn() -> ViuiResult<Box<dyn NodeState>> + Send>,
        make_props: Box<dyn Fn() -> ViuiResult<Box<dyn NodeProps>> + Send>,
        event_handler: NodeEventHandler<Box<dyn NodeEvents>>,
        render_fn: NodeRenderFn,
        layout_fn: LayoutFn,
        children: Vec<ItemAst>,
    ) {
        let kind_index = self.nodes.len();
        self.nodes.push(NodeDescriptor {
            kind_index,
            event_handler,
            render_fn,
            make_state,
            make_props,
            layout_fn,
            children,
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
                |render_context: &mut RenderContext,
                 render_parameters: &RenderParameters,
                 node_data: &NodeData| {
                    let (state, props) = node_data.cast_state_and_props::<T::State, T::Props>()?;
                    T::render_element(render_context, render_parameters, state, props);
                    Ok(())
                },
            ),
            Box::new(
                |layout_context: &mut LayoutContext, node_data: &mut NodeData| {
                    let (state, props) =
                        node_data.cast_state_mut_and_props::<T::State, T::Props>()?;
                    T::layout_element(layout_context, state, props)
                },
            ),
            vec![],
        );
    }

    pub fn get_node_by_name(&self, name: &str) -> ViuiResult<&NodeDescriptor> {
        Ok(&self.nodes[*self
            .node_map
            .get(name)
            .ok_or_else(|| err!("Could not find node: {}", name))?])
    }

    pub fn get_node_by_kind(&self, node_kind: usize) -> ViuiResult<&NodeDescriptor> {
        Ok(&self.nodes[node_kind])
    }

    pub fn make_node_props(&self, name: &str) -> ViuiResult<Box<dyn NodeProps>> {
        (self.get_node_by_name(name)?.make_props)()
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
        render_context: &mut RenderContext,
        render_parameters: &RenderParameters,
        node_data: &NodeData,
    ) -> ViuiResult<()> {
        (self.nodes[node_data.kind_index()].render_fn)(render_context, render_parameters, node_data)
    }
    pub fn layout_node(
        &self,
        layout_context: &mut LayoutContext,
        node_data: &mut NodeData,
    ) -> ViuiResult<LayoutConstraints> {
        (self.nodes[node_data.kind_index()].layout_fn)(layout_context, node_data)
    }
}
