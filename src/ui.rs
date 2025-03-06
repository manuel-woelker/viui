use crate::arenal::Arenal;
use crate::bail;
use crate::component::ast::{ComponentAst, ExpressionAst, ItemAst, ItemDefinition};
use crate::component::eval::eval;
use crate::component::parser::parse_ui;
use crate::component::value::ExpressionValue;
use crate::infrastructure::binding_stack::BindingStack;
use crate::infrastructure::font_pool::FontPool;
use crate::infrastructure::image_pool::ImagePool;
use crate::infrastructure::layout_context::LayoutContext;
use crate::infrastructure::styling::Styling;
use crate::nodes::data::{LayoutInfo, NodeData, NodeIdx, PropExpression};
use crate::nodes::elements::button::ButtonElement;
use crate::nodes::elements::hstack::HStackElement;
use crate::nodes::elements::image::ImageElement;
use crate::nodes::elements::kind::{Element, LayoutConstraints};
use crate::nodes::elements::knob::KnobElement;
use crate::nodes::elements::label::LabelElement;
use crate::nodes::elements::spinner::SpinnerElement;
use crate::nodes::elements::textinput::TextInputElement;
use crate::nodes::events::{InputEvent, MouseEventKind, UiEvent, UiEventKind};
use crate::nodes::item::{BlockItem, ForItem, IfItem, ItemIdx, NodeItem, NodeItemKind};
use crate::nodes::registry::NodeRegistry;
use crate::nodes::types::NodeEvents;
use crate::observable_state::ObservableState;
use crate::render::backend::RenderBackendParameters;
use crate::render::command::RenderCommand;
use crate::render::context::RenderContext;
use crate::render::parameters::RenderParameters;
use crate::resource::Resource;
use crate::result::{context, ViuiResult};
use crate::types::{Point, Rect, Size};
use bevy_reflect::{
    DynamicEnum, DynamicTuple, DynamicVariant, FromReflect, GetPath, Reflect, ReflectRef, TypeInfo,
    Typed, VariantInfo,
};
use crossbeam_channel::{select, tick, Receiver, Sender};
use itertools::{EitherOrBoth, Itertools};
use log::debug;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::mem::take;
use std::ops::IndexMut;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use taffy::prelude::length;
use taffy::{FlexDirection, Style, TaffyTree};
use tracing::error;

pub type ApplicationEventHandler = Box<dyn Fn(&mut ObservableState, &dyn Reflect) + Send>;
pub type MessageStringToEnumConverter = Box<dyn Fn(&str) -> ViuiResult<ExpressionValue> + Send>;

pub struct UI {
    root_component_name: String,
    node_registry: NodeRegistry,
    node_arena: Arenal<NodeData>,
    item_arena: Arenal<NodeItem>,
    root_item_idx: ItemIdx,
    root_node_idx: NodeIdx,
    app_state: Box<ObservableState>,
    event_handler: ApplicationEventHandler,
    message_string_to_enum_converter: MessageStringToEnumConverter,
    mouse_position: Point,
    render_backends: Vec<RenderBackend>,
    ui_event_receiver: Receiver<UiEvent>,
    ui_event_sender: Sender<UiEvent>,
    file_change_receiver: Receiver<()>,
    file_watcher: Debouncer<RecommendedWatcher>,
    root_node_file: PathBuf,
    active_nodes: Vec<NodeIdx>,
    animated_nodes: Vec<NodeIdx>,
    image_pool: ImagePool,
    font_pool: FontPool,
    start: Instant,
    styling: Styling,
}

struct RenderBackend {
    render_backend_sender: Sender<RenderBackendMessage>,
    maximum_font_index_loaded: usize,
    window_size: Size,
}

pub struct RenderBackendMessage {
    pub(crate) render_commands: Vec<RenderCommand>,
}

pub trait AppMessage: DeserializeOwned + Reflect + FromReflect + Debug + Sized + Typed {}
impl<T> AppMessage for T where T: DeserializeOwned + Reflect + FromReflect + Debug + Sized + Typed {}

