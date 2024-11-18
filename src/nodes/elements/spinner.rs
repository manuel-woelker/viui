use crate::infrastructure::layout_context::LayoutContext;
use crate::nodes::elements::kind::{Element, LayoutConstraints, NoEvents};
use crate::render::command::RenderCommand;
use crate::render::context::RenderContext;
use crate::result::ViuiResult;
use crate::types::{Color, Point, Rect, Size};
use std::f32::consts::PI;

pub struct SpinnerElement {}

impl Element for SpinnerElement {
    const NAME: &'static str = "spinner";
    type State = ();
    type Props = ();
    type Events = NoEvents;
    fn render_element(
        render_context: &mut RenderContext,
        _state: &Self::State,
        _props: &Self::Props,
    ) {
        let t = render_context.time() * 3.0;
        render_context.set_animated();
        render_context.add_command(RenderCommand::SetStrokeWidth(0.0));
        render_context.add_command(RenderCommand::SetFillColor(Color::new(255, 255, 255, 255)));
        render_context.add_command(RenderCommand::FillRect {
            rect: Rect::new(Point::new(0.0, 0.0), Size::new(60.0, 60.0)),
        });
        render_context.add_command(RenderCommand::SetStrokeWidth(3.0));
        render_context.add_command(RenderCommand::SetStrokeColor(Color::new(0, 105, 0, 255)));
        render_context.add_command(RenderCommand::Arc {
            center: Point::new(30.0, 30.0),
            radius: 20.0,
            start_angle: 0.0 + t,
            end_angle: 1.0 * PI + t,
        });
    }

    fn layout_element(
        _layout_context: &mut LayoutContext,
        _state: &mut Self::State,
        _props: &Self::Props,
    ) -> ViuiResult<LayoutConstraints> {
        Ok(LayoutConstraints::FixedLayout {
            width: 60.0,
            height: 60.0,
        })
    }
}
