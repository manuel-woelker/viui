use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fs::File;
use std::num::NonZeroU32;
use bevy_reflect::{GetPath, ParsedPath, Reflect};
use femtovg::renderer::OpenGl;
use femtovg::{Baseline, Canvas, Color, FillRule, Paint, Path, Renderer};
use glutin::surface::Surface;
use glutin::{context::PossiblyCurrentContext, display::Display};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit::{dpi::PhysicalSize, window::Window};

use glutin::{
    config::ConfigTemplateBuilder,
    context::ContextAttributesBuilder,
    display::GetGlDisplay,
    prelude::*,
    surface::{SurfaceAttributesBuilder, WindowSurface},
};
use rstar::primitives::Rectangle;
use viui::observable_state::{ObservableState, TypedPath};
use viui::render::femtovg_renderer::FemtovgRenderer;
use viui::render::renderer::CommandRenderer;
use viui::types::{Point, Rect, Size};
use viui::ui::{UiEvent, WidgetData, WidgetEvent, WidgetEventKind, UI};
use viui::widget_model::{Text, TextPart, WidgetState, WidgetModel, ButtonWidgetProps, WidgetProps, WidgetRegistry, ButtonWidget, ButtonWidgetState};

#[derive(Debug, Reflect)]
struct AppState {
    counter: i32,
}


struct RectEntry {
    rect: Rectangle<(f32, f32)>,
    index: isize,
}

struct UiState {
    rect_list: Vec<RectEntry>,
}

fn main() {
    println!("Starting VIUI");
//    dbg!(&b);
    let event_loop = EventLoop::new();
    let (context, gl_display, window, surface) = create_window(&event_loop);

    let renderer = unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s).cast()) }
        .expect("Cannot create renderer");

    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    canvas.set_size(1000, 600, window.scale_factor() as f32);
    File::open("assets/fonts/Roboto-Regular.ttf").unwrap();
    canvas.add_font("assets/fonts/Roboto-Regular.ttf").unwrap();

    let mut mouse_position = PhysicalPosition::new(0., 0.);
    let mut counter = 19;


    let mut app_state = ObservableState::new(AppState { counter });
    let counter_path = TypedPath::<i32>::new(ParsedPath::parse("counter").unwrap());
    let mut ui_state = UiState { rect_list: Vec::new() };
    let mut widget_registry = WidgetRegistry::new();
    widget_registry.register_widget::<ButtonWidget>();
    let mut first_button =widget_registry.make_widget_props("button");
/*    let text = first_button.path_mut::<Text>("#0").unwrap();
    *text = Text {
        parts: vec![TextPart::FixedText("The Counter: ".to_string()),
                    TextPart::VariableText("counter".to_string()), ]
    };*/
    let mut ui = UI::new();
    ui.register_widget::<ButtonWidget>();

    let _button_idx = ui.add_widget("button", ButtonWidgetState::default(), ButtonWidgetProps {
        label: "Increment".to_string(),
    }, );
    ui.add_widget("button", ButtonWidgetState::default(), ButtonWidgetProps {
        label: "Counter".to_string(),
    });


/*    let _button_idx = ui.add_widget("button", ButtonWidgetState::default(), ButtonWidgetProps {
        label: Text {
            parts: vec![TextPart::FixedText("Increment".to_string())],
        },
    }, );
    ui.add_widget("button", ButtonWidgetState::default(), ButtonWidgetProps {
        label: Text {
            parts: vec![TextPart::FixedText("We were clicked ".to_string()),
                        TextPart::VariableText("counter".to_string()),
                        TextPart::FixedText(" times.".to_string()),]
        }
    });

 */
/*    let widget_model = WidgetModel {
        widgets: vec![
            first_button,
            Box::new(ButtonWidgetProps {
                label: Text {
                        parts: vec![TextPart::FixedText("Increment".to_string())],
                    },
            }),
            Box::new(ButtonWidgetProps {
                label: Text {
                        parts: vec![TextPart::FixedText("We were clicked ".to_string()),
                                    TextPart::VariableText("counter".to_string()),
                                    TextPart::FixedText(" times.".to_string()),]
                    }
                }),
        ],
    };
*/
    event_loop.run(move |event, _target, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CursorMoved { position, .. } => {
                mouse_position = position;
                ui.handle_ui_event(UiEvent::mouse_move(Point::new(mouse_position.x as f32, mouse_position.y as f32)));
                window.request_redraw();
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                counter += 1;
                app_state.apply_change("Increment", |mutator| {
                    mutator.mutate(&counter_path, |counter| *counter += 1);
                });
                window.request_redraw();
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        Event::RedrawRequested(_) => {
            render(&context, &surface, &window, &mut canvas, &mut ui, &app_state, &mut ui_state, mouse_position, counter);
        }
        _ => {}
    })
}

fn create_window(event_loop: &EventLoop<()>) -> (PossiblyCurrentContext, Display, Window, Surface<WindowSurface>) {
    let window_builder = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1000., 600.))
        .with_title("Femtovg");

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(event_loop, template, |mut configs| configs.next().unwrap())
        .unwrap();

    let window = window.unwrap();

    let gl_display = gl_config.display();

    let context_attributes = ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));

    let mut not_current_gl_context =
        Some(unsafe { gl_display.create_context(&gl_config, &context_attributes).unwrap() });

    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window.raw_window_handle(),
        NonZeroU32::new(1000).unwrap(),
        NonZeroU32::new(600).unwrap(),
    );

    let surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };

    (
        not_current_gl_context.take().unwrap().make_current(&surface).unwrap(),
        gl_display,
        window,
        surface,
    )
}

