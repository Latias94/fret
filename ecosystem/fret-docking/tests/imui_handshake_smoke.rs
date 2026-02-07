#![cfg(feature = "imui")]

use fret_authoring::UiWriter;
use fret_core::AppWindowId;
use fret_docking::DockingRuntime;
use fret_docking::imui::{DockSpaceImUiOptions, dock_space_with};
use fret_ui::UiHost;

#[allow(dead_code)]
fn docking_imui_handshake_compiles<H: UiHost + 'static>(ui: &mut impl UiWriter<H>) {
    dock_space_with(ui, DockSpaceImUiOptions::default(), |_app, _window| {});

    let runtime = DockingRuntime::new(AppWindowId::default());
    let _ = runtime.main_window();
}
