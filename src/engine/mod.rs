use crate::ast::parser::parse_ui;
use crate::eval::tree::{eval_component, EvalNode};
use crate::infrastructure::font_pool::{FontIndex, FontPool};
use crate::infrastructure::image_pool::ImagePool;
use crate::ir::node::ast_to_ir;
use crate::nodes::events::UiEvent;
use crate::render::backend::RenderBackendParameters;
use crate::render::command::{RenderCommand, RenderCommands};
use crate::render::context::RenderContext;
use crate::resource::Resource;
use crate::result::ViuiResult;
use crate::types::{Color, Point, Rect, Size};
use crate::ui::RenderBackendMessage;
use crate::widget::label::LabelWidget;
use crate::widget::Widget;
use crossbeam_channel::{select, tick, Receiver, Sender};
use std::mem::take;
use std::thread;
use std::time::Duration;
use tracing::{debug, error};

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
    pub fn new() -> ViuiResult<UIEngine> {
        let mut font_pool = FontPool::default();
        font_pool.load_font(Resource::from_path("assets/fonts/OpenSans-Regular.ttf"))?;

        let (ui_event_sender, ui_event_receiver) = crossbeam_channel::bounded::<UiEvent>(4);
        Ok(UIEngine {
            image_pool: ImagePool::default(),
            font_pool,
            render_backends: Vec::new(),
            ui_event_receiver,
            ui_event_sender,
        })
    }

    pub fn start(mut self) -> ViuiResult<()> {
        thread::Builder::new()
            .name("VIUI Thread".into())
            .spawn(move || {
                debug!("Running main loop");
                let ticker = tick(Duration::from_micros(1_000_000 / 60));
                loop {
                    let result: ViuiResult<()> = (|| {
                        select! {
                            recv(self.ui_event_receiver) -> event => {
                                self.handle_ui_event(event?)?;
                                self.eval_layout_and_redraw()?;
                            }
                            /*
                            recv(self.file_change_receiver) -> _event => {
                                self.load_root_node_file()?;
                                self.eval_layout_and_redraw()?;
                            }
                            recv(ticker) -> _ => {
                                if !self.animated_nodes.is_empty() {
                                    self.redraw()?;
                                }
                            }*/
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
            let mut render_commands = vec![];

            let tree_render_commands = self.render_commands(&evaled)?;
            let maximum_font_index = self.font_pool.maximum_font_index();
            if maximum_font_index > backend.maximum_font_index_loaded {
                for (font_index, font) in self
                    .font_pool
                    .get_fonts_from(backend.maximum_font_index_loaded)
                {
                    render_commands.push(RenderCommand::LoadFont {
                        font_idx: font_index,
                        resource: font.resource().clone(),
                    });
                }
                backend.maximum_font_index_loaded = maximum_font_index;
            }
            render_commands.push(RenderCommand::SetFont {
                font_idx: FontIndex::new(0),
            });
            render_commands.extend_from_slice(&tree_render_commands);
            backend
                .render_backend_sender
                .send(RenderBackendMessage { render_commands })
                .unwrap();
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
        LabelWidget {}.render(&mut render_context)?;
        Ok(render_context.render_queue())
    }

    fn handle_ui_event(&self, ui_event: UiEvent) -> ViuiResult<()> {
        Ok(())
    }
}
