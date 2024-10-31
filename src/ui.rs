use std::any::TypeId;
use bevy_reflect::{GetPath, Reflect};
use UiEventKind::MouseMoved;
use crate::arenal::{Arenal, Idx};
use crate::observable_state::ObservableState;
use crate::render::command::RenderCommand;
use crate::types::{Point, Rect, Size};
use crate::widget_model::{ButtonWidgetProps, Text, TextPart, Widget, WidgetEventHandler, WidgetProps, WidgetRegistry, WidgetState};

pub type StateBox = Box<dyn WidgetState>;
pub type PropsBox = Box<dyn WidgetProps>;



#[derive(Clone, Debug, Default)]
pub struct LayoutInfo {
    bounds: Rect,
}

pub struct WidgetData {
    kind_index: usize,
    props_type_id: TypeId,
    layout: LayoutInfo,
    state: StateBox,
    props: PropsBox,
    prop_expressions: Vec<PropExpression>
}

pub struct PropExpression {
    pub field_name: String,
    pub text: Text,
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
    event_handler: Box<dyn Fn(&mut ObservableState)>,
}

impl UI {
    pub fn new(state: ObservableState, event_handler: impl Fn(&mut ObservableState) + 'static) -> UI {
        UI {
            widget_registry: WidgetRegistry::new(),
            state_arena: Arenal::new(),
            hovered_widget: None,
            app_state: Box::new(state),
            event_handler: Box::new(event_handler),
        }
    }

    pub fn add_widget<S: WidgetState, P: WidgetProps>(&mut self, kind: &str, state:S, props: P) -> Idx<WidgetData> {
        let kind_index = self.widget_registry.get_widget_index(kind);
        self.state_arena.insert(WidgetData {
            kind_index,
            props_type_id: TypeId::of::<P>(),
            state: Box::new(state),
            props: Box::new(props),
            layout: LayoutInfo::default(),
            prop_expressions: Vec::new(),
        })
    }

    pub fn widgets(&mut self) -> impl Iterator<Item = &mut WidgetData> {
        self.state_arena.entries_mut()
    }

    pub fn handle_ui_event(&mut self, event: UiEvent) {
        match event.kind {
            MouseMoved(position) => {
                for widget in self.state_arena.entries_mut() {
                    self.widget_registry.handle_event(widget.kind_index, WidgetEvent::mouse_out(), widget);
                    if widget.layout.bounds.contains(position) {
                        self.widget_registry.handle_event(widget.kind_index, WidgetEvent::mouse_over(), widget);
//                        break;
                    }
                }
            }
            UiEventKind::MouseInput(position) => {
                (self.event_handler)(self.app_state.as_mut());
            }
        }
    }

    pub fn register_widget<T: Widget>(&mut self) {
        self.widget_registry.register_widget::<T>();
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
            render_commands.push(RenderCommand::Translate {x: 0.0, y: 40.0})
        }
        render_commands
    }

    pub fn set_widget_prop(&mut self, widget_index: Idx<WidgetData>, field_name: &str, text: Text) {
        self.state_arena[&widget_index].prop_expressions.push(PropExpression {
            field_name: field_name.to_string(),
            text,
        });
    }

    pub fn handle_event(&mut self, event: UiEvent) {}


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
    MouseInput(Point),
}

impl UiEvent {
    pub fn mouse_move(position: Point) -> Self {
        Self {
            kind: MouseMoved(position),
        }
    }
    pub fn mouse_input(point: Point) -> Self {
        Self {
            kind: UiEventKind::MouseInput(point),
        }
    }
}