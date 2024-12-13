use crate::bail;
use crate::render::command::RenderCommand;
use crate::result::ViuiResult;
use crate::types::Size;
use std::collections::HashMap;
use std::io::Write;
use svg::node::element::Group;
use svg::{Document, Node};

pub fn render_svg(
    size: Size,
    render_list: &[RenderCommand],
    write: &mut dyn Write,
) -> ViuiResult<()> {
    let document = render_svg_document(size, render_list)?;
    write.write_all(&document.to_string().into_bytes())?;
    Ok(())
}

pub fn render_svg_document(size: Size, render_list: &[RenderCommand]) -> ViuiResult<Document> {
    // Create an SVG from

    let mut document = Document::new().set("viewBox", (0, 0, size.width, size.height));
    #[derive(Debug)]
    struct Entry {
        group: Group,
        children: Vec<Box<dyn Node>>,
    }

    let entry_stack: &mut Vec<Entry> = &mut vec![Entry {
        group: Group::new(),
        children: Vec::new(),
    }];
    let mut save_stack = vec![];
    fn push_element(element: impl Node, entry_stack: &mut Vec<Entry>) {
        entry_stack
            .last_mut()
            .unwrap()
            .children
            .push(Box::new(element));
    }
    fn push_group(group: Group, entry_stack: &mut Vec<Entry>) {
        entry_stack.push(Entry {
            group,
            children: Vec::new(),
        });
    }

    fn pop_stack(entry_stack: &mut Vec<Entry>) {
        let Entry {
            mut group,
            children,
        } = entry_stack.pop().unwrap();
        for child in children {
            group.append(child);
        }
        entry_stack
            .last_mut()
            .unwrap()
            .children
            .push(Box::new(group));
    }
    let mut image_map = HashMap::new();
    for render_command in render_list {
        match render_command {
            RenderCommand::Line { start, end } => {
                push_element(
                    svg::node::element::Line::new()
                        .set("x1", start.x)
                        .set("y1", start.y)
                        .set("x2", end.x)
                        .set("y2", end.y),
                    entry_stack,
                );
            }
            RenderCommand::SetStrokeWidth(width) => {
                push_group(Group::new().set("stroke-width", *width), entry_stack);
            }
            RenderCommand::SetStrokeColor(color) => {
                let rgba = color.rgba;
                push_group(
                    Group::new().set("stroke", format!("rgb({} {} {})", rgba.r, rgba.g, rgba.b)),
                    entry_stack,
                );
            }
            RenderCommand::SetFillColor(color) => {
                let rgba = color.rgba;
                push_group(
                    Group::new().set("fill", format!("rgb({} {} {})", rgba.r, rgba.g, rgba.b)),
                    entry_stack,
                );
            }
            RenderCommand::FillRect { rect } => {
                push_element(
                    svg::node::element::Rectangle::new()
                        .set("x", rect.origin.x)
                        .set("y", rect.origin.y)
                        .set("width", rect.size.width)
                        .set("height", rect.size.height),
                    entry_stack,
                );
            }
            RenderCommand::FillRoundRect { rect, radius } => {
                push_element(
                    svg::node::element::Rectangle::new()
                        .set("x", rect.origin.x)
                        .set("y", rect.origin.y)
                        .set("width", rect.size.width)
                        .set("height", rect.size.height)
                        .set("rx", *radius),
                    entry_stack,
                );
            }
            RenderCommand::Arc {
                center,
                radius,
                start_angle,
                end_angle,
            } => {
                let start_pos = (
                    center.x + radius * start_angle.cos(),
                    center.y + radius * start_angle.sin(),
                );
                let end_pos = (
                    center.x + radius * end_angle.cos(),
                    center.y + radius * end_angle.sin(),
                );
                push_element(
                    svg::node::element::Path::new().set(
                        "d",
                        format!(
                            "M {} {} A {} {} 0.0 0.0 0.0 {} {}",
                            start_pos.0, start_pos.1, *radius, *radius, end_pos.0, end_pos.1,
                        ),
                    ),
                    entry_stack,
                );
            }
            RenderCommand::DrawText(text) => {
                push_element(svg::node::element::Text::new(text), entry_stack);
            }
            RenderCommand::Save => {
                save_stack.push(entry_stack.len());
            }
            RenderCommand::Restore => {
                let desired_sized = save_stack.pop().unwrap();
                while entry_stack.len() > desired_sized {
                    pop_stack(entry_stack);
                }
            }
            RenderCommand::Translate { x, y } => {
                push_group(
                    Group::new().set("transform", format!("translate({} {})", x, y)),
                    entry_stack,
                );
            }
            RenderCommand::LoadImage { image_id, resource } => {
                image_map.insert(image_id, resource);
            }
            RenderCommand::DrawImage { image_id } => {
                push_element(
                    svg::node::element::Image::new().set("href", image_map[&image_id].as_path()?),
                    entry_stack,
                );
            }
            RenderCommand::ClipRect(rect) => {
                push_group(
                    Group::new().set("clip-path", "url(#clip-rect)"),
                    entry_stack,
                );
                push_element(
                    svg::node::element::Rectangle::new()
                        .set("id", "clip-rect")
                        .set("x", rect.origin.x)
                        .set("y", rect.origin.y)
                        .set("width", rect.size.width)
                        .set("height", rect.size.height),
                    entry_stack,
                );
            }
            _ => {
                bail!("Unsupported render command {:?}", render_command);
            }
        }
    }
    while entry_stack.len() > 1 {
        pop_stack(entry_stack);
    }
    let root_children = entry_stack.pop().unwrap().children;
    for child in root_children {
        document.append(child);
    }
    document.assign("style", "background-color: white");
    Ok(document)
}

