use crate::infrastructure::layout_context::LayoutContext;
use crate::nodes::elements::kind::{Element, LayoutConstraints, NoEvents};
use crate::nodes::types::{NodeProps, NodeState};
use crate::render::command::RenderCommand;
use crate::render::context::RenderContext;
use crate::render::parameters::RenderParameters;
use crate::result::ViuiResult;
use crate::types::Float;
use bevy_reflect::Reflect;

pub struct ImageElement {}

impl Element for ImageElement {
    const NAME: &'static str = "image";
    type State = ImageElementState;
    type Props = ImageElementProps;
    type Events = NoEvents;
    fn render_element(
        render_context: &mut RenderContext,
        _parameters: &RenderParameters,
        _state: &Self::State,
        props: &Self::Props,
    ) {
        let image_id = render_context.get_image_id(&props.src).unwrap();
        render_context.add_command(RenderCommand::DrawImage { image_id });
    }

    fn layout_element(
        layout_context: &mut LayoutContext,
        _state: &mut Self::State,
        props: &Self::Props,
    ) -> ViuiResult<LayoutConstraints> {
        let size = layout_context.get_image_size(&props.src)?;
        Ok(LayoutConstraints::FixedLayout {
            width: size.width,
            height: size.height,
        })
    }
}

#[derive(Default, Reflect, Debug)]
pub struct ImageElementProps {
    pub src: String,
}

impl NodeProps for ImageElementProps {}

#[derive(Default, Reflect, Debug)]
pub struct ImageElementState {
    pub width: Float,
    pub height: Float,
}
impl NodeState for ImageElementState {}
