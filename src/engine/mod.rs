use crate::arenal::Arenal;
use crate::ast::parser::parse_ui;
use crate::eval::tree::{eval_component, EvalNode, EvalNodeIdx};
use crate::infrastructure::font_pool::{FontIndex, FontPool};
use crate::infrastructure::image_pool::ImagePool;
use crate::ir::node::{ast_to_ir, IrComponent, WidgetRegistry};
use crate::nodes::data::NodeIdx;
use crate::nodes::elements::kind::LayoutConstraints;
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
use std::ops::Index;
use std::thread;
use std::time::Duration;
use taffy::prelude::length;
use taffy::{FlexDirection, NodeId, Style, TaffyTree};
use tracing::{debug, error};

struct RenderBackend {
    render_backend_sender: Sender<RenderBackendMessage>,
    maximum_font_index_loaded: usize,
    window_size: Size,
}

pub struct UIEngineStarter {
    render_backends: Vec<RenderBackend>,
    ui_event_receiver: Receiver<UiEvent>,
    ui_event_sender: Sender<UiEvent>,
}

pub struct UIEngine {
    image_pool: ImagePool,
    font_pool: FontPool,
    render_backends: Vec<RenderBackend>,
    ui_event_receiver: Receiver<UiEvent>,
    ui_event_sender: Sender<UiEvent>,
    ir: Vec<IrComponent>,
    eval_node_arenal: Arenal<EvalNode>,
    widget_registry: WidgetRegistry,
}

impl UIEngineStarter {
    pub fn new() -> ViuiResult<Self> {
        let (ui_event_sender, ui_event_receiver) = crossbeam_channel::bounded::<UiEvent>(4);
        Ok(UIEngineStarter {
            render_backends: Vec::new(),
            ui_event_receiver,
            ui_event_sender,
        })
    }

    pub fn start(mut self) -> ViuiResult<()> {
        thread::Builder::new()
            .name("VIUI Thread".into())
            .spawn(move || {
                let result: ViuiResult<()> = (|| {
                    let ui = UIEngine::new(self)?;
                    debug!("Running main loop");
                    ui.run();
                    Ok(())
                })();
                if let Err(err) = result {
                    error!("Error in VIUI Thread: {:?}", err);
                    std::process::exit(1);
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
}

impl UIEngine {
    pub fn new(starter: UIEngineStarter) -> ViuiResult<Self> {
        let mut widget_registry = WidgetRegistry::default();

        widget_registry.register_widget("label".to_string(), LabelWidget {});

        let source = std::fs::read_to_string("examples/simple/labels.viui-component").unwrap();
        let ast = parse_ui(&source)?;
        let ir = ast_to_ir(&ast, &widget_registry)?;

        let mut font_pool = FontPool::default();
        font_pool.load_font(Resource::from_path("assets/fonts/OpenSans-Regular.ttf"))?;

        Ok(UIEngine {
            ir,
            image_pool: ImagePool::default(),
            font_pool,
            render_backends: starter.render_backends,
            ui_event_receiver: starter.ui_event_receiver,
            ui_event_sender: starter.ui_event_sender,
            eval_node_arenal: Arenal::new(),
            widget_registry,
        })
    }

    pub fn run(mut self) {
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
    }

    pub fn eval_layout_and_redraw(&mut self) -> ViuiResult<()> {
        self.eval_node_arenal.clear();
        let evaled_idx = eval_component(&self.ir[0], &mut self.eval_node_arenal)?;
        let mut render_backends = take(&mut self.render_backends);
        for backend in &mut render_backends {
            let mut tree: TaffyTree<EvalNodeIdx> = TaffyTree::new();
            let root_layout_node = tree.new_leaf_with_context(
                Style {
                    flex_direction: FlexDirection::Column,
                    size: taffy::Size {
                        width: length(backend.window_size.width),
                        height: length(backend.window_size.height),
                    },
                    ..Default::default()
                },
                evaled_idx,
            )?;
            let mut todo: Vec<(NodeId, EvalNodeIdx)> = vec![];
            for child in self.eval_node_arenal[&evaled_idx].children().iter().rev() {
                todo.push((root_layout_node, *child));
            }
            while let Some((parent_layout_id, node_idx)) = todo.pop() {
                let node = &mut self.eval_node_arenal[&node_idx];
                let style = Style {
                    size: taffy::Size {
                        width: length(10.0),
                        height: length(100.0),
                    },
                    ..Default::default()
                };
                let layout_node = tree.new_leaf_with_context(style, node_idx)?;
                tree.add_child(parent_layout_id, layout_node)?;
                for child in node.children() {
                    todo.push((layout_node, *child));
                }
            }
            tree.compute_layout(root_layout_node, taffy::Size::max_content())?;
            //dbg!(tree.layout(root_layout_node)?);

            // Set absolute position and bounds for each node
            let mut todo = vec![(0.0, 0.0, root_layout_node)];
            while let Some((parent_x, parent_y, node_id)) = todo.pop() {
                let node_index = tree.get_node_context(node_id).unwrap();
                let node = &mut self.eval_node_arenal[node_index];
                let layout = tree.layout(node_id)?;
                let x = parent_x + layout.location.x;
                let y = parent_y + layout.location.y;
                node.layout.bounds = Rect::new(
                    Point::new(x, y),
                    Size::new(layout.size.width, layout.size.height),
                );
                for child in tree.children(node_id)? {
                    todo.push((x, y, child));
                }
            }

            let mut render_list = vec![];

            let mut render_context =
                RenderContext::new(&mut self.image_pool, &mut self.font_pool, 0.0)?;
            render_context.add_command(RenderCommand::SetFillColor(Color::gray(10)));
            let size = Size::new(1200.0, 1200.0);
            render_context.add_command(RenderCommand::SetWindowSize { size: size.clone() });

            render_commands(evaled_idx, &mut render_context, &self.eval_node_arenal)?;
            let mut widget_render_list = render_context.render_queue();
            let maximum_font_index = self.font_pool.maximum_font_index();
            if maximum_font_index > backend.maximum_font_index_loaded {
                for (font_index, font) in self
                    .font_pool
                    .get_fonts_from(backend.maximum_font_index_loaded)
                {
                    render_list.push(RenderCommand::LoadFont {
                        font_idx: font_index,
                        resource: font.resource().clone(),
                    });
                }
                backend.maximum_font_index_loaded = maximum_font_index;
            }
            render_list.push(RenderCommand::SetFont {
                font_idx: FontIndex::new(0),
            });

            render_list.append(&mut widget_render_list);
            backend
                .render_backend_sender
                .send(RenderBackendMessage {
                    render_commands: render_list,
                })
                .unwrap();
        }
        self.render_backends = render_backends;
        Ok(())
    }

    fn handle_ui_event(&self, ui_event: UiEvent) -> ViuiResult<()> {
        Ok(())
    }
}

pub fn render_commands(
    node_idx: EvalNodeIdx,
    render_context: &mut RenderContext,
    eval_node_arenal: &Arenal<EvalNode>,
) -> ViuiResult<()> {
    render_context.add_command(RenderCommand::Save);
    let eval_node = &eval_node_arenal[&node_idx];
    let translate = eval_node.layout.bounds.origin;
    render_context.add_command(RenderCommand::Translate {
        x: translate.x,
        y: translate.y,
    });
    eval_node.widget.render(render_context, eval_node.props())?;
    for child in eval_node.children() {
        render_commands(*child, render_context, eval_node_arenal)?
    }
    render_context.add_command(RenderCommand::Restore);
    Ok(())
}