impl UI {
    pub fn new<MESSAGE: AppMessage>(
        state: ObservableState,
        root_component_name: String,
        event_handler: impl Fn(&mut ObservableState, &MESSAGE) + Send + 'static,
    ) -> ViuiResult<UI> {
        let (event_sender, event_receiver) = crossbeam_channel::bounded::<UiEvent>(4);
        let (file_change_sender, file_change_receiver) = crossbeam_channel::bounded::<()>(4);
        let message_string_to_enum_converter = Self::make_enum_variant_for_name::<MESSAGE>();
        let file_watcher = new_debouncer(
            Duration::from_millis(10),
            move |res: DebounceEventResult| match res {
                Ok(_events) => {
                    if let Err(err) = file_change_sender.send(()) {
                        error!("File watcher error {:?}", err)
                    }
                }
                Err(err) => error!("File watcher error {:?}", err),
            },
        )?;

        let mut node_registry = NodeRegistry::new();
        node_registry.register_node::<LabelElement>();
        node_registry.register_node::<TextInputElement>();
        node_registry.register_node::<ButtonElement>();
        node_registry.register_node::<KnobElement>();
        node_registry.register_node::<HStackElement>();
        node_registry.register_node::<ImageElement>();
        node_registry.register_node::<SpinnerElement>();
        let mut font_pool = FontPool::new();
        font_pool.load_font(Resource::from_path("assets/fonts/Quicksand-Regular.ttf"))?;
        Ok(UI {
            root_component_name,
            node_registry,
            node_arena: Arenal::new(),
            item_arena: Arenal::new(),
            app_state: Box::new(state),
            event_handler: Box::new(move |state, message| {
                let typed_message = message.downcast_ref::<MESSAGE>().unwrap();
                event_handler(state, typed_message);
            }),
            mouse_position: Default::default(),
            render_backends: Vec::new(),
            ui_event_receiver: event_receiver,
            ui_event_sender: event_sender,
            message_string_to_enum_converter,
            file_change_receiver,
            file_watcher,
            root_node_file: Default::default(),
            active_nodes: Default::default(),
            root_item_idx: Default::default(),
            root_node_idx: Default::default(),
            image_pool: Default::default(),
            font_pool,
            start: Instant::now(),
            animated_nodes: Default::default(),
            //styling: Styling::light(),
            styling: Styling::dark(),
        })
    }

    fn make_enum_variant_for_name<MESSAGE: AppMessage>() -> MessageStringToEnumConverter {
        let message_string_to_enum_converter =
            Box::new(|variant_name: &str| -> ViuiResult<ExpressionValue> {
                let type_info = MESSAGE::type_info();
                let TypeInfo::Enum(enum_info) = type_info else {
                    bail!("Not an enum value: {}", type_info.type_path());
                };
                let Some(variant_info) = enum_info.variant(variant_name) else {
                    bail!(
                        "Not enum variant: {}::{} (found variants: {})",
                        type_info.type_path(),
                        variant_name,
                        enum_info.variant_names().join(", ")
                    );
                };
                match variant_info {
                    VariantInfo::Unit(_unit_info) => {
                        let dynamic_enum = DynamicEnum::new(variant_name, DynamicVariant::Unit);
                        let message = MESSAGE::from_reflect(&dynamic_enum).unwrap();
                        Ok(ExpressionValue::Reflect(Arc::new(message)))
                    }
                    VariantInfo::Tuple(_tuple_info) => {
                        let variant_name = variant_name.to_string();
                        Ok(ExpressionValue::function(
                            variant_name.clone(),
                            move |args: &[ExpressionValue]| -> ViuiResult<ExpressionValue> {
                                let mut tuple = DynamicTuple::default();
                                for arg in args {
                                    tuple.insert_boxed(arg.as_reflect_box());
                                }
                                let dynamic_enum =
                                    DynamicEnum::new(&variant_name, DynamicVariant::Tuple(tuple));
                                let message = MESSAGE::from_reflect(&dynamic_enum).unwrap();
                                Ok(ExpressionValue::Reflect(Arc::new(message)))
                            },
                        ))
                    }
                    VariantInfo::Struct(_) => {
                        todo!("Implement struct enum variant");
                    }
                }
            });
        message_string_to_enum_converter
    }