// Tests
#[cfg(test)]
mod tests {
    use crate::render::backend_svg::render_svg;
    use crate::render::command::{ImageId, RenderCommand};
    use crate::resource::Resource;
    use crate::types::{Color, Point, Rect, Size};
    use expect_test::{expect, Expect};
    use std::fs;

    #[test]
    fn test_line() {
        test_render_svg(
            "xline",
            &[RenderCommand::Line {
                start: Point::new(0.0, 1.0),
                end: Point::new(200.0, 300.0),
            }],
            expect![[r#"
                <svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
                <line x1="0" x2="200" y1="1" y2="300"/>
                </svg>"#]],
        );
    }

    fn test_render_svg(name: &str, commands: &[RenderCommand], expected: Expect) {
        let mut buffer = Vec::new();
        render_svg(Size::new(1000.0, 1000.0), commands, &mut buffer).unwrap();
        let svg = String::from_utf8(buffer).unwrap();
        let file_path = format!("test/{name}.svg");
        let file_path = std::path::Path::new(&file_path);
        let content = if file_path.exists() {
            fs::read_to_string(file_path).unwrap()
        } else {
            String::new()
        };
        if content != svg {
            fs::write(file_path, &svg).unwrap();
        }
        expected.assert_eq(&svg);
    }

    macro_rules! test_render_svg {
        ($($name:ident, $input:expr, $expected:expr;)+) => {
            $(#[test]
            fn $name() {
                test_render_svg(stringify!($name), $input, $expected);
            })+
        };
    }

    test_render_svg!(
      empty, &[], expect![[r#"<svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg"/>"#]];
      line, &[RenderCommand::Line { start: Point::new(0.0, 1.0), end: Point::new(200.0, 300.0) }], expect![[r#"
          <svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <line x1="0" x2="200" y1="1" y2="300"/>
          </svg>"#]];
      line_with_stroke_width, &[RenderCommand::SetStrokeWidth(2.0), RenderCommand::Line { start: Point::new(0.0, 1.0), end: Point::new(200.0, 300.0) }], expect![[r#"
          <svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <g stroke-width="2">
          <line x1="0" x2="200" y1="1" y2="300"/>
          </g>
          </svg>"#]];
      line_with_stroke_width_and_color, &[RenderCommand::SetStrokeWidth(2.0), RenderCommand::SetStrokeColor(Color::GRAY), RenderCommand::Line { start: Point::new(0.0, 1.0), end: Point::new(200.0, 300.0) }], expect![[r#"
          <svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <g stroke-width="2">
          <g stroke="rgb(235 235 235)">
          <line x1="0" x2="200" y1="1" y2="300"/>
          </g>
          </g>
          </svg>"#]];
      save_restore, &[RenderCommand::Save, RenderCommand::SetStrokeWidth(2.0), RenderCommand::Line { start: Point::new(0.0, 10.0), end: Point::new(200.0, 300.0) }, RenderCommand::Restore, RenderCommand::Line { start: Point::new(0.0, 1.0), end: Point::new(2.0, 3.0) }], expect![[r#"
          <svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <g stroke-width="2">
          <line x1="0" x2="200" y1="10" y2="300"/>
          </g>
          <line x1="0" x2="2" y1="1" y2="3"/>
          </svg>"#]];
      translate, &[RenderCommand::Translate { x: 70.0, y: 80.9}, RenderCommand::Line { start: Point::new(0.0, 10.0), end: Point::new(200.0, 300.0) }], expect![[r#"
          <svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <g transform="translate(70 80.9)">
          <line x1="0" x2="200" y1="10" y2="300"/>
          </g>
          </svg>"#]];
      fill_rect, &[RenderCommand::SetFillColor(Color::GRAY), RenderCommand::FillRect { rect: Rect::new(Point::new(3.0, 4.0), Size::new(100.0, 200.0)) }], expect![[r#"
          <svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <g fill="rgb(235 235 235)">
          <rect height="200" width="100" x="3" y="4"/>
          </g>
          </svg>"#]];
      fill_round_rect, &[RenderCommand::SetFillColor(Color::GRAY), RenderCommand::FillRoundRect { rect: Rect::new(Point::new(3.0, 4.0), Size::new(100.0, 200.0)), radius: 5.0 }], expect![[r#"
          <svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <g fill="rgb(235 235 235)">
          <rect height="200" rx="5" width="100" x="3" y="4"/>
          </g>
          </svg>"#]];
      text, &[RenderCommand::DrawText("foo<bar&".to_string())], expect![[r#"
          <svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <text>
          foo&lt;bar&amp;
          </text>
          </svg>"#]];
      arc, &[RenderCommand::Arc{ center: Point::new(10.0, 20.0), radius: 3.0, start_angle: 0.0, end_angle: std::f32::consts::PI}], expect![[r#"
          <svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <path d="M 13 20 A 3 3 0.0 0.0 0.0 7 20"/>
          </svg>"#]];
      image, &[RenderCommand::LoadImage{ image_id: ImageId(42), resource: Resource::from("../assets/images/cat_playing.jpg") }, RenderCommand::DrawImage{ image_id: ImageId(42)}], expect![[r#"
          <svg style="background-color: white" viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <image href="../assets/images/cat_playing.jpg"/>
          </svg>"#]];

    );
}
