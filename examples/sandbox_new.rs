use bevy_reflect::{ParsedPath, Reflect};
use log::{error, info};
use serde::{Deserialize, Serialize};
use viui::engine::UIEngine;
use viui::logging::init_logging;
use viui::observable_state::{ObservableState, TypedPath};
use viui::render::backend_femtovg::FemtovgRenderBackend;
use viui::result::ViuiResult;

fn main() {
    if let Err(error) = main_internal() {
        error!("Aborted with error: {:?}", error);
        std::process::exit(1);
    }
}

fn main_internal() -> ViuiResult<()> {
    init_logging()?;
    info!("VIUI Sandbox starting");

    let mut ui = UIEngine::new();
    let render_backend = FemtovgRenderBackend::new(ui.add_render_backend()?);
    //    ui.start()?;
    info!("VIUI Sandbox started");
    render_backend.start();
}