    pub fn start(mut self) -> ViuiResult<()> {
        thread::Builder::new()
            .name("VIUI Thread".into())
            .spawn(move || {
                debug!("Running main loop");
                let ticker = tick(Duration::from_micros(1_000_000 / 60));
                loop {
                    let result: ViuiResult<()> = (|| {
                        select! {
                            recv(self.file_change_receiver) -> _event => {
                                self.load_root_node_file()?;
                                self.eval_layout_and_redraw()?;
                            }
                            recv(self.ui_event_receiver) -> event => {
                                self.handle_ui_event(event?)?;
                                self.eval_layout_and_redraw()?;
                            }
                            recv(ticker) -> _ => {
                                if !self.animated_nodes.is_empty() {
                                    self.redraw()?;
                                }
                            }
                        }
                        Ok(())
                    })();
                    if let Err(err) = result {
                        error!("Error in VIUI Thread: {:?}", err);
                        std::process::exit(1);
                    }
                }
            })?;
        Ok(())
    }

    pub fn eval_layout_and_redraw(&mut self) -> ViuiResult<()> {
        self.eval_expressions()?;
        self.perform_layout()?;
        self.redraw()?;
        Ok(())
    }

    fn redraw(&mut self) -> ViuiResult<()> {
        let mut render_backends = take(&mut self.render_backends);
        for backend in &mut render_backends {
            let render_commands = self.make_render_commands(backend)?;
            backend
                .render_backend_sender
                .send(RenderBackendMessage { render_commands })
                .unwrap();
        }
        self.render_backends = render_backends;
        Ok(())
    }

    pub fn event_sender(&self) -> Sender<UiEvent> {
        self.ui_event_sender.clone()
    }

    pub fn add_render_backend(&mut self) -> ViuiResult<RenderBackendParameters> {
        let (render_backend_sender, message_receiver) =
            crossbeam_channel::bounded::<RenderBackendMessage>(4);
        let backend_index = self.render_backends.len();
        self.render_backends.push(RenderBackend {
            render_backend_sender,
            maximum_font_index_loaded: 0,
            window_size: Size::new(1200.0, 1200.0),
        });
        self.eval_layout_and_redraw()?;
        Ok(RenderBackendParameters {
            message_receiver,
            event_sender: self.event_sender(),
            backend_index,
            initial_window_size: Size::new(1200.0, 1200.0),
        })
    }

    pub fn nodes(&mut self) -> impl Iterator<Item = &mut NodeData> {
        self.node_arena.entries_mut()
    }

    pub fn handle_ui_event(&mut self, event: UiEvent) -> ViuiResult<()> {
        let mut events_to_trigger = Vec::new();
        let mut add_event_trigger = |node_idx: NodeIdx, node_event: InputEvent| {
            events_to_trigger.push((node_idx, node_event));
        };
        // Clear out nodes that no longer exist
        self.active_nodes
            .retain(|node_idx| self.node_arena.contains(node_idx));
        match event.kind {
            UiEventKind::MouseMoved(position) => {
                self.mouse_position = position;
                for node in &self.active_nodes {
                    add_event_trigger(*node, InputEvent::mouse_move(position));
                }
                for (node, node_idx) in self.node_arena.entries_mut_indexed() {
                    if node.layout.bounds.contains(position) {
                        add_event_trigger(node_idx, InputEvent::mouse_over());
                    } else {
                        add_event_trigger(node_idx, InputEvent::mouse_out());
                    }
                }
            }
            UiEventKind::MouseInput(input) => {
                self.active_nodes.clear();
                let position = self.mouse_position;
                for (node, idx) in self.node_arena.entries_mut_indexed() {
                    if node.layout.bounds.contains(position) {
                        self.active_nodes.push(idx);
                        if input.mouse_event_kind == MouseEventKind::Pressed {
                            add_event_trigger(idx, InputEvent::mouse_press(position));
                        } else if input.mouse_event_kind == MouseEventKind::Released {
                            add_event_trigger(idx, InputEvent::mouse_release(position));
                        }
                    }
                }
            }
            UiEventKind::CharInput(character) => {
                for node in &self.active_nodes {
                    add_event_trigger(*node, InputEvent::character(character.character));
                }
            }
            UiEventKind::KeyInput(key_input) => {
                for node in &self.active_nodes {
                    add_event_trigger(*node, InputEvent::key_input(key_input.key));
                }
            }
            UiEventKind::WindowResized {
                size,
                backend_index,
            } => {
                self.render_backends[backend_index].window_size = size;
            }
        }

        for (node_idx, event) in events_to_trigger {
            let node = &mut self.node_arena[&node_idx];
            let mut events = Vec::new();
            let mut event_trigger = |event: Box<dyn NodeEvents>| {
                events.push(event);
            };
            self.node_registry
                .handle_event(node.kind_index, event, node, &mut event_trigger)?;
            for event in events {
                let ReflectRef::Enum(dyn_enum) = event.reflect_ref() else {
                    bail!(
                        "Event is not an enum: {}",
                        event.get_represented_type_info().unwrap().type_path()
                    );
                };
                let variant_name = dyn_enum.variant_name().to_lowercase();
                if let Some(message_expression) = node.event_mappings.get(&variant_name) {
                    let result = eval_expression(
                        self.app_state.state(),
                        &self.message_string_to_enum_converter,
                        message_expression,
                        &|name| {
                            Ok(if let Some(field) = dyn_enum.field(name) {
                                Some(field.try_into()?)
                            } else {
                                None
                            })
                        },
                    )?;
                    (self.event_handler)(self.app_state.as_mut(), result.as_reflect());
                } else {
                    bail!("No event mapping found for event: {:?}", event);
                }
            }
        }
        Ok(())
    }

