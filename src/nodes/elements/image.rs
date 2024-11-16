use crate::nodes::elements::kind::{Element, LayoutConstraints, NoEvents};
use crate::nodes::types::NodeProps;
use crate::render::command::{ImageId, RenderCommand};
use crate::render::context::RenderContext;
use crate::resource::Resource;
use crate::result::ViuiResult;
use bevy_reflect::Reflect;

pub struct ImageElement {}

impl Element for ImageElement {
    const NAME: &'static str = "image";
    type State = ();
    type Props = ImageElementProps;
    type Events = NoEvents;
    fn render_element(
        render_context: &mut RenderContext,
        _state: &Self::State,
        props: &Self::Props,
    ) {
        let image_id = render_context.get_image_id(&props.src).unwrap();
        render_context.add_command(RenderCommand::DrawImage { image_id });
    }

    fn layout_element(_state: &Self::State, _props: &Self::Props) -> ViuiResult<LayoutConstraints> {
        Ok(LayoutConstraints::FixedLayout {
            width: 512.0,
            height: 512.0,
        })
    }
}

#[derive(Default, Reflect, Debug)]
pub struct ImageElementProps {
    pub src: String,
}

impl NodeProps for ImageElementProps {}
