use crate::nodes::events::{MouseEventKind, UiEvent};
use crate::render::command::{ImageId, RenderCommand};
use crate::types::Point;
use crate::ui::RenderBackendMessage;
use crossbeam_channel::{Receiver, Sender};
use femtovg::renderer::OpenGl;
use femtovg::{Baseline, Canvas, Color, FontId, ImageFlags, Paint, Path, Solidity};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{
    ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor, PossiblyCurrentContext,
};
use glutin::display::{Display, GetGlDisplay, GlDisplay};
use glutin::prelude::GlSurface;
use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use log::info;
use raw_window_handle::HasRawWindowHandle;
use std::collections::HashMap;
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

pub struct RenderState<'a> {
    context: &'a PossiblyCurrentContext,
    surface: &'a Surface<WindowSurface>,
    window: &'a Window,
    canvas: &'a mut Canvas<OpenGl>,
    image_map: &'a mut HashMap<ImageId, femtovg::ImageId>,
    fonts: &'a [FontId],
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
        canvas.set_size(1200, 1200, window.scale_factor() as f32);
        //        canvas.add_font("assets/fonts/Roboto-Regular.ttf").unwrap();
        let font_id = canvas
            .add_font("assets/fonts/OpenSans-Regular.ttf")
            .unwrap();
        let fonts = vec![font_id];
        let mut render_list: Vec<RenderCommand> = Vec::new();
        let mut image_map = Default::default();
        event_loop.run(move |event, _target, control_flow| {
            let mut render_state = RenderState {
                context: &context,
                surface: &surface,
                window: &window,
                canvas: &mut canvas,
                image_map: &mut image_map,
                fonts: &fonts,
            };
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
                    WindowEvent::ReceivedCharacter(character) => self
                        .event_sender
                        .send(UiEvent::character_input(character))
                        .unwrap(),
                    _ => {}
                },
                Event::RedrawRequested(_) => {
                    render(&mut render_state, &render_list);
                }
                Event::UserEvent(message) => {
                    render_list = message.render_commands;
                    render(&mut render_state, &render_list);
                    //window.request_redraw();
                }
                _ => {}
            }
        });
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
        .with_inner_size(PhysicalSize::new(1200., 1200.))
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
        NonZeroU32::new(1200).unwrap(),
        NonZeroU32::new(1200).unwrap(),
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

fn render(render_state: &mut RenderState, render_commands: &[RenderCommand]) {
    let RenderState {
        window: _window,
        canvas,
        surface,
        context,
        image_map,
        fonts,
    } = render_state;
    canvas.reset_transform();

    let mut fill_paint = Paint::color(Color::hsl(0.0, 0.0, 1.0))
        .with_text_baseline(Baseline::Middle)
        .with_font_size(20.0)
        .with_font(fonts)
        .with_anti_alias(true);
    let mut stroke_paint = Paint::color(Color::hsl(0.0, 0.0, 0.0))
        .with_text_baseline(Baseline::Middle)
        .with_font(fonts)
        .with_font_size(25.0)
        .with_line_width(0.5)
        .with_anti_alias(true);
    for command in render_commands {
        match command {
            RenderCommand::FillRect { rect } => {
                let mut path = Path::new();
                path.rect(rect.min_x(), rect.min_y(), rect.width(), rect.height());

                canvas.fill_path(&path, &fill_paint);
            }
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
                stroke_paint.set_line_width(1.0);
                stroke_paint.set_anti_alias(true);
                stroke_paint.set_text_baseline(Baseline::Bottom);
                canvas.fill_text(0.0, 10.0, text, &stroke_paint).unwrap();
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
            RenderCommand::LoadImage { image_id, resource } => {
                info!("Loading image: {:?}", resource);
                let femto_id = canvas
                    .load_image_mem(&resource.as_bytes().unwrap(), ImageFlags::empty())
                    .unwrap();
                image_map.insert(*image_id, femto_id);
            }
            RenderCommand::DrawImage { image_id } => {
                let femto_img = image_map[image_id];
                let (iw, ih) = canvas.image_size(femto_img).unwrap();
                let img_paint = Paint::image(femto_img, 0.0, 0.0, iw as f32, ih as f32, 0.0, 1.0);
                let mut path = Path::new();
                path.rect(0.0, 0.0, iw as f32, ih as f32);
                canvas.fill_path(&path, &img_paint);
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
