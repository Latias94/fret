#![cfg(feature = "imui")]

use fret_ui::UiHost;
use fret_ui_kit::imui::{
    DragSourceOptions, DragSourceResponse, DropTargetOptions, DropTargetResponse, ResponseExt,
    UiWriterImUiFacadeExt,
};

#[derive(Clone)]
#[allow(dead_code)]
struct DemoPayload {
    label: &'static str,
}

#[allow(dead_code)]
fn drag_drop_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let trigger = ResponseExt::default();

    let source: DragSourceResponse = ui.drag_source(trigger, DemoPayload { label: "Cube" });
    let cross_window_source: DragSourceResponse = ui.drag_source_with_options(
        trigger,
        DemoPayload { label: "Sphere" },
        DragSourceOptions {
            cross_window: true,
            ..Default::default()
        },
    );
    assert!(!source.active());
    assert!(!source.cross_window());
    assert!(source.position().is_none());
    assert!(source.pointer_id().is_none());
    assert!(source.session_id().is_none());
    assert!(!cross_window_source.active());

    let target: DropTargetResponse<DemoPayload> = ui.drop_target(trigger);
    let target_with_options: DropTargetResponse<DemoPayload> =
        ui.drop_target_with_options(trigger, DropTargetOptions::default());
    assert!(!target.active());
    assert!(!target.over());
    assert!(!target.delivered());
    assert!(target.preview_position().is_none());
    assert!(target.delivered_position().is_none());
    assert!(target.preview_payload().is_none());
    assert!(target.delivered_payload().is_none());
    assert!(!target_with_options.active());
}

#[test]
fn drag_drop_option_defaults_compile() {
    let source = DragSourceOptions::default();
    assert!(source.enabled);
    assert!(!source.cross_window);

    let target = DropTargetOptions::default();
    assert!(target.enabled);
}
