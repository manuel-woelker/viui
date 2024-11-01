use std::any::Any;
use std::collections::HashMap;
use bevy_reflect::{Reflect, Typed};
use default_boxed::DefaultBoxed;
use femtovg::Canvas;
use crate::render::command::RenderCommand;
use crate::types::{Color, Point, Rect, Size};
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

pub trait WidgetEvents: Reflect + 'static {

}

pub type WidgetEventHandler = Box<dyn Fn(WidgetEvent, &mut WidgetData)+Send>;
pub type WidgetRenderFn = Box<dyn Fn(&mut Vec<RenderCommand>, &WidgetData)+Send>;


pub type EventList = Vec<String>;

pub struct WidgetDescriptor {
    pub(crate) kind_index: usize,
    pub make_state: Box<dyn Fn() -> Box<dyn WidgetState>+Send>,
    pub make_props: Box<dyn Fn() -> Box<dyn WidgetProps>+Send>,
    event_handler: WidgetEventHandler,
    render_fn: WidgetRenderFn,
    // events this widget may emit
    emitted_events: EventList,
}

pub struct WidgetRegistry {
    pub widgets: Vec<WidgetDescriptor>,
    pub widget_map: HashMap<String, usize>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            widget_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: impl Into<String>, make_state: impl Fn() -> Box<dyn WidgetState> +Send+ 'static, make_props: impl Fn() -> Box<dyn WidgetProps> +Send+ 'static, event_handler: impl Fn(WidgetEvent, &mut WidgetData)+Send + 'static, render_fn: impl Fn(&mut Vec<RenderCommand>, &WidgetData) +Send+ 'static, emitted_events: EventList,) {
        self.register_internal(name.into(), Box::new(make_state), Box::new(make_props), Box::new(event_handler), Box::new(render_fn), emitted_events);
    }

    fn register_internal(&mut self, name: String, make_state: Box<dyn Fn() -> Box<dyn WidgetState>+Send>, make_props: Box<dyn Fn() -> Box<dyn WidgetProps>+Send>, event_handler: WidgetEventHandler, render_fn: WidgetRenderFn,emitted_events: EventList,) {
        let kind_index = self.widgets.len();
        self.widgets.push(WidgetDescriptor { kind_index, event_handler, render_fn, make_state, make_props, emitted_events });
        self.widget_map.insert(name, kind_index);
    }

    pub fn register_widget<T: Widget>(&mut self,emitted_events: EventList) {
        self.register(T::NAME, || Box::new(T::State::default()), || Box::new(T::Props::default()), Box::new(|event: WidgetEvent, widget_data: &mut WidgetData| {
            let (state, props) = widget_data.cast_state_mut_and_props::<T::State, T::Props>();
            T::handle_event(&event, state, props);
        }), Box::new(|render_queue: &mut Vec<RenderCommand>, widget_data: &WidgetData| {
            let (state, props) = widget_data.cast_state_and_props::<T::State, T::Props>();
            T::render_widget(render_queue, state, props);
        }), emitted_events);
    }

    pub fn get_widget_by_name(&self, name: &str) -> &WidgetDescriptor {
        &self.widgets[*self.widget_map.get(name).unwrap()]
    }

    pub fn make_widget_props(&self, name: &str) -> Box<dyn WidgetProps>{
        (self.get_widget_by_name(name).make_props)()
    }

    pub fn handle_event(&self, widget_index: usize, event: WidgetEvent, widget_data: &mut WidgetData) {
        (self.widgets[widget_index].event_handler)(event, widget_data);
    }

    pub fn get_widget_index(&self, kind: &str) -> usize {
        *self.widget_map.get(kind).unwrap()
    }

    pub fn render_widget(&self, render_queue: &mut Vec<RenderCommand>, widget_data: &WidgetData) {
        (self.widgets[widget_data.kind_index()].render_fn)(render_queue, widget_data);
    }

}

pub trait Widget {
    const NAME: &'static str;
    type State: WidgetState + Default;
    type Props: WidgetProps + Default;

    fn handle_event(event: &WidgetEvent, state: &mut Self::State, props: &Self::Props);
    fn render_widget(render_queue: &mut Vec<RenderCommand>, state: &Self::State, props: &Self::Props);
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
            WidgetEventKind::MousePress => {
                state.is_pressed = true;
            }
            WidgetEventKind::MouseRelease => {
                state.is_pressed = false;
            }
        }
    }

    fn render_widget(render_queue: &mut Vec<RenderCommand>, state: &Self::State, props: &Self::Props) {
        if state.is_pressed {
            render_queue.push(RenderCommand::SetFillColor(Color::new(250, 250, 250, 255)));
        } else if state.is_hovering {
            render_queue.push(RenderCommand::SetFillColor(Color::new(230, 230, 230, 255)));
        } else {
            render_queue.push(RenderCommand::SetFillColor(Color::new(220, 220, 220, 255)));
        }
        render_queue.push(RenderCommand::FillRoundRect {
            rect: Rect::new(Point::new(0.0,0.0), Size::new(100.0, 40.0)),
            radius: 5.0,
        });
        render_queue.push(RenderCommand::Translate {x: 10.0, y: 20.0});
        render_queue.push(RenderCommand::DrawText(props.label.clone()));
    }
}

#[derive(Default, Reflect, Debug)]
pub struct ButtonWidgetProps {
    pub label: String,
}

impl WidgetProps for ButtonWidgetProps {}

#[derive(Reflect, Debug, Default)]
pub struct ButtonWidgetState {
    pub is_hovering: bool,
    pub is_pressed: bool,
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