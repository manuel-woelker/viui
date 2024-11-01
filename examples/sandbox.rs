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
use winit::platform::run_return::EventLoopExtRunReturn;
use viui::observable_state::{ObservableState, TypedPath};
use viui::render::femtovg_renderer::FemtovgRenderer;
use viui::render::renderer::CommandRenderer;
use viui::types::{Point, Rect, Size};
use viui::ui::{MouseEventKind, UiEvent, WidgetData, WidgetEvent, WidgetEventKind, UI};
use viui::widget_model::{Text, TextPart, WidgetState, WidgetModel, ButtonWidgetProps, WidgetProps, WidgetRegistry, ButtonWidget, ButtonWidgetState};

#[derive(Debug, Reflect)]
struct AppState {
    counter: i32,
}

#[derive(Debug, Reflect)]
enum AppMessage {
    Increment,
    Decrement,
}


fn main() {
    println!("Starting VIUI");
    let event_loop = EventLoop::new();
    let (context, gl_display, window, surface) = create_window(&event_loop);

    let renderer = unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s).cast()) }
        .expect("Cannot create renderer");

    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    canvas.set_size(1000, 600, window.scale_factor() as f32);
    File::open("assets/fonts/Roboto-Regular.ttf").unwrap();
    canvas.add_font("assets/fonts/Roboto-Regular.ttf").unwrap();

    let app_state = ObservableState::new(AppState { counter: 19 });
    let counter_path = TypedPath::<i32>::new(ParsedPath::parse("counter").unwrap());
    let mut widget_registry = WidgetRegistry::new();
    widget_registry.register_widget::<ButtonWidget>(vec!["click".to_string()]);
    let mut ui = UI::new(app_state, move |app_state, message: &AppMessage| {
        match message {
            AppMessage::Increment => {
                app_state.apply_change("Increment",  |mutator| {
                    mutator.mutate(&counter_path, |counter| *counter += 1);
                });
            }
            AppMessage::Decrement => {
                app_state.apply_change("Decrement",  |mutator| {
                    mutator.mutate(&counter_path, |counter| *counter -= 1);
                });
            }
        }
    });
    ui.register_widget::<ButtonWidget>();

    let label_idx = ui.add_widget("button", ButtonWidgetState::default(), ButtonWidgetProps {
        label: "Counter".to_string(),
    });
    let increment_button = ui.add_widget("button", ButtonWidgetState::default(), ButtonWidgetProps {
        label: "Increment".to_string(),
    }, );
    ui.set_event_mapping(&increment_button, "click", AppMessage::Increment);
    let decrement_button = ui.add_widget("button", ButtonWidgetState::default(), ButtonWidgetProps {
        label: "Decrement".to_string(),
    }, );
    ui.set_event_mapping(&decrement_button, "click", AppMessage::Decrement);

    ui.set_widget_prop(&label_idx, "label", Text {
        parts: vec![TextPart::FixedText("The Counter: ".to_string()),
                    TextPart::VariableText("counter".to_string()), ]
    });
    let mut mouse_position: Point = Point::new(0.0, 0.0);
    event_loop.run(move |event, _target, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CursorMoved { position, .. } => {
                mouse_position = Point::new(position.x as f32, position.y as f32);
                ui.handle_ui_event(UiEvent::mouse_move(mouse_position));
                window.request_redraw();
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                // TODO: remove mouse_position from mouse_input
                ui.handle_ui_event(UiEvent::mouse_input(if state == ElementState::Pressed { MouseEventKind::Pressed} else { MouseEventKind::Released }));
                window.request_redraw();
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        Event::RedrawRequested(_) => {
            render(&context, &surface, &window, &mut canvas, &mut ui);
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
) {
    ui.eval_expressions();
    ui.perform_layout();
    let render_commands = ui.make_render_commands();

    let size = window.inner_size();
    canvas.set_size(size.width, size.height, window.scale_factor() as f32);
    canvas.reset_transform();
    canvas.clear_rect(0, 0, size.width, size.height, Color::white());
    FemtovgRenderer::new(canvas).render(&render_commands);
    // Tell renderer to execute all drawing commands*/
    canvas.flush();
    // Display what we've just rendered
    surface.swap_buffers(context).expect("Could not swap buffers");
}

