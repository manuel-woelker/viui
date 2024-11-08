use crate::arenal::{Arenal, Idx};
use crate::bail;
use crate::component::ast::ExpressionAst;
use crate::component::eval::eval;
use crate::component::parser::parse_expression;
use crate::component::value::ExpressionValue;
use crate::model::ComponentNode;
use crate::nodes::data::{LayoutInfo, NodeData, PropExpression};
use crate::nodes::elements::button::ButtonElement;
use crate::nodes::elements::kind::Element;
use crate::nodes::elements::knob::KnobElement;
use crate::nodes::elements::label::LabelElement;
use crate::nodes::events::{MouseEventKind, NodeEvent, UiEvent, UiEventKind};
use crate::nodes::registry::NodeRegistry;
use crate::observable_state::ObservableState;
use crate::render::command::RenderCommand;
use crate::result::{context, ViuiResult};
use crate::types::{Point, Rect, Size};
use bevy_reflect::{FromReflect, GetPath, Reflect};
use crossbeam_channel::{select, Receiver, Sender};
use log::debug;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::fs::File;
use std::mem::take;
use std::ops::IndexMut;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tracing::error;

pub type ApplicationEventHandler = Box<dyn Fn(&mut ObservableState, &dyn Reflect) + Send>;
pub type MessageStringToEnumConverter = Box<dyn Fn(&str) -> ViuiResult<Box<dyn Reflect>> + Send>;

pub struct UI {
    node_registry: NodeRegistry,
    node_arena: Arenal<NodeData>,
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
    active_node: Option<Idx<NodeData>>,
}

struct RenderBackend {
    render_backend_sender: Sender<RenderBackendMessage>,
}

pub struct RenderBackendMessage {
    pub(crate) render_commands: Vec<RenderCommand>,
}

impl UI {
    pub fn new<MESSAGE: DeserializeOwned + Reflect + FromReflect + Debug + Sized>(
        state: ObservableState,
        event_handler: impl Fn(&mut ObservableState, &MESSAGE) + Send + 'static,
    ) -> ViuiResult<UI> {
        let (event_sender, event_receiver) = crossbeam_channel::bounded::<UiEvent>(4);
        let (file_change_sender, file_change_receiver) = crossbeam_channel::bounded::<()>(4);
        let message_string_to_enum_converter =
            Box::new(|message_string: &str| -> ViuiResult<Box<dyn Reflect>> {
                let message = ron::de::from_str::<MESSAGE>(message_string)?;
                Ok(Box::new(message))
            });
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
        node_registry.register_node::<LabelElement>(vec![]);
        node_registry.register_node::<ButtonElement>(vec!["click".to_string()]);
        node_registry.register_node::<KnobElement>(vec!["click".to_string()]);
        Ok(UI {
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
            active_node: Default::default(),
        })
    }

