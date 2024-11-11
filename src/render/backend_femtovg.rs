use crate::nodes::events::{MouseEventKind, UiEvent};
use crate::render::command::RenderCommand;
use crate::types::Point;
use crate::ui::RenderBackendMessage;
use crossbeam_channel::{Receiver, Sender};
use femtovg::renderer::OpenGl;
use femtovg::{Baseline, Canvas, Color, Paint, Path, Renderer, Solidity};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{
    ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor, PossiblyCurrentContext,
};
use glutin::display::{Display, GetGlDisplay, GlDisplay};
use glutin::prelude::GlSurface;
use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use std::fs::File;
use std::num::NonZeroU32;
use std::thread;
use tracing::error;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::window::{Window, WindowBuilder};

pub struct FemtovgRenderBackend {
    message_receiver: Receiver<RenderBackendMessage>,
    event_sender: Sender<UiEvent>,
}

impl FemtovgRenderBackend {
    pub fn new(
        message_receiver: Receiver<RenderBackendMessage>,
        event_sender: Sender<UiEvent>,
    ) -> Self {
        Self {
            message_receiver,
            event_sender,
        }
    }
    pub fn start(self) -> ! {
        let event_loop: EventLoop<RenderBackendMessage> =
            EventLoopBuilder::<RenderBackendMessage>::with_user_event().build();
        let event_loop_proxy = event_loop.create_proxy();
        thread::Builder::new()
            .name("Femtovg Forwarder".into())
            .spawn(move || loop {
                if let Ok(message) = self.message_receiver.recv() {
                    if let Err(err) = event_loop_proxy.send_event(message) {
                        error!("Event loop closed: {}", err);
                    }
                } else {
                    error!("Could not receive message");
                    return;
                }
            })
            .unwrap();
        let (context, gl_display, window, surface) = create_window(&event_loop);

        let renderer =
            unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s).cast()) }
                .expect("Cannot create renderer");

        let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
        canvas.set_size(1000, 600, window.scale_factor() as f32);
        File::open("assets/fonts/Roboto-Regular.ttf").unwrap();
        canvas.add_font("assets/fonts/Roboto-Regular.ttf").unwrap();
        let mut render_list: Vec<RenderCommand> = Vec::new();
        event_loop.run(move |event, _target, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        let mouse_position = Point::new(position.x as f32, position.y as f32);
                        self.event_sender
                            .send(UiEvent::mouse_move(mouse_position))
                            .unwrap()
                    }
                    WindowEvent::MouseInput {
                        state,
                        button: MouseButton::Left,
                        ..
                    } => self
                        .event_sender
                        .send(UiEvent::mouse_input(if state == ElementState::Pressed {
                            MouseEventKind::Pressed
                        } else {
                            MouseEventKind::Released
                        }))
                        .unwrap(),
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                },
                Event::RedrawRequested(_) => {
                    render(&context, &surface, &window, &mut canvas, &render_list);
                }
                Event::UserEvent(message) => {
                    render_list = message.render_commands;
                    window.request_redraw();
                }
                _ => {}
            }
        })
    }
}

fn create_window(
    event_loop: &EventLoop<RenderBackendMessage>,
) -> (
    PossiblyCurrentContext,
    Display,
    Window,
    Surface<WindowSurface>,
) {
    let window_builder = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1000., 600.))
        .with_title("viui");

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(event_loop, template, |mut configs| configs.next().unwrap())
        .unwrap();

    let window = window.unwrap();

    let gl_display = gl_config.display();

    let context_attributes =
        ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));

    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap()
    });

    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window.raw_window_handle(),
        NonZeroU32::new(1000).unwrap(),
        NonZeroU32::new(600).unwrap(),
    );

    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    (
        not_current_gl_context
            .take()
            .unwrap()
            .make_current(&surface)
            .unwrap(),
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
    render_commands: &[RenderCommand],
) {
    let size = window.inner_size();
    canvas.set_size(size.width, size.height, window.scale_factor() as f32);
    canvas.reset_transform();
    canvas.clear_rect(0, 0, size.width, size.height, Color::white());

    let mut fill_paint = Paint::color(Color::hsl(0.0, 0.0, 1.0))
        .with_text_baseline(Baseline::Middle)
        .with_font_size(20.0)
        .with_anti_alias(true);
    let mut stroke_paint = Paint::color(Color::hsl(0.0, 0.0, 0.0))
        .with_text_baseline(Baseline::Middle)
        .with_font_size(20.0)
        .with_anti_alias(true);
    for command in render_commands {
        match command {
            RenderCommand::FillRect { .. } => {}
            RenderCommand::FillRoundRect { rect, radius } => {
                let mut path = Path::new();
                path.rounded_rect(
                    rect.min_x(),
                    rect.min_y(),
                    rect.width(),
                    rect.height(),
                    *radius,
                );

                canvas.fill_path(&path, &fill_paint);
                canvas.stroke_path(&path, &stroke_paint);
            }
            RenderCommand::Translate { x, y } => {
                canvas.translate(*x, *y);
            }
            RenderCommand::ResetTransform => {
                canvas.reset_transform();
            }
            RenderCommand::DrawText(text) => {
                canvas.fill_text(0.0, 0.0, text, &stroke_paint).unwrap();
            }
            RenderCommand::Save => {
                canvas.save();
            }
            RenderCommand::Restore => {
                canvas.restore();
            }
            RenderCommand::SetFillColor(color) => {
                fill_paint.set_color(Color::rgba(color.r, color.g, color.b, color.a));
            }
            RenderCommand::SetStrokeColor(color) => {
                stroke_paint.set_color(Color::rgba(color.r, color.g, color.b, color.a));
            }
            RenderCommand::Line { start, end } => {
                let mut path = Path::new();
                path.move_to(start.x, start.y);
                path.line_to(end.x, end.y);
                canvas.stroke_path(&path, &stroke_paint);
            }
            RenderCommand::Arc {
                center,
                radius,
                start_angle,
                end_angle,
            } => {
                let mut path = Path::new();
                path.arc(
                    center.x,
                    center.y,
                    *radius,
                    *start_angle,
                    *end_angle,
                    Solidity::Hole,
                );
                canvas.stroke_path(&path, &stroke_paint);
            }
            RenderCommand::SetStrokeWidth(width) => {
                stroke_paint.set_line_width(*width);
            }
            RenderCommand::ClipRect(clip_rect) => {
                canvas.scissor(
                    clip_rect.origin.x,
                    clip_rect.origin.y,
                    clip_rect.size.width,
                    clip_rect.size.height,
                );
            }
        }
    }

    // Tell renderer to execute all drawing commands*/
    canvas.flush();
    // Display what we've just rendered
    surface
        .swap_buffers(context)
        .expect("Could not swap buffers");
}
