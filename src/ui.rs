use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::mem::take;
use std::ops::{Index, IndexMut};
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use bevy_reflect::{DynamicEnum, DynamicVariant, FromReflect, GetPath, Reflect};
use regex::Regex;
use UiEventKind::MouseMoved;
use crate::arenal::{Arenal, Idx};
use crate::model::ComponentNode;
use crate::observable_state::ObservableState;
use crate::render::command::RenderCommand;
use crate::types::{Point, Rect, Size};
use crate::widget_model::{ButtonWidgetProps, ButtonWidgetState, Text, TextPart, Widget, WidgetEventHandler, WidgetProps, WidgetRegistry, WidgetState};

pub type StateBox = Box<dyn WidgetState>;
pub type PropsBox = Box<dyn WidgetProps>;


#[derive(Clone, Debug, Default)]
pub struct LayoutInfo {
    bounds: Rect,
}

pub struct WidgetData {
    kind_index: usize,
    //props_type_id: TypeId,
    layout: LayoutInfo,
    state: StateBox,
    props: PropsBox,
    prop_expressions: Vec<PropExpression>,
    event_mappings: HashMap<String, Box<dyn Reflect>>,
}

pub struct PropExpression {
    pub field_name: String,
    pub text: Text,
}

impl WidgetData {
    pub fn props(&self) -> &dyn WidgetProps {
        self.props.as_ref()
    }

    pub fn cast_props<T: 'static>(&self) -> &T {
        self.props.as_any().downcast_ref::<T>().unwrap()
    }
    pub fn cast_state<T: 'static>(&self) -> &T {
        self.state.as_any().downcast_ref::<T>().unwrap()
    }
    pub fn cast_state_mut<T: 'static>(&mut self) -> &mut T {
        self.state.as_any_mut().downcast_mut::<T>().unwrap()
    }

    pub fn cast_state_mut_and_props<S: 'static, P: 'static>(&mut self) -> (&mut S, &P) {
        (self.state.as_any_mut().downcast_mut::<S>().unwrap(), self.props.as_any().downcast_ref::<P>().unwrap())
    }

    pub fn cast_state_and_props<S: 'static, P: 'static>(&self) -> (&S, &P) {
        (self.state.as_any().downcast_ref::<S>().unwrap(), self.props.as_any().downcast_ref::<P>().unwrap())
    }

    pub fn set_bounds(&mut self, bounds: Rect) {
        self.layout.bounds = bounds;
    }

    pub fn bounds(&self) -> &Rect {
        &self.layout.bounds
    }

    pub fn kind_index(&self) -> usize {
        self.kind_index
    }
}

pub struct UI {
    widget_registry: WidgetRegistry,
    state_arena: Arenal<WidgetData>,
    hovered_widget: Option<Idx<WidgetData>>,
    app_state: Box<ObservableState>,
    event_handler: Box<dyn Fn(&mut ObservableState, &dyn Reflect) + Send>,
    message_string_to_enum_converter: Box<dyn Fn(&str) -> Box<dyn Reflect> + Send>,
    mouse_position: Point,
    render_backends: Vec<RenderBackend>,
    event_receiver: Receiver<UiEvent>,
    event_sender: Sender<UiEvent>,
}

struct RenderBackend {
    render_backend_sender: Sender<RenderBackendMessage>,
}

pub struct RenderBackendMessage {
    pub(crate) render_commands: Vec<RenderCommand>,
}

impl UI {
    pub fn new<MESSAGE: Reflect + FromReflect + Debug + Sized>(state: ObservableState, event_handler: impl Fn(&mut ObservableState, &MESSAGE) + Send + 'static) -> UI {
        let (event_sender, event_receiver) = mpsc::channel::<UiEvent>();
        let message_string_to_enum_converter = Box::new(|message_string: &str| -> Box<dyn Reflect> {
            let dynamic_enum = DynamicEnum::new(message_string.to_string(), DynamicVariant::Unit);
            let message = MESSAGE::from_reflect(&dynamic_enum).unwrap();
            Box::new(message)
        });

        UI {
            widget_registry: WidgetRegistry::new(),
            state_arena: Arenal::new(),
            hovered_widget: None,
            app_state: Box::new(state),
            event_handler: Box::new(move |state, message| {
                let typed_message = message.downcast_ref::<MESSAGE>().unwrap();
                event_handler(state, typed_message);
            }),
            mouse_position: Default::default(),
            render_backends: Vec::new(),
            event_receiver,
            event_sender,
            message_string_to_enum_converter,
        }
    }