    pub fn register_node<T: Element>(&mut self) {
        // TODO: Check if node is already registered
        self.node_registry.register_node::<T>();
    }

    pub fn eval_expressions(&mut self) -> ViuiResult<()> {
        let mut binding_stack = BindingStack::new();
        self.eval_expressions_internal(self.root_item_idx, &mut binding_stack)
    }

    pub fn eval_expressions_internal(
        &mut self,
        item_idx: ItemIdx,
        binding_stack: &mut BindingStack,
    ) -> ViuiResult<()> {
        enum Todo {
            Item(ItemIdx),
            CloneItem {
                template_idx: ItemIdx,
                for_idx: ItemIdx,
            },
            PushBindings,
            PopBindings,
            SetBinding {
                name: String,
                value: ExpressionValue,
            },
        }
        let mut todos = vec![Todo::Item(item_idx)];
        while let Some(todo) = todos.pop() {
            match todo {
                Todo::Item(item_idx) => {
                    let item = &mut self.item_arena[&item_idx];
                    match &mut item.kind {
                        NodeItemKind::Node(node_idx) => {
                            let node = &mut self.node_arena[node_idx];
                            todos.extend(node.children.iter().map(|item| Todo::Item(*item)));
                            for expression in &node.prop_expressions {
                                let prop = node.props.reflect_path_mut(&*expression.field_name)?;
                                let app_state = self.app_state.state();
                                let value = eval_expression(
                                    app_state,
                                    &self.message_string_to_enum_converter,
                                    &expression.expression,
                                    &|name| Ok(binding_stack.get_binding(name)),
                                )?;
                                if let Some(prop) = prop.downcast_mut::<f32>() {
                                    let ExpressionValue::Float(value) = value else {
                                        bail!(
                                            "Expected float for property {}, but was: {}",
                                            expression.field_name,
                                            value
                                        );
                                    };
                                    *prop = value;
                                } else if let Some(prop) = prop.downcast_mut::<i32>() {
                                    let ExpressionValue::Float(value) = value else {
                                        bail!(
                                            "Expected number for property {}, but was: {}",
                                            expression.field_name,
                                            value
                                        );
                                    };
                                    *prop = value as i32;
                                } else if let Some(prop) = prop.downcast_mut::<String>() {
                                    let ExpressionValue::String(value) = value else {
                                        bail!(
                                            "Expected string for property {}, but was: {}",
                                            expression.field_name,
                                            value
                                        );
                                    };
                                    *prop = value;
                                } else {
                                    error!(
                                        "Unsupported property type for {}: {}",
                                        expression.field_name,
                                        prop.reflect_short_type_path()
                                    );
                                }
                            }
                        }
                        NodeItemKind::If(ref mut if_item) => {
                            let value = eval_expression(
                                self.app_state.state(),
                                &self.message_string_to_enum_converter,
                                &if_item.condition_expression,
                                &|_name| Ok(None),
                            )?;
                            let ExpressionValue::Bool(condition_value) = value else {
                                bail!("Condition must be a boolean, instead got {:?}", value);
                            };
                            if_item.condition = condition_value;
                            if condition_value {
                                todos.push(Todo::Item(if_item.then_item));
                            }
                        }
                        NodeItemKind::Block(block_item) => {
                            todos.extend(block_item.items.iter().map(|item| Todo::Item(*item)));
                        }
                        NodeItemKind::For(ref mut for_item) => {
                            let value = eval_expression(
                                self.app_state.state(),
                                &self.message_string_to_enum_converter,
                                &for_item.expression,
                                &|_name| Ok(None),
                            )?;
                            let ExpressionValue::Vec(values) = value else {
                                bail!("For expression must be a vector, instead got {:?}", value);
                            };
                            for (index, value) in values
                                .into_iter()
                                .zip_longest(for_item.items.iter())
                                .enumerate()
                            {
                                match value {
                                    EitherOrBoth::Both(value, item_idx) => {
                                        todos.push(Todo::PopBindings);
                                        todos.push(Todo::Item(*item_idx));
                                        todos.push(Todo::SetBinding {
                                            name: format!("{}#index", for_item.binding_name),
                                            value: ExpressionValue::Float(index as f32),
                                        });
                                        todos.push(Todo::SetBinding {
                                            name: for_item.binding_name.clone(),
                                            value,
                                        });
                                        todos.push(Todo::PushBindings);
                                    }
                                    EitherOrBoth::Left(value) => {
                                        //let item_idx = self.clone_item(for_item.item_template)?;
                                        todos.push(Todo::PopBindings);
                                        todos.push(Todo::CloneItem {
                                            template_idx: for_item.item_template,
                                            for_idx: item_idx,
                                        });
                                        todos.push(Todo::SetBinding {
                                            name: format!("{}#index", for_item.binding_name),
                                            value: ExpressionValue::Float(index as f32),
                                        });
                                        todos.push(Todo::SetBinding {
                                            name: for_item.binding_name.clone(),
                                            value,
                                        });
                                        todos.push(Todo::PushBindings);
                                    }
                                    EitherOrBoth::Right(_) => {
                                        // TODO: remove items
                                    }
                                }
                            }
                        }
                    }
                }
                Todo::CloneItem {
                    template_idx,
                    for_idx,
                } => {
                    let item_idx = self.clone_item(template_idx)?;
                    let item = &mut self.item_arena[&for_idx];
                    let NodeItemKind::For(ref mut for_item) = item.kind else {
                        bail!("Expected for item");
                    };
                    for_item.items.push(item_idx);
                }
                Todo::PushBindings => {
                    binding_stack.push();
                }
                Todo::PopBindings => {
                    binding_stack.pop();
                }
                Todo::SetBinding { name, value } => {
                    binding_stack.add_binding(name, value);
                }
            }
        }
        Ok(())
    }

