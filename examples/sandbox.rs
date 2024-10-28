use std::fs::File;
use std::num::NonZeroU32;
use bevy_reflect::{GetPath, ParsedPath, Reflect};
use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color, FillRule, Paint, Path, Renderer};
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
use xui::observable_state::{ObservableState, TypedPath};
use xui::widget_model::{Text, TextPart, TextWidget, Widget, WidgetKind, WidgetModel};

#[derive(Debug, Reflect)]
struct AppState {
    counter: i32,
}


fn main() {
    println!("Starting XUI");
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

    let widget_model = WidgetModel {
        widgets: vec![Widget {
            kind: WidgetKind::Text(TextWidget {
                text: Text {
                    parts: vec![TextPart::FixedText("Counter: ".to_string()),
                                TextPart::VariableText("counter".to_string()),]
                }
            })
        }],
    };

    event_loop.run(move |event, _target, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CursorMoved { position, .. } => {
                mouse_position = position;
                window.request_redraw();
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                counter+=1;
                app_state.apply_change("Increment", |mutator| {
                    mutator.mutate(&counter_path, |counter| *counter+=1);
                });
                window.request_redraw();
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        Event::RedrawRequested(_) => {
            render(&context, &surface, &window, &mut canvas, &widget_model, &app_state, mouse_position, counter);
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
    widget_model: &WidgetModel,
    app_state: &ObservableState,
    square_position: PhysicalPosition<f64>,
    counter: i32,
) {
    // Make sure the canvas has the right size:
    let size = window.inner_size();
    canvas.set_size(size.width, size.height, window.scale_factor() as f32);

    canvas.clear_rect(0, 0, size.width, size.height, Color::white());

    for widget in &widget_model.widgets {
        match &widget.kind { WidgetKind::Text(text_widget) => {
            let mut text = "".to_string();
            for part in &text_widget.text.parts {
                match part {
                    TextPart::FixedText(fixed_string) => {
                        text.push_str(fixed_string);
                    }
                    TextPart::VariableText(path) => {
                        text.push_str(&format!("{:?}", app_state.state().reflect_path(&**path).unwrap()));
                    }
                }
            }
            canvas.fill_text(140.0, 140.0, text, &Paint::color(Color::hsl(0.0, 0.0, 0.0)).with_font_size(18.0).with_anti_alias(true)).unwrap();
        } }
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
    // Tell renderer to execute all drawing commands
    canvas.flush();
    // Display what we've just rendered
    surface.swap_buffers(context).expect("Could not swap buffers");
}