    pub fn start(mut self){
        thread::Builder::new()
         .name("VIUI Thread".into()).spawn(move || {
            loop {
                let event = self.event_receiver.recv().unwrap();
                self.handle_ui_event(event);
                self.redraw();
            }
        }).unwrap();
    }

    pub fn redraw(&mut self){
        self.eval_expressions();
        self.perform_layout();
        let render_backends = take(&mut self.render_backends);
        for backend in &render_backends {
            let render_commands = self.make_render_commands();
            backend.render_backend_sender.send(RenderBackendMessage {
                render_commands,
            }).unwrap();
        }
        self.render_backends = render_backends;
    }



    pub fn event_sender(&self) -> Sender<UiEvent> {
        self.event_sender.clone()
    }

    pub fn add_render_backend(&mut self) -> Receiver<RenderBackendMessage> {
        let (render_backend_sender, message_receiver) = mpsc::channel::<RenderBackendMessage>();
        self.render_backends.push(RenderBackend {
            render_backend_sender
        });
        self.redraw();
        message_receiver
    }


    pub fn add_widget2(&mut self, kind: &str) -> Idx<WidgetData> {
        let widget_descriptor = self.widget_registry.get_widget_by_name(kind);
        self.state_arena.insert(WidgetData {
            kind_index: widget_descriptor.kind_index,
//            props_type_id: TypeId::of::<P>(),
            state: (widget_descriptor.make_state)(),
            props: (widget_descriptor.make_props)(),
            layout: LayoutInfo::default(),
            prop_expressions: Vec::new(),
            event_mappings: Default::default(),
        })
    }

    pub fn add_widget<S: WidgetState, P: WidgetProps>(&mut self, kind: &str, state: S, props: P) -> Idx<WidgetData> {
        let kind_index = self.widget_registry.get_widget_index(kind);
        self.state_arena.insert(WidgetData {
            kind_index,
//            props_type_id: TypeId::of::<P>(),
            state: Box::new(state),
            props: Box::new(props),
            layout: LayoutInfo::default(),
            prop_expressions: Vec::new(),
            event_mappings: Default::default(),
        })
    }

    pub fn widgets(&mut self) -> impl Iterator<Item=&mut WidgetData> {
        self.state_arena.entries_mut()
    }

    pub fn handle_ui_event(&mut self, event: UiEvent) {
        match event.kind {
            MouseMoved(position) => {
                self.mouse_position = position;
                for widget in self.state_arena.entries_mut() {
                    self.widget_registry.handle_event(widget.kind_index, WidgetEvent::mouse_out(), widget);
                    if widget.layout.bounds.contains(position) {
                        self.widget_registry.handle_event(widget.kind_index, WidgetEvent::mouse_over(), widget);
                        //                        break;
                    }
                }
            }
            UiEventKind::MouseInput(input) => {
                let position = self.mouse_position;
                for widget in self.state_arena.entries_mut() {
                    if widget.layout.bounds.contains(position) {
                        if input.mouse_event_kind == MouseEventKind::Pressed {
                            self.widget_registry.handle_event(widget.kind_index, WidgetEvent::mouse_press(), widget);
                            // Found clicked widget
                            if let Some(message) = widget.event_mappings.get("click") {
                                (self.event_handler)(self.app_state.as_mut(), message.as_ref());
                            }
                        } else if input.mouse_event_kind == MouseEventKind::Released {
                            self.widget_registry.handle_event(widget.kind_index, WidgetEvent::mouse_release(), widget);
                        }
                    }
                }
            }
        }
    }

    pub fn register_widget<T: Widget>(&mut self) {
        self.widget_registry.register_widget::<T>(vec!["click".to_string()]);
    }

    pub fn eval_expressions(&mut self) {
        for widget in self.state_arena.entries_mut() {
            for expression in &widget.prop_expressions {
                let string = text_to_string(self.app_state.as_ref(), &expression.text.parts);
                widget.props.reflect_path_mut(&*expression.field_name).unwrap().apply(&string);
            }
        }
    }


    pub fn perform_layout(&mut self) {
        let widget_height = 40.0;
        let widget_width = 200.0;
        let mut current_y = 0.0f32;
        for widget in self.state_arena.entries_mut() {
            widget.layout.bounds = Rect::new(Point::new(0.0, current_y), Size::new(widget_width, widget_height));
            current_y += widget_height;
        }
    }

    pub fn make_render_commands(&self) -> Vec<RenderCommand> {
        let mut render_commands: Vec<RenderCommand> = Vec::new();
        for widget in self.state_arena.entries() {
            render_commands.push(RenderCommand::Save);
            self.widget_registry.render_widget(&mut render_commands, widget);
            render_commands.push(RenderCommand::Restore);
            render_commands.push(RenderCommand::Translate { x: 0.0, y: 40.0 })
        }
        render_commands
    }