    fn clone_item(&mut self, old_item_idx: ItemIdx) -> ViuiResult<ItemIdx> {
        let old_item = &self.item_arena[&old_item_idx];
        let mut new_item = old_item.clone();
        match &mut new_item.kind {
            NodeItemKind::Node(node_idx) => {
                *node_idx = self.clone_node(*node_idx)?;
            }
            NodeItemKind::If(if_item) => {
                if_item.then_item = self.clone_item(if_item.then_item)?;
                if let Some(else_item) = &mut if_item.else_item {
                    *else_item = self.clone_item(*else_item)?;
                }
            }
            NodeItemKind::Block(block_item) => {
                for item in &mut block_item.items {
                    *item = self.clone_item(*item)?;
                }
            }
            NodeItemKind::For(for_item) => {
                for_item.item_template = self.clone_item(for_item.item_template)?;
                for item in &mut for_item.items {
                    *item = self.clone_item(*item)?;
                }
            }
        }
        let new_item_idx = self.item_arena.insert(new_item);
        Ok(new_item_idx)
    }

    fn clone_node(&mut self, old_node_idx: NodeIdx) -> ViuiResult<NodeIdx> {
        let mut children = self.node_arena[&old_node_idx].children.clone();
        for child in &mut children {
            *child = self.clone_item(*child)?;
        }
        let old_node = &self.node_arena[&old_node_idx];
        let component = self.node_registry.get_node_by_kind(old_node.kind_index)?;
        let state = (component.make_state)()?;
        let props = (component.make_props)()?;
        let new_node = NodeData {
            tag: old_node.tag.clone(),
            kind_index: old_node.kind_index,
            state,
            props,
            layout: old_node.layout.clone(),
            prop_expressions: old_node.prop_expressions.clone(),
            event_mappings: old_node.event_mappings.clone(),
            children,
        };
        let new_node_idx = self.node_arena.insert(new_node);
        Ok(new_node_idx)
    }