fn render<T: Renderer>(
    context: &PossiblyCurrentContext,
    surface: &Surface<WindowSurface>,
    window: &Window,
    canvas: &mut Canvas<T>,
    ui: &mut UI,
    app_state: &ObservableState,
    ui_state: &mut UiState,
    square_position: PhysicalPosition<f64>,
    counter: i32,
) {
    let render_commands = ui.make_render_commands();

    let size = window.inner_size();
    canvas.set_size(size.width, size.height, window.scale_factor() as f32);
    canvas.reset_transform();
    canvas.clear_rect(0, 0, size.width, size.height, Color::white());
    FemtovgRenderer::new(canvas).render(&render_commands);
    /*
    let line_height = 50.0;
    let mut render_registry: HashMap<TypeId, Box<dyn Fn(&mut Canvas<T>, &WidgetData)>> = HashMap::new();
/*    render_registry.insert(TypeId::of::<TextWidget>(), Box::new(|canvas: &mut Canvas<T>, widget: &dyn WidgetState| {
        let text_widget = widget.as_any().downcast_ref::<TextWidget>().unwrap();
        let string = text_to_string(&app_state, &text_widget.text.parts);
        canvas.fill_text(140.0, 0.0, string, &Paint::color(Color::hsl(0.0, 0.0, 0.0)).with_font_size(18.0).with_anti_alias(true)).unwrap();
    }));*/
    render_registry.insert(TypeId::of::<ButtonWidgetProps>(), Box::new(|canvas: &mut Canvas<T>, widget: &WidgetData| {
        let props = widget.cast_props::<ButtonWidgetProps>();
        let state = widget.cast_state::<ButtonWidgetState>();
        let mut path = Path::new();
        let corner_radius = 5.0;
        let bounds = widget.bounds();
        path.rounded_rect(
            0.0,
            0.0,
            bounds.width(),
            bounds.height(),
            corner_radius,
        );
        if state.is_hovering {
            canvas.fill_path(&path, &Paint::color(Color::hsl(0.5, 0.5, 0.8)));
        }
        canvas.stroke_path(&path, &Paint::color(Color::hsl(0.5, 0.5, 0.0)));

        let string = text_to_string(&app_state, &props.label.parts);
        canvas.fill_text(10.0, bounds.height()/2.0, string, &Paint::color(Color::hsl(0.0, 0.0, 0.0)).with_text_baseline(Baseline::Middle).with_font_size(20.0).with_anti_alias(true)).unwrap();
    }));
    // Make sure the canvas has the right size:
    let mut ypos = 40.0;
    let mut index = 0isize;
    for widget in ui.widgets() {
        widget.set_bounds(Rect::new(Point::new(50.0, ypos), Size::new(400.0, line_height)));
        let bounds = widget.bounds();
        canvas.reset_transform();
        canvas.translate(bounds.upper_left.x, bounds.upper_left.y);
/*        rect_list.push(RectEntry {
            index,
            rect: Rectangle::from_corners((0.0, ypos), (200.0, ypos+line_height)),
        });*/
        index +=1;
        let renderer = render_registry.get(&widget.props_type_id()).unwrap();
        renderer(canvas, widget);
/*        match &widget.kind {
            WidgetKind::Text(text_widget) => {
                let string = text_to_string(&app_state, &text_widget.text.parts);
                canvas.fill_text(140.0, ypos, string, &Paint::color(Color::hsl(0.0, 0.0, 0.0)).with_font_size(18.0).with_anti_alias(true)).unwrap();
            }
            WidgetKind::Button(button_widget) => {
                let string = text_to_string(&app_state, &button_widget.text.parts);
                let mut path = Path::new();
                let corner_radius = 10.0;
                path.rounded_rect_varying(
                    120.0,
                    ypos-line_height+line_height/3.0,
                    200.0,
                    line_height,
                    corner_radius,
                    corner_radius,
                    corner_radius,
                    corner_radius,
                );
                canvas.stroke_path(&path, &Paint::color(Color::hsl(0.5, 0.5, 0.0)));
                canvas.fill_text(140.0, ypos, string, &Paint::color(Color::hsl(0.0, 0.0, 0.0)).with_font_size(18.0).with_anti_alias(true)).unwrap();
            }
        }
*/
        ypos += line_height;
    }

    /*
        // Make smol red rectangle
        canvas.clear_rect(
            square_position.x as u32,
            square_position.y as u32,
            30,
            30,
            Color::rgbf(1., 0., 0.),
        );
    //    canvas.clear_rect(140, 140, 40, 40, Color::rgbf(1., 0., 0.));
        let mut path = Path::new();
        let corner_radius = 10.0;
        path.rounded_rect_varying(
            120.0,
            120.0,
            200.0,
            100.0,
            corner_radius,
            corner_radius,
            corner_radius,
            corner_radius,
        );
        canvas.translate(5.0, 5.0);
        canvas.fill_path(&path, &Paint::color(Color::hsl(0.5, 0.0, 0.8)));
        canvas.reset();
        canvas.fill_path(&path, &Paint::color(Color::hsl(0.5, 0.5, 0.5)));
        canvas.stroke_path(&path, &Paint::color(Color::hsl(0.5, 0.5, 0.0)));
        canvas.fill_text(140.0, 140.0, format!("Increment {counter}"), &Paint::color(Color::hsl(0.0, 0.0, 0.0)).with_font_size(18.0).with_anti_alias(true)).unwrap();
    */
    // Tell renderer to execute all drawing commands*/
    canvas.flush();
    // Display what we've just rendered
    surface.swap_buffers(context).expect("Could not swap buffers");
}

fn text_to_string(app_state: &&ObservableState, text: &Vec<TextPart>) -> String {
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