    pub fn set_widget_prop(&mut self, widget_index: &Idx<WidgetData>, field_name: &str, text: Text) {
        self.state_arena[&widget_index].prop_expressions.push(PropExpression {
            field_name: field_name.to_string(),
            text,
        });
    }

    pub fn set_event_mapping<T: Reflect>(&mut self, widget_index: &Idx<WidgetData>, event: &str, message: T) {
        self.state_arena.index_mut(&widget_index).event_mappings.insert(event.to_string(), Box::new(message));
    }
    pub fn set_event_mapping_boxed(&mut self, widget_index: &Idx<WidgetData>, event: &str, message: Box<dyn Reflect>) {
        self.state_arena.index_mut(&widget_index).event_mappings.insert(event.to_string(), message);
    }


    pub fn set_root_node_file<P: AsRef<Path>>(&mut self, root_path: P) {
        let model: ComponentNode = serde_yml::from_reader(File::open(&root_path).unwrap()).unwrap();
        self.set_root_node(model);

    }

    pub fn set_root_node(&mut self, root: ComponentNode) {
        for child in root.children {
            let widget_idx = self.add_widget2(&child.kind);
            for (prop, expression) in child.props {
                self.set_widget_prop(&widget_idx, &prop, expression_to_text(&expression));
            }
            for (event_name, message_name) in child.events {
                self.set_event_mapping_boxed(&widget_idx, &event_name, (self.message_string_to_enum_converter)(&message_name));
            }
        }
    }

}

fn expression_to_text(original_expression: &str) -> Text {
    let mut parts =  vec![];
    let string_regex = Regex::new(r#"^([^$]*)"#).unwrap();
    let placeholder_regex = Regex::new(r#"^\$\{([^}]+)}"#).unwrap();
    let mut matched = true;
    let mut expression = original_expression;
    while !expression.is_empty() {
        // TODO: error handling
        if !matched {
            panic!("Failed to parse placeholder expression: '{}' at '{}'", original_expression, expression);
        }
        matched = false;
        if let Some(found) = string_regex.find(expression) {
            parts.push(TextPart::FixedText(found.as_str().to_string()));
            expression = &expression[found.end()..];
            matched = true;
        }
        if let Some(found) = placeholder_regex.find(expression) {
            parts.push(TextPart::VariableText(expression[found.start()+2..found.end()-1].to_string()));
            expression = &expression[found.end()..];
            matched = true;
        }

    }
    Text {
        parts,
    }
}

fn text_to_string(app_state: &ObservableState, text: &Vec<TextPart>) -> String {
    let mut string = "".to_string();
    for part in text {
        match part {
            TextPart::FixedText(fixed_string) => {
                string.push_str(fixed_string);
            }
            TextPart::VariableText(path) => {
                string.push_str(&format!("{:?}", app_state.state().reflect_path(&**path).unwrap()));
            }
        }
    }
    string
}


#[derive(Debug)]
pub struct WidgetEvent {
    kind: WidgetEventKind,
}

#[derive(Debug)]
pub enum WidgetEventKind {
    MouseOver,
    MouseOut,
    MousePress,
    MouseRelease,
}

impl WidgetEvent {
    pub fn mouse_over() -> Self {
        Self {
            kind: WidgetEventKind::MouseOver,
        }
    }
    pub fn mouse_out() -> Self {
        Self {
            kind: WidgetEventKind::MouseOut,
        }
    }
    pub fn mouse_press() -> Self {
        Self {
            kind: WidgetEventKind::MousePress,
        }
    }
    pub fn mouse_release() -> Self {
        Self {
            kind: WidgetEventKind::MouseRelease,
        }
    }

    pub fn kind(&self) -> &WidgetEventKind {
        &self.kind
    }
}

#[derive(Debug)]
pub struct UiEvent {
    kind: UiEventKind,
}

#[derive(Debug)]
pub enum UiEventKind {
    MouseMoved(Point),
    MouseInput(MouseInput),
}


#[derive(Debug)]
pub struct MouseInput {
    pub mouse_event_kind: MouseEventKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MouseEventKind {
    Pressed,
    Released,
}


impl UiEvent {
    pub fn mouse_move(position: Point) -> Self {
        Self {
            kind: MouseMoved(position),
        }
    }
    pub fn mouse_input(mouse_event_kind: MouseEventKind) -> Self {
        Self {
            kind: UiEventKind::MouseInput(MouseInput {
                mouse_event_kind,
            }, )
        }
    }
}