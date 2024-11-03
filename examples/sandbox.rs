use bevy_reflect::{ParsedPath, Reflect};
use log::{error, info};
use viui::logging::init_logging;
use viui::nodes::elements::button::ButtonElement;
use viui::observable_state::{ObservableState, TypedPath};
use viui::render::backend_femtovg::FemtovgRenderBackend;
use viui::result::ViuiResult;
use viui::ui::UI;

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
    if let Err(error) = main_internal() {
        error!("Aborted with error: {:?}", error);
        std::process::exit(1);
    }
}
fn main_internal() -> ViuiResult<()> {
    init_logging()?;
    info!("VIUI Sandbox starting");

    let app_state = ObservableState::new(AppState { counter: 19 });
    let counter_path = TypedPath::<i32>::new(ParsedPath::parse("counter")?);
    let mut ui = UI::new(
        app_state,
        move |app_state, message: &AppMessage| match message {
            AppMessage::Increment => {
                app_state.apply_change("Increment", |mutator| {
                    mutator.mutate(&counter_path, |counter| *counter += 1);
                });
            }
            AppMessage::Decrement => {
                app_state.apply_change("Decrement", |mutator| {
                    mutator.mutate(&counter_path, |counter| *counter -= 1);
                });
            }
        },
    )?;
    ui.register_node::<ButtonElement>();
    ui.set_root_node_file("counter.viui.yaml")?;
    let render_backend = FemtovgRenderBackend::new(ui.add_render_backend()?, ui.event_sender());
    ui.start()?;
    info!("VIUI Sandbox started");
    render_backend.start();
}
