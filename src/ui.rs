use crate::arenal::{Arenal, Idx};
use crate::bail;
use crate::component::ast::{ComponentAst, ExpressionAst, NodeAst};
use crate::component::eval::eval;
use crate::component::parser::parse_ui;
use crate::component::value::ExpressionValue;
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
    root_node_idx: Idx<NodeData>,
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
    active_nodes: Vec<Idx<NodeData>>,
    animated_nodes: Vec<Idx<NodeData>>,
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
        let mut add_event_trigger = |node_idx: Idx<NodeData>, node_event: InputEvent| {
            events_to_trigger.push((node_idx, node_event));
        };
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
                    add_event_trigger(*node, InputEvent::key_input(key_input.key.clone()));
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
        for node in self.node_arena.entries_mut() {
            for expression in &node.prop_expressions {
                let prop = node.props.reflect_path_mut(&*expression.field_name)?;
                let app_state = self.app_state.state();
                let value = eval_expression(
                    app_state,
                    &self.message_string_to_enum_converter,
                    &expression.expression,
                    &|_name| Ok(None),
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
        Ok(())
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
            let mut todo: Vec<_> = self.node_arena[&self.root_node_idx]
                .children
                .iter()
                .map(|child_id| (root_layout_node, *child_id))
                .rev()
                .collect();
            let mut layout_context = LayoutContext::new(&mut self.image_pool);
            while let Some((parent_layout_id, node_idx)) = todo.pop() {
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
                    let child_id = tree.new_leaf_with_context(style, node_idx)?;
                    tree.add_child(parent_layout_id, child_id)?;
                    child_id
                } else {
                    parent_layout_id
                };
                for child in node.children.iter().rev() {
                    todo.push((layout_id, *child));
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
        let mut todo = vec![self.root_node_idx];
        while let Some(node_idx) = todo.pop() {
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

            self.node_registry
                .render_node(&mut render_context, &render_parameters, node)?;
            if render_context.reset_animated() {
                animated_nodes.push(node_idx);
            }
            render_context.add_command(RenderCommand::Restore);
            todo.extend(node.children.iter());
        }
        Ok(render_context.render_queue())
    }

    pub fn set_node_prop(
        &mut self,
        node_index: &Idx<NodeData>,
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

    pub fn set_event_mapping(
        &mut self,
        node_index: &Idx<NodeData>,
        event: &str,
        message: ExpressionAst,
    ) {
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
        self.root_node_idx = self.create_node(&self.root_component_name.to_string())?;
        Ok(())
    }

    pub fn create_node(&mut self, name: &str) -> ViuiResult<Idx<NodeData>> {
        let component = self.node_registry.get_node_by_name(name)?;
        let kind_index = component.kind_index;
        let mut children = vec![];
        for child in &component.children.clone() {
            let node_idx = self.create_child(child)?;
            children.push(node_idx);
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

    fn create_child(&mut self, child: &NodeAst) -> ViuiResult<Idx<NodeData>> {
        let node_idx = self.create_node(&child.tag)?;
        for prop in &child.props {
            self.set_node_prop(&node_idx, &prop.name, prop.expression.clone());
        }
        for event in &child.events {
            self.set_event_mapping(&node_idx, &event.name, event.expression.clone());
        }
        let mut children = vec![];
        for child in &child.children {
            children.push(self.create_child(child)?)
        }
        self.add_children(&node_idx, children);
        Ok(node_idx)
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

    #[allow(dead_code)]
    fn set_children(&mut self, parent: &Idx<NodeData>, children: Vec<Idx<NodeData>>) {
        self.node_arena.index_mut(parent).children = children;
    }

    fn add_children(&mut self, parent: &Idx<NodeData>, children: Vec<Idx<NodeData>>) {
        self.node_arena.index_mut(parent).children.extend(children);
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
            if let Some(value) = value.downcast_ref::<f32>() {
                Ok(ExpressionValue::Float(*value))
            } else if let Some(value) = value.downcast_ref::<i32>() {
                Ok(ExpressionValue::Float(*value as f32))
            } else if let Some(value) = value.downcast_ref::<String>() {
                Ok(ExpressionValue::String(value.clone()))
            } else {
                bail!(
                    "Unsupported property type for {}: {}",
                    name,
                    value.reflect_short_type_path()
                );
            }
        } else {
            converter(name)
        }
    })?;
    Ok(value)
}
