use crate::ast::parser::parse_ui;
use crate::eval::tree::{eval_component, EvalNode};
use crate::infrastructure::font_pool::FontPool;
use crate::infrastructure::image_pool::ImagePool;
use crate::ir::node::ast_to_ir;
use crate::nodes::events::UiEvent;
use crate::render::backend::RenderBackendParameters;
use crate::render::command::{RenderCommand, RenderCommands};
use crate::render::context::RenderContext;
use crate::result::ViuiResult;
use crate::types::{Color, Point, Rect, Size};
use crate::ui::RenderBackendMessage;
use crossbeam_channel::{Receiver, Sender};
use std::mem::take;

struct RenderBackend {
    render_backend_sender: Sender<RenderBackendMessage>,
    maximum_font_index_loaded: usize,
    window_size: Size,
}

pub struct UIEngine {
    image_pool: ImagePool,
    font_pool: FontPool,
    render_backends: Vec<RenderBackend>,
    ui_event_receiver: Receiver<UiEvent>,
    ui_event_sender: Sender<UiEvent>,
}

impl UIEngine {
    pub fn new() -> UIEngine {
        let (ui_event_sender, ui_event_receiver) = crossbeam_channel::bounded::<UiEvent>(4);
        UIEngine {
            image_pool: ImagePool::default(),
            font_pool: FontPool::default(),
            render_backends: Vec::new(),
            ui_event_receiver,
            ui_event_sender,
        }
    }
    /*
        pub fn start(mut self) -> ViuiResult<()> {
            thread::Builder::new()
                .name("VIUI Thread".into())
                .spawn(move || {
                    debug!("Running main loop");
                    let ticker = tick(Duration::from_micros(1_000_000 / 60));
                    loop {
                        let result: ViuiResult<()> = (|| {
                            select! {
                                recv(self.file_change_receiver) -> _event => {
                                    self.load_root_node_file()?;
                                    self.eval_layout_and_redraw()?;
                                }
                                recv(self.ui_event_receiver) -> event => {
                                    self.handle_ui_event(event?)?;
                                    self.eval_layout_and_redraw()?;
                                }
                                recv(ticker) -> _ => {
                                    if !self.animated_nodes.is_empty() {
                                        self.redraw()?;
                                    }
                                }
                            }
                            Ok(())
                        })();
                        if let Err(err) = result {
                            error!("Error in VIUI Thread: {:?}", err);
                            std::process::exit(1);
                        }
                    }
                })?;
            Ok(())
        }
    */
    pub fn add_render_backend(&mut self) -> ViuiResult<RenderBackendParameters> {
        let (render_backend_sender, message_receiver) =
            crossbeam_channel::bounded::<RenderBackendMessage>(4);
        let backend_index = self.render_backends.len();
        self.render_backends.push(RenderBackend {
            render_backend_sender,
            maximum_font_index_loaded: 0,
            window_size: Size::new(1200.0, 1200.0),
        });
        self.eval_layout_and_redraw()?;
        Ok(RenderBackendParameters {
            message_receiver,
            event_sender: self.event_sender(),
            backend_index,
            initial_window_size: Size::new(1200.0, 1200.0),
        })
    }

    pub fn event_sender(&self) -> Sender<UiEvent> {
        self.ui_event_sender.clone()
    }

    pub fn eval_layout_and_redraw(&mut self) -> ViuiResult<()> {
        let source = std::fs::read_to_string("examples/simple/label.viui-component")?;
        let ast = parse_ui(&source)?;
        let ir = ast_to_ir(&ast)?;
        let evaled = eval_component(&ir[0])?;
        let mut render_backends = take(&mut self.render_backends);
        for backend in &mut render_backends {
            let render_commands = self.render_commands(&evaled)?;
            println!("Rendering {} commands", render_commands.len());
            backend
                .render_backend_sender
                .send(RenderBackendMessage { render_commands })
                .unwrap();
            println!("Rendered");
        }
        self.render_backends = render_backends;
        Ok(())
    }

    pub fn render_commands(&mut self, evaled: &EvalNode) -> ViuiResult<RenderCommands> {
        let mut render_context =
            RenderContext::new(&mut self.image_pool, &mut self.font_pool, 0.0)?;
        render_context.add_command(RenderCommand::SetFillColor(Color::gray(10)));
        let size = Size::new(1200.0, 1200.0);
        render_context.add_command(RenderCommand::SetWindowSize { size: size.clone() });
        render_context.add_command(RenderCommand::FillRect {
            rect: Rect::new(Point::new(0.0, 0.0), size),
        });

        render_context.add_command(RenderCommand::SetStrokeColor(Color::gray(127)));
        let stroke_width = 2.0f32;
        render_context.add_command(RenderCommand::SetStrokeWidth(stroke_width));
        render_context.add_command(RenderCommand::FillRoundRect {
            rect: Rect::new(
                Point::new(stroke_width, stroke_width),
                Size::new(200.0 - stroke_width * 2.0, 40.0 - stroke_width * 2.0),
            ),
            radius: 5.0,
        });
        Ok(render_context.render_queue())
    }
}
