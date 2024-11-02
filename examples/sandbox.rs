use bevy_reflect::{ParsedPath, Reflect};

use viui::observable_state::{ObservableState, TypedPath};
use viui::render::backend_femtovg::FemtovgRenderBackend;
use viui::result::ViuiResult;
use viui::ui::UI;
use viui::widget_model::{ButtonWidget, WidgetRegistry};

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
        println!("Aborted with error: {:?}", error);
        std::process::exit(1);
    }
}
fn main_internal() -> ViuiResult<()> {
    println!("Starting VIUI");

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
    })?;
    ui.register_widget::<ButtonWidget>();
    ui.set_root_node_file("counter.viui.yaml")?;
    /*

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
*/
    let render_backend = FemtovgRenderBackend::new(ui.add_render_backend()?, ui.event_sender());
    ui.start()?;
    render_backend.start();
}

