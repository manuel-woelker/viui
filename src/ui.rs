use std::any::TypeId;
use std::ops::Deref;
use crate::arenal::{Arenal, Idx};
use crate::geometry::{Point, Rect};
use crate::widget_model::{ButtonWidgetProps, WidgetProps, WidgetState};

pub type StateBox = Box<dyn WidgetState>;
pub type PropsBox = Box<dyn WidgetProps>;

pub type WidgetEventHandler = Box<dyn Fn(WidgetEvent, &mut WidgetData)>;


pub struct WidgetData {
    props_type_id: TypeId,
    state: StateBox,
    props: PropsBox,
    event_handler: WidgetEventHandler,
    bounds: Rect,
}

impl WidgetData {
    pub fn props_type_id(&self) -> TypeId {
        self.props_type_id
    }
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
    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    pub fn bounds(&self) -> &Rect {
        &self.bounds
    }
}

pub struct UI {
    state_arena: Arenal<WidgetData>,
    hovered_widget: Option<Idx<WidgetData>>}

impl UI {
    pub fn new() -> UI {
        UI {
            state_arena: Arenal::new(),
            hovered_widget: None,
        }
    }

    pub fn add_widget<S: WidgetState, P: WidgetProps>(&mut self, state:S, props: P, event_handler: WidgetEventHandler) -> Idx<WidgetData> {
        self.state_arena.insert(WidgetData {
            props_type_id: TypeId::of::<P>(),
            state: Box::new(state),
            props: Box::new(props),
            bounds: Rect::default(),
            event_handler,
        })
    }

    pub fn widgets(&mut self) -> impl Iterator<Item = &mut WidgetData> {
        self.state_arena.entries()
    }

    pub fn handle_ui_event(&mut self, event: UiEvent) {
        match event.kind {
            UiEventKind::MouseMoved(position) => {
                if let Some(hovered_widget) = &self.hovered_widget {
                    let widget = &self.state_arena[hovered_widget];
                }
                for widget in self.widgets() {
                    let event_handler = std::mem::replace(&mut widget.event_handler, Box::new(|a, b|{}));
                    event_handler(WidgetEvent::mouse_out(), widget);
                    widget.event_handler = event_handler;
                    if widget.bounds.contains(&position) {
                        let event_handler = std::mem::replace(&mut widget.event_handler, Box::new(|a, b|{}));
                        event_handler(WidgetEvent::mouse_over(), widget);
                        widget.event_handler = event_handler;
//                        break;
                    }
                }
            }
        }
    }

}

#[derive(Debug)]
pub struct WidgetEvent {
    kind: WidgetEventKind,
}

#[derive(Debug)]
pub enum WidgetEventKind {
    MouseOver,
    MouseOut,
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
}

impl UiEvent {
    pub fn mouse_move(point: Point) -> Self {
        Self {
            kind: UiEventKind::MouseMoved(point),
        }
    }
}