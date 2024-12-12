use crate::bail;
use crate::render::command::RenderCommand;
use crate::result::ViuiResult;
use crate::types::Size;
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
    struct Collection {
        group: Group,
        children: Vec<Entry>,
    }
    #[derive(Debug)]
    enum Entry {
        Group(Collection),
        Element(Box<dyn Node>),
    }

    let entry_stack: &mut Vec<Collection> = &mut vec![Collection {
        group: Group::new(),
        children: Vec::new(),
    }];
    fn push_element(element: impl Node, entry_stack: &mut Vec<Collection>) {
        entry_stack
            .last_mut()
            .unwrap()
            .children
            .push(Entry::Element(Box::new(element)));
    }
    fn push_group(group: Group, entry_stack: &mut Vec<Collection>) {
        entry_stack.push(Collection {
            group,
            children: Vec::new(),
        });
    }
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
            _ => {
                bail!("Unsupported render command {:?}", render_command);
            }
        }
    }
    let mut root_group = Group::new();
    while let Some(mut collection) = entry_stack.pop() {
        let group = &mut collection.group;
        for child in collection.children {
            match child {
                Entry::Group(inner_group) => {
                    group.append(inner_group.group);
                }
                Entry::Element(element) => {
                    group.append(element);
                }
            }
        }
        group.append(root_group);
        root_group = collection.group;
    }
    document.append(root_group);
    Ok(document)
}

// Tests
#[cfg(test)]
mod tests {
    use crate::render::backend_svg::render_svg;
    use crate::render::command::RenderCommand;
    use crate::types::{Point, Size};
    use expect_test::{expect, Expect};

    #[test]
    fn test_line() {
        test_render_svg(
            &[RenderCommand::Line {
                start: Point::new(0.0, 1.0),
                end: Point::new(2.0, 3.0),
            }],
            expect![[r#"
                <svg viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
                <g>
                <line x1="0" x2="2" y1="1" y2="3"/>
                <g/>
                </g>
                </svg>"#]],
        );
    }

    fn test_render_svg(commands: &[RenderCommand], expected: Expect) {
        let mut buffer = Vec::new();
        render_svg(Size::new(1000.0, 1000.0), commands, &mut buffer).unwrap();
        let svg = String::from_utf8(buffer).unwrap();
        expected.assert_eq(&svg);
    }

    macro_rules! test_render_svg {
        ($($name:ident, $input:expr, $expected:expr;)+) => {
            $(#[test]
            fn $name() {
                test_render_svg($input, $expected);
            })+
        };
    }

    test_render_svg!(
      empty, &[], expect![[r#"
          <svg viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <g>
          <g/>
          </g>
          </svg>"#]];
      line, &[RenderCommand::Line { start: Point::new(0.0, 1.0), end: Point::new(2.0, 3.0) }], expect![[r#"
          <svg viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <g>
          <line x1="0" x2="2" y1="1" y2="3"/>
          <g/>
          </g>
          </svg>"#]];
      line_with_stroke_width, &[RenderCommand::SetStrokeWidth(2.0), RenderCommand::Line { start: Point::new(0.0, 1.0), end: Point::new(2.0, 3.0) }], expect![[r#"
          <svg viewBox="0 0 1000 1000" xmlns="http://www.w3.org/2000/svg">
          <g>
          <g stroke-width="2">
          <line x1="0" x2="2" y1="1" y2="3"/>
          <g/>
          </g>
          </g>
          </svg>"#]];

    );
}
