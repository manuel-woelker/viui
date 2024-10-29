use std::any::Any;
use std::collections::HashMap;
use bevy_reflect::{Reflect, Typed};
use default_boxed::DefaultBoxed;
use femtovg::Canvas;

pub struct WidgetModel {
    pub widgets: Vec<Box<dyn WidgetProps>>,
}

impl WidgetModel {

}

/*
pub struct Widget {
    pub kind: WidgetKind,
}
*/

pub trait WidgetState: Reflect {

}

pub trait WidgetProps: Reflect {

}

pub struct WidgetDescriptor {
    make_state: Box<dyn Fn() -> Box<dyn WidgetState>>,
    make_props: Box<dyn Fn() -> Box<dyn WidgetProps>>,
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

    pub fn register(&mut self, name: impl Into<String>, make_state: impl Fn() -> Box<dyn WidgetState> + 'static, make_props: impl Fn() -> Box<dyn WidgetProps> + 'static) {
        self.register_internal(name.into(), Box::new(make_state), Box::new(make_props));
    }

    fn register_internal(&mut self, name: String, make_state: Box<dyn Fn() -> Box<dyn WidgetState>>, make_props: Box<dyn Fn() -> Box<dyn WidgetProps>>) {
        self.widgets.insert(name, WidgetDescriptor { make_state, make_props });
    }

    pub fn register_widget<T: Widget>(&mut self) {
        dbg!(T::NAME);
        self.register(T::NAME, || Box::new(T::State::default()), || Box::new(T::Props::default()))
    }

    pub fn make_widget_props(&self, name: &str) -> Box<dyn WidgetProps>{
        (self.widgets.get(name).unwrap().make_props)()
    }

}

trait Widget {
    const NAME: &'static str;
    type State: WidgetState + Default;
    type Props: WidgetProps + Default;
}

pub struct ButtonWidget {}


impl Widget for ButtonWidget {
    const NAME: &'static str = "button";
    type State = ButtonWidgetState;
    type Props = ButtonWidgetProps;
}
/*
#[derive(Default, Reflect)]
pub struct TextWidget {
    pub text: Text,
}
impl WidgetState for TextWidget {
}
*/

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