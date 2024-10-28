


pub struct WidgetModel {
    pub widgets: Vec<Widget>,
}

impl WidgetModel {

}


pub struct Widget {
    pub kind: WidgetKind,
}

pub enum WidgetKind {
    Text(TextWidget)
}

pub struct TextWidget {
    pub text: Text,
}

pub struct Text {
    pub parts: Vec<TextPart>,
}

pub enum TextPart {
    FixedText(String),
    VariableText(String),
}