use crate::infrastructure::layout_context::LayoutContext;
use crate::nodes::elements::kind::{Element, LayoutConstraints, NoEvents};
use crate::nodes::types::NodeProps;
use crate::render::command::RenderCommand;
use crate::render::context::RenderContext;
use crate::render::parameters::RenderParameters;
use crate::result::ViuiResult;
use bevy_reflect::Reflect;

pub struct LabelElement {}

impl Element for LabelElement {
    const NAME: &'static str = "label";
    type State = ();
    type Props = LabelElementProps;
    type Events = NoEvents;
    fn render_element(
        render_context: &mut RenderContext,
        parameters: &RenderParameters,
        _state: &Self::State,
        props: &Self::Props,
    ) {
        render_context.add_command(RenderCommand::SetStrokeColor(
            parameters.styling().text_color,
        ));
        render_context.add_command(RenderCommand::Translate { x: 10.0, y: 25.0 });
        render_context.add_command(RenderCommand::DrawText(props.label.clone()));
    }

    fn layout_element(
        _layout_context: &mut LayoutContext,
        _state: &mut Self::State,
        _props: &Self::Props,
    ) -> ViuiResult<LayoutConstraints> {
        Ok(LayoutConstraints::FixedLayout {
            width: 400.0,
            height: 40.0,
        })
    }
}

#[derive(Default, Reflect, Debug)]
pub struct LabelElementProps {
    pub label: String,
}

impl NodeProps for LabelElementProps {}

#[cfg(test)]
mod tests {
    use crate::infrastructure::font_pool::FontPool;
    use crate::infrastructure::image_pool::ImagePool;
    use crate::infrastructure::layout_context::LayoutContext;
    use crate::infrastructure::styling::Styling;
    use crate::nodes::data::NodeData;
    use crate::nodes::elements::kind::LayoutConstraints;
    use crate::nodes::elements::label::{LabelElement, LabelElementProps};
    use crate::nodes::registry::NodeRegistry;
    use crate::render::backend_svg::render_svg;
    use crate::render::command::RenderCommand;
    use crate::render::context::RenderContext;
    use crate::render::parameters::RenderParameters;
    use crate::resource::Resource;
    use crate::types::{Point, Size};
    use euclid::Rect;
    use std::fs;
    use std::path::Path;

    #[test]
    fn it_works() {
        let mut node_registry = NodeRegistry::new();
        node_registry.register_node::<LabelElement>();
        let node_descriptor = node_registry.get_node_by_name("label").unwrap();
        let state = (node_descriptor.make_state)().unwrap();
        let props = Box::new(LabelElementProps {
            label: "Hello".to_string(),
        });
        let mut image_pool = ImagePool::default();
        let mut font_pool = FontPool::new();
        let font_resource = Resource::from_path("assets/fonts/Quicksand-Regular.ttf");
        let font_idx = font_pool.load_font(&font_resource).unwrap();
        let styling = Styling::light();
        //        let styling = Styling::dark();
        let render_parameters = RenderParameters::new(&styling).unwrap();
        let mut node_data = NodeData {
            tag: "".to_string(),
            kind_index: 0,
            layout: Default::default(),
            state,
            props,
            children: vec![],
            prop_expressions: vec![],
            event_mappings: Default::default(),
        };
        let constraints =
            (node_descriptor.layout_fn)(&mut LayoutContext::new(&mut image_pool), &mut node_data)
                .unwrap();

        let mut render_context = RenderContext::new(&mut image_pool, &mut font_pool, 0.0).unwrap();
        let LayoutConstraints::FixedLayout { width, height } = constraints else {
            panic!("Expected fixed layout");
        };
        let size = Size::new(width, height);
        render_context.add_command(RenderCommand::SetFillColor(styling.background_color));
        render_context.add_command(RenderCommand::FillRect {
            rect: Rect::new(Point::new(0.0, 0.0), size),
        });
        render_context.add_command(RenderCommand::LoadFont {
            font_idx,
            resource: font_resource,
        });
        render_context.add_command(RenderCommand::SetFont { font_idx });
        (node_descriptor.render_fn)(&mut render_context, &render_parameters, &node_data).unwrap();
        let render_queue = render_context.render_queue();
        let name = "label";
        let file_path = format!("test/elements/{name}.svg");
        let file_path = Path::new(&file_path);
        let parent_path = file_path.parent().unwrap();
        fs::create_dir_all(parent_path).unwrap();
        let mut buffer = Vec::new();
        render_svg(size, &render_queue, &mut buffer, &file_path).unwrap();
        let svg = String::from_utf8(buffer).unwrap();
        let content = if file_path.exists() {
            fs::read_to_string(file_path).unwrap()
        } else {
            String::new()
        };
        if content != svg {
            fs::write(file_path, &svg).unwrap();
        }
    }
}