    pub fn perform_layout(&mut self) -> ViuiResult<()> {
        let mut render_backends = take(&mut self.render_backends);
        for backend in &mut render_backends {
            let mut tree: TaffyTree<NodeIdx> = TaffyTree::new();
            let root_layout_node = tree.new_leaf_with_context(
                Style {
                    flex_direction: FlexDirection::Column,
                    size: taffy::Size {
                        width: length(backend.window_size.width),
                        height: length(backend.window_size.height),
                    },
                    ..Default::default()
                },
                self.root_node_idx,
            )?;
            let root_node = &self.node_arena[&self.root_node_idx];
            let mut todo: Vec<_> = root_node
                .children
                .iter()
                .map(|child_id| (root_layout_node, *child_id))
                .rev()
                .collect();
            let mut layout_context = LayoutContext::new(&mut self.image_pool);
            while let Some((parent_layout_id, item_idx)) = todo.pop() {
                let item = &self.item_arena[&item_idx];
                match &item.kind {
                    NodeItemKind::Node(node_idx) => {
                        let node = &mut self.node_arena[&node_idx];
                        let layout_contraints =
                            self.node_registry.layout_node(&mut layout_context, node)?;
                        let style = match layout_contraints {
                            LayoutConstraints::FixedLayout { width, height } => Some(Style {
                                size: taffy::Size {
                                    width: length(width),
                                    height: length(height),
                                },
                                ..Default::default()
                            }),
                            LayoutConstraints::HorizontalLayout {} => Some(Style {
                                flex_direction: FlexDirection::Row,
                                size: taffy::Size::auto(),
                                ..Default::default()
                            }),
                            LayoutConstraints::Passthrough => None,
                        };
                        let layout_id = if let Some(style) = style {
                            let child_id = tree.new_leaf_with_context(style, *node_idx)?;
                            tree.add_child(parent_layout_id, child_id)?;
                            child_id
                        } else {
                            parent_layout_id
                        };
                        for child in node.children.iter().rev() {
                            todo.push((layout_id, *child));
                        }
                    }
                    NodeItemKind::If(if_item) => {
                        if if_item.condition {
                            todo.push((parent_layout_id, if_item.then_item))
                        }
                    }
                    NodeItemKind::Block(block_item) => {
                        for child in block_item.items.iter().rev() {
                            todo.push((parent_layout_id, *child));
                        }
                    }
                    NodeItemKind::For(for_item) => {
                        for child in for_item.items.iter().rev() {
                            todo.push((parent_layout_id, *child));
                        }
                    }
                }
            }
            // Compute layout
            tree.compute_layout(root_layout_node, taffy::Size::max_content())?;

            // Set absolute position and bounds for each node
            let mut todo = vec![(0.0, 0.0, root_layout_node)];
            while let Some((parent_x, parent_y, node_id)) = todo.pop() {
                let node_index = tree.get_node_context(node_id).unwrap();
                let node = &mut self.node_arena[node_index];
                let layout = tree.layout(node_id)?;
                let x = parent_x + layout.location.x;
                let y = parent_y + layout.location.y;
                node.layout.bounds = Rect::new(
                    Point::new(x, y),
                    Size::new(layout.size.width, layout.size.height),
                );
                for child in tree.children(node_id)? {
                    todo.push((x, y, child));
                }
            }
        }
        self.render_backends = render_backends;
        Ok(())
    }

