#![cfg(feature = "imui")]

use fret_authoring::Response;
use fret_core::{Point, Rect, Size};
use fret_ui_kit::imui::ResponseExt;

fn to_shared_response(response: ResponseExt) -> Response {
    response.core
}

#[test]
fn shared_and_facade_response_boundary_compiles() {
    let response = ResponseExt {
        core: Response {
            hovered: true,
            pressed: false,
            focused: false,
            clicked: true,
            changed: false,
            rect: Some(Rect::new(
                Point::new(0.0.into(), 0.0.into()),
                Size::new(8.0.into(), 4.0.into()),
            )),
        },
        secondary_clicked: true,
        double_clicked: false,
        context_menu_requested: true,
        ..ResponseExt::default()
    };

    let shared = to_shared_response(response);
    assert!(shared.clicked());
    assert!(!shared.changed());

    assert!(response.secondary_clicked());
    assert!(!response.double_clicked());
    assert!(response.context_menu_requested());
}

#[test]
fn facade_drag_and_long_press_accessors_compile() {
    let response = ResponseExt::default();

    let _ = response.drag_started();
    let _ = response.dragging();
    let _ = response.drag_stopped();
    let _ = response.drag_delta();
    let _ = response.drag_total();
    let _ = response.long_pressed();
    let _ = response.press_holding();
}
