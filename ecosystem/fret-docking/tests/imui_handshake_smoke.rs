#![cfg(feature = "imui")]

use fret_authoring::UiWriter;
use fret_core::AppWindowId;
use fret_docking::DockingRuntime;
use fret_docking::imui::{DockSpaceElementOptions, dock_space_declarative_with};
use fret_ui::UiHost;

#[allow(dead_code)]
fn docking_imui_handshake_compiles<H: UiHost + 'static>(ui: &mut impl UiWriter<H>) {
    dock_space_declarative_with(ui, DockSpaceElementOptions::default());

    let runtime = DockingRuntime::new(AppWindowId::default());
    let _ = runtime.main_window();
}

#[test]
fn imui_handshake_smoke_test_binary_is_non_empty() {
    let options = DockSpaceElementOptions::default();
    assert!(options.test_id.is_none());
}