    pub fn start(mut self) -> ViuiResult<()> {
        thread::Builder::new()
            .name("VIUI Thread".into())
            .spawn(move || {
                debug!("Running main loop");

                loop {
                    let result: ViuiResult<()> = (|| {
                        select! {
                        recv(self.file_change_receiver) -> _event => {
                            self.load_root_node_file()?;
                            self.redraw()?;
                        }
                        recv(self.ui_event_receiver) -> event => {
                            self.handle_ui_event(event?)?;
                            self.redraw()?;
                        }
                        };
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

    pub fn redraw(&mut self) -> ViuiResult<()> {
        self.eval_expressions()?;
        self.perform_layout();
        let render_backends = take(&mut self.render_backends);
        for backend in &render_backends {
            let render_commands = self.make_render_commands()?;
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

    pub fn add_render_backend(&mut self) -> ViuiResult<Receiver<RenderBackendMessage>> {
        let (render_backend_sender, message_receiver) =
            crossbeam_channel::bounded::<RenderBackendMessage>(4);
        self.render_backends.push(RenderBackend {
            render_backend_sender,
        });
        self.redraw()?;
        Ok(message_receiver)
    }

    pub fn add_node(&mut self, kind: &str) -> ViuiResult<Idx<NodeData>> {
        let node_descriptor = self.node_registry.get_node_by_name(kind);
        Ok(self.node_arena.insert(NodeData {
            kind_index: node_descriptor.kind_index,
            state: (node_descriptor.make_state)()?,
            props: (node_descriptor.make_props)()?,
            layout: LayoutInfo::default(),
            prop_expressions: Vec::new(),
            event_mappings: Default::default(),
        }))
    }

    pub fn nodes(&mut self) -> impl Iterator<Item = &mut NodeData> {
        self.node_arena.entries_mut()
    }

    pub fn handle_ui_event(&mut self, event: UiEvent) -> ViuiResult<()> {
        let mut events_to_trigger = Vec::new();
        let mut add_event_trigger = |node_idx: Idx<NodeData>, node_event: NodeEvent| {
            events_to_trigger.push((node_idx, node_event));
        };
        match event.kind {
            UiEventKind::MouseMoved(position) => {
                self.mouse_position = position;
                if let Some(node) = &self.active_node {
                    add_event_trigger(*node, NodeEvent::mouse_move(position));
                }
                for (node, node_idx) in self.node_arena.entries_mut_indexed() {
                    if node.layout.bounds.contains(position) {
                        add_event_trigger(node_idx, NodeEvent::mouse_over());
                    } else {
                        add_event_trigger(node_idx, NodeEvent::mouse_out());
                    }
                }
            }
            UiEventKind::MouseInput(input) => {
                let position = self.mouse_position;
                for (node, idx) in self.node_arena.entries_mut_indexed() {
                    if node.layout.bounds.contains(position) {
                        self.active_node = Some(idx);
                        if input.mouse_event_kind == MouseEventKind::Pressed {
                            add_event_trigger(idx, NodeEvent::mouse_press(position));
                        } else if input.mouse_event_kind == MouseEventKind::Released {
                            add_event_trigger(idx, NodeEvent::mouse_release(position));
                        }
                    }
                }
            }
        }

        for (node_idx, event) in events_to_trigger {
            let node = &mut self.node_arena[&node_idx];
            let mut events = Vec::new();
            let mut event_trigger = |event: &str| {
                events.push(event.to_string());
            };
            self.node_registry
                .handle_event(node.kind_index, event, node, &mut event_trigger)?;
            for event in events {
                let (event_name, value) = event.split_once(":").unwrap_or((&event, ""));
                if let Some(message_expression) = node.event_mappings.get(event_name) {
                    let message_expression = message_expression.replace("${value}", value);
                    let message = (self.message_string_to_enum_converter)(&message_expression)?;
                    (self.event_handler)(self.app_state.as_mut(), message.as_ref());
                } else {
                    bail!("No event mapping found for event: {}", event);
                }
            }
            /*            // Found clicked node
            if let Some(message) = node.event_mappings.get(event) {
                (self.event_handler)(self.app_state.as_mut(), message.as_ref());
            } else {
                bail!("No event mapping found for event: {}", event);
            }*/
        }
        Ok(())
    }

    pub fn register_node<T: Element>(&mut self) {
        // TODO: fix event registration
        // TODO: Check if node is already registered
        self.node_registry
            .register_node::<T>(vec!["click".to_string(), "change".to_string()]);
    }

    pub fn eval_expressions(&mut self) -> ViuiResult<()> {
        for node in self.node_arena.entries_mut() {
            for expression in &node.prop_expressions {
                let prop = node.props.reflect_path_mut(&*expression.field_name)?;
                let app_state = self.app_state.state();
                let value = eval(&expression.expression, &|name| {
                    let value = app_state.reflect_path(name)?;
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
                })?;
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

    pub fn perform_layout(&mut self) {
        let node_height = 80.0;
        let node_width = 200.0;
        let mut current_y = 0.0f32;
        for node in self.node_arena.entries_mut() {
            node.layout.bounds = Rect::new(
                Point::new(0.0, current_y),
                Size::new(node_width, node_height),
            );
            current_y += node_height;
        }
    }

    pub fn make_render_commands(&self) -> ViuiResult<Vec<RenderCommand>> {
        let mut render_commands: Vec<RenderCommand> = Vec::new();
        for node in self.node_arena.entries() {
            render_commands.push(RenderCommand::Save);
            self.node_registry.render_node(&mut render_commands, node)?;
            render_commands.push(RenderCommand::Restore);
            render_commands.push(RenderCommand::Translate { x: 0.0, y: 80.0 })
        }
        Ok(render_commands)
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

    pub fn set_event_mapping(&mut self, node_index: &Idx<NodeData>, event: &str, message: String) {
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
            let model: ComponentNode = serde_yml::from_reader(File::open(&self.root_node_file)?)?;
            self.set_root_node(model)?;
            Ok(())
        })
    }

    pub fn set_root_node(&mut self, root: ComponentNode) -> ViuiResult<()> {
        self.node_arena.clear();
        for child in root.children {
            let node_idx = self.add_node(&child.kind)?;
            for (prop, expression) in child.props {
                self.set_node_prop(&node_idx, &prop, parse_expression(&expression)?);
            }
            for (event_name, message_expression) in child.events {
                self.set_event_mapping(&node_idx, &event_name, message_expression);
            }
        }
        Ok(())
    }
}