    fn make_render_commands(
        &mut self,
        backend: &mut RenderBackend,
    ) -> ViuiResult<Vec<RenderCommand>> {
        let time = self.start.elapsed().as_secs_f32();

        let animated_nodes = &mut self.animated_nodes;
        animated_nodes.clear();

        let mut initial_commands = vec![];
        let maximum_font_index = self.font_pool.maximum_font_index();
        if maximum_font_index > backend.maximum_font_index_loaded {
            for (font_index, font) in self
                .font_pool
                .get_fonts_from(backend.maximum_font_index_loaded)
            {
                initial_commands.push(RenderCommand::LoadFont {
                    font_idx: font_index,
                    resource: font.resource().clone(),
                });
            }
            backend.maximum_font_index_loaded = maximum_font_index;
        }
        let mut render_context =
            RenderContext::new(&mut self.image_pool, &mut self.font_pool, time)?;
        render_context.add_commands(initial_commands);
        render_context.add_command(RenderCommand::SetFont {
            font_idx: self.styling.font_face,
        });

        render_context.add_command(RenderCommand::SetFillColor(self.styling.background_color));
        render_context.add_command(RenderCommand::SetWindowSize {
            size: backend.window_size,
        });
        render_context.add_command(RenderCommand::FillRect {
            rect: Rect::new(Point::new(0.0, 0.0), backend.window_size),
        });
        let render_parameters = RenderParameters::new(&self.styling)?;
        let mut todo = vec![self.root_item_idx];
        while let Some(item_idx) = todo.pop() {
            let item = &self.item_arena[&item_idx];
            match &item.kind {
                NodeItemKind::Node(node_idx) => {
                    let node = &self.node_arena[&node_idx];
                    render_context.add_command(RenderCommand::Save);
                    render_context.add_command(RenderCommand::Translate {
                        x: node.layout.bounds.origin.x,
                        y: node.layout.bounds.origin.y,
                    });
                    render_context.add_command(RenderCommand::ClipRect(Rect::new(
                        Point::new(0.0, 0.0),
                        node.layout.bounds.size,
                    )));
                    render_context.set_active(self.active_nodes.contains(node_idx));
                    self.node_registry.render_node(
                        &mut render_context,
                        &render_parameters,
                        node,
                    )?;
                    if render_context.reset_animated() {
                        animated_nodes.push(*node_idx);
                    }
                    render_context.add_command(RenderCommand::Restore);
                    todo.extend(node.children.iter());
                }
                NodeItemKind::If(if_item) => {
                    if if_item.condition {
                        todo.push(if_item.then_item);
                    }
                }
                NodeItemKind::Block(block_item) => todo.extend(block_item.items.iter()),
                NodeItemKind::For(for_item) => todo.extend(for_item.items.iter()),
            }
        }
        Ok(render_context.render_queue())
    }

    pub fn set_node_prop(
        &mut self,
        node_index: &NodeIdx,
        field_name: &str,
        expression: ExpressionAst,
    ) {
        self.node_arena[node_index]
            .prop_expressions
            .push(PropExpression {
                field_name: field_name.to_string(),
                expression,
            });
    }

    pub fn set_event_mapping(&mut self, node_index: &NodeIdx, event: &str, message: ExpressionAst) {
        self.node_arena
            .index_mut(node_index)
            .event_mappings
            .insert(event.to_string(), message);
    }
    pub fn set_root_node_file<P: AsRef<Path>>(&mut self, root_path: P) -> ViuiResult<()> {
        context!("set root context file {:?}", self.root_node_file => {
            self.root_node_file = root_path.as_ref().to_path_buf();
            self.load_root_node_file()?;

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            self.file_watcher.watcher().watch(&self.root_node_file, RecursiveMode::Recursive)?;
            Ok(())
        })
    }

    fn load_root_node_file(&mut self) -> ViuiResult<()> {
        context!("load root context file {:?}", self.root_node_file => {
            let mut string = String::new();
            File::open(&self.root_node_file)?.read_to_string(&mut string)?;
            let ast = parse_ui(&string)?;
            let ast_data = ast.into_data();
            for component in &ast_data.components {
                self.register_component_node(component);
            }
            self.set_root_node()?;
            Ok(())
        })
    }

    pub fn set_root_node(&mut self) -> ViuiResult<()> {
        self.node_arena.clear();
        self.item_arena.clear();
        self.root_node_idx = self.create_node(&self.root_component_name.to_string())?;
        self.root_item_idx = self.item_arena.insert(NodeItem {
            kind: NodeItemKind::Node(self.root_node_idx),
        });
        Ok(())
    }

    pub fn create_node(&mut self, name: &str) -> ViuiResult<NodeIdx> {
        let component = self.node_registry.get_node_by_name(name)?;
        let kind_index = component.kind_index;
        let mut children = vec![];
        for child in &component.children.clone() {
            children.push(self.create_children(child)?);
        }
        let component = self.node_registry.get_node_by_name(name)?;
        Ok(self.node_arena.insert(NodeData {
            tag: name.to_string(),
            kind_index,
            state: (component.make_state)()?,
            props: (component.make_props)()?,
            layout: LayoutInfo::default(),
            prop_expressions: Vec::new(),
            event_mappings: Default::default(),
            children,
        }))
    }

