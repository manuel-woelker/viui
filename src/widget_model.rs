use std::any::Any;
use femtovg::Canvas;

pub struct WidgetModel {
    pub widgets: Vec<Box<dyn Widget>>,
}

impl WidgetModel {

}

/*
pub struct Widget {
    pub kind: WidgetKind,
}
*/
pub trait Widget: Any + 'static {
    fn as_any(&self) -> &dyn Any;
}

pub struct TextWidget {
    pub text: Text,
}

impl Widget for TextWidget {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
/*
pub struct ButtonWidget {
    pub text: Text,
    pub on_click: String,
}
*/
pub struct Text {
    pub parts: Vec<TextPart>,
}

pub enum TextPart {
    FixedText(String),
    VariableText(String),
}