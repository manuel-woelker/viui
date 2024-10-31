use std::any::Any;
use std::collections::HashMap;
use bevy_reflect::{Reflect, Typed};
use default_boxed::DefaultBoxed;
use femtovg::Canvas;
use crate::ui::{WidgetData, WidgetEvent, WidgetEventKind};

pub struct WidgetModel {
    pub widgets: Vec<Box<dyn WidgetProps>>,
}

impl WidgetModel {

}

pub trait WidgetState: Reflect + 'static {

}

pub trait WidgetProps: Reflect + 'static {

}

pub type WidgetEventHandler = Box<dyn Fn(WidgetEvent, &mut WidgetData)>;



pub struct WidgetDescriptor {
    make_state: Box<dyn Fn() -> Box<dyn WidgetState>>,
    make_props: Box<dyn Fn() -> Box<dyn WidgetProps>>,
    event_handler: WidgetEventHandler,
}

pub struct WidgetRegistry {
    pub widgets: HashMap<String, WidgetDescriptor>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: impl Into<String>, make_state: impl Fn() -> Box<dyn WidgetState> + 'static, make_props: impl Fn() -> Box<dyn WidgetProps> + 'static, event_handler: impl Fn(WidgetEvent, &mut WidgetData) + 'static) {
        self.register_internal(name.into(), Box::new(make_state), Box::new(make_props), Box::new(event_handler));
    }

    fn register_internal(&mut self, name: String, make_state: Box<dyn Fn() -> Box<dyn WidgetState>>, make_props: Box<dyn Fn() -> Box<dyn WidgetProps>>, event_handler: WidgetEventHandler) {
        self.widgets.insert(name, WidgetDescriptor { event_handler, make_state, make_props });
    }

    pub fn register_widget<T: Widget>(&mut self) {
        self.register(T::NAME, || Box::new(T::State::default()), || Box::new(T::Props::default()), Box::new(|event: WidgetEvent, widget_data: &mut WidgetData| {
            let (state, props) = widget_data.cast_state_and_props::<T::State, T::Props>();
            T::handle_event(&event, state, props);
        }));
    }

    pub fn make_widget_props(&self, name: &str) -> Box<dyn WidgetProps>{
        (self.widgets.get(name).unwrap().make_props)()
    }

    pub fn handle_event(&self, widget_kind: &str, event: WidgetEvent, widget_data: &mut WidgetData) {
        (self.widgets.get(widget_kind).unwrap().event_handler)(event, widget_data);
    }

}

pub trait Widget {
    const NAME: &'static str;
    type State: WidgetState + Default;
    type Props: WidgetProps + Default;

    fn handle_event(event: &WidgetEvent, state: &mut Self::State, props: &Self::Props);
}

pub struct ButtonWidget {}


impl Widget for ButtonWidget {
    const NAME: &'static str = "button";
    type State = ButtonWidgetState;
    type Props = ButtonWidgetProps;

    fn handle_event(event: &WidgetEvent, state: &mut Self::State, props: &Self::Props) {
        match event.kind() {
            WidgetEventKind::MouseOver => {
                state.is_hovering = true;
            }
            WidgetEventKind::MouseOut => {
                state.is_hovering = false;
            }
        }
    }
}

#[derive(Default, Reflect, Debug)]
pub struct ButtonWidgetProps {
    pub label: Text,
}

impl WidgetProps for ButtonWidgetProps {}

#[derive(Reflect, Debug, Default)]
pub struct ButtonWidgetState {
    pub is_hovering: bool,
}

impl WidgetState for ButtonWidgetState {}

#[derive(Reflect, Default, Debug)]
pub struct Text {
    pub parts: Vec<TextPart>,
}

#[derive(Reflect, Debug)]
pub enum TextPart {
    FixedText(String),
    VariableText(String),
}