    fn create_children(&mut self, child: &ItemAst) -> ViuiResult<ItemIdx> {
        Ok(match child.data() {
            ItemDefinition::Node { node } => {
                let child = node.data();
                let node_idx = self.create_node(&child.tag)?;
                for prop in &child.props {
                    self.set_node_prop(&node_idx, &prop.name, prop.expression.clone());
                }
                for event in &child.events {
                    self.set_event_mapping(&node_idx, &event.name, event.expression.clone());
                }
                let mut children = vec![];
                for child in &child.children {
                    children.push(self.create_children(child)?);
                }
                self.add_children(&node_idx, children);
                let item_idx = self.item_arena.insert(NodeItem {
                    kind: NodeItemKind::Node(node_idx),
                });
                item_idx
            }
            ItemDefinition::If(if_item) => {
                let item = NodeItem {
                    kind: NodeItemKind::If(IfItem {
                        condition_expression: if_item.condition.clone(),
                        condition: true,
                        then_item: self.create_children(&if_item.then_item)?,
                        else_item: if_item
                            .else_item
                            .as_ref()
                            .map(|item| self.create_children(item))
                            .transpose()?,
                    }),
                };
                let item_idx = self.item_arena.insert(item);
                item_idx
            }
            ItemDefinition::Block { items } => {
                let item = NodeItem {
                    kind: NodeItemKind::Block(BlockItem {
                        items: items
                            .iter()
                            .map(|item| self.create_children(item))
                            .collect::<ViuiResult<Vec<ItemIdx>>>()?,
                    }),
                };
                let item_idx = self.item_arena.insert(item);
                item_idx
            }
            ItemDefinition::For(for_item) => {
                let item = NodeItem {
                    kind: NodeItemKind::For(ForItem {
                        binding_name: for_item.binding_name.clone(),
                        expression: for_item.expression.clone(),
                        item_template: self.create_children(&for_item.each_item)?,
                        items: vec![],
                    }),
                };
                let item_idx = self.item_arena.insert(item);
                item_idx
            }
        })
    }

    fn register_component_node(&mut self, component_ast: &ComponentAst) {
        self.node_registry.register(
            &component_ast.name,
            || Ok(Box::new(())),
            || Ok(Box::new(())),
            |_, _, _| Ok(()),
            |_, _, _| Ok(()),
            |_, _| Ok(LayoutConstraints::Passthrough {}),
            component_ast.children.clone(),
        )
    }

    fn add_children(&mut self, parent: &NodeIdx, children: Vec<ItemIdx>) {
        self.node_arena.index_mut(parent).children.extend(children);
    }
}

fn reflect_to_value(value: &dyn Reflect) -> ViuiResult<ExpressionValue> {
    match value.reflect_ref() {
        ReflectRef::List(list) => {
            let values = list
                .iter()
                .map(|value| reflect_to_value(value))
                .collect::<ViuiResult<Vec<_>>>()?;
            return Ok(ExpressionValue::Vec(values));
        }
        _ => {}
    }
    if let Some(value) = value.downcast_ref::<f32>() {
        Ok(ExpressionValue::Float(*value))
    } else if let Some(value) = value.downcast_ref::<i32>() {
        Ok(ExpressionValue::Float(*value as f32))
    } else if let Some(value) = value.downcast_ref::<String>() {
        Ok(ExpressionValue::String(value.clone()))
    } else if let Some(value) = value.downcast_ref::<bool>() {
        Ok(ExpressionValue::Bool(*value))
    } else {
        bail!(
            "Unsupported property type: {}",
            value.reflect_short_type_path()
        );
    }
}

fn eval_expression(
    app_state: &dyn Reflect,
    converter: &MessageStringToEnumConverter,
    expression: &ExpressionAst,
    lookup: &dyn Fn(&str) -> ViuiResult<Option<ExpressionValue>>,
) -> ViuiResult<ExpressionValue> {
    let value = eval(expression, &|name| {
        if let Some(value) = lookup(name)? {
            return Ok(value);
        }
        if let Ok(value) = app_state.reflect_path(name) {
            reflect_to_value(value)
        } else {
            converter(name)
        }
    })?;
    Ok(value)
}
