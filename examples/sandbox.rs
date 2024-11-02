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
use viui::model::ComponentNode;
use viui::observable_state::{ObservableState, TypedPath};
use viui::render::backend_femtovg::FemtovgRenderBackend;
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
    ui.set_root_node_file("counter.viui.yaml");
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
    let render_backend = FemtovgRenderBackend::new(ui.add_render_backend(), ui.event_sender());
    ui.start();
    render_backend.start();
}

