use bevy_reflect::{ParsedPath, Reflect};
use log::{error, info};
use serde::{Deserialize, Serialize};
use viui::logging::init_logging;
use viui::observable_state::{ObservableState, TypedPath};
use viui::render::backend_femtovg::FemtovgRenderBackend;
use viui::result::ViuiResult;
use viui::types::Float;
use viui::ui::UI;

#[derive(Debug, Reflect)]
struct AppState {
    counter: i32,
    gain: Float,
    name: String,
    show_image: bool,
    nicknames: Vec<String>,
    counters: Vec<i32>,
}

#[derive(Debug, Reflect, Serialize, Deserialize)]
enum AppMessage {
    Increment,
    Decrement,
    Set(Float),
    SetName(String),
    Change(Float),
    ToggleImage,
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

    let app_state = ObservableState::new(AppState {
        counter: 3,
        gain: 3.0,
        name: "Bob".to_string(),
        nicknames: vec!["Foo".to_string(), "Bar".to_string(), "Baz".to_string()],
        show_image: false,
        counters: vec![1, 2, 3],
    });
    let counter_path = TypedPath::<i32>::new(ParsedPath::parse("counter")?);
    let gain_path = TypedPath::<Float>::new(ParsedPath::parse("gain")?);
    let name_path = TypedPath::<String>::new(ParsedPath::parse("name")?);
    let show_image_path = TypedPath::<bool>::new(ParsedPath::parse("show_image")?);
    let counter_list_path = TypedPath::<i32>::new(ParsedPath::parse("counters[0]")?);
    let mut ui = UI::new(
        app_state,
        "CounterComponent".to_string(),
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
            AppMessage::Set(value) => {
                app_state.apply_change(format!("Set to {}", value), |mutator| {
                    mutator.mutate(&gain_path, |gain| *gain = *value);
                });
            }
            AppMessage::SetName(new_name) => {
                app_state.apply_change("Set name", |mutator| {
                    mutator.mutate(&name_path, |name| *name = new_name.to_string());
                });
            }
            AppMessage::ToggleImage => {
                app_state.apply_change("Toggle image", |mutator| {
                    mutator.mutate(&show_image_path, |show_image| *show_image = !*show_image);
                });
            }
            AppMessage::Change(by) => {
                app_state.apply_change(format!("Change by {}", by), |mutator| {
                    mutator.mutate(&counter_list_path, |counter| *counter += *by as i32);
                });
            }
        },
    )?;
    ui.set_root_node_file("counter.viui-component")?;
    let render_backend = FemtovgRenderBackend::new(ui.add_render_backend()?);
    ui.start()?;
    info!("VIUI Sandbox started");
    render_backend.start();
}
