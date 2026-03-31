#![cfg(feature = "imui")]

use fret_ui::UiHost;
use fret_ui_kit::imui::{DragSourceResponse, UiWriterImUiFacadeExt};
use fret_ui_kit::recipes::imui_drag_preview::{
    DragPreviewGhostOptions, drag_preview_ghost, drag_preview_ghost_with_options,
    publish_cross_window_drag_preview_ghost, publish_cross_window_drag_preview_ghost_with_options,
    render_cross_window_drag_preview_ghosts,
};

#[allow(dead_code)]
fn drag_preview_recipe_api_compiles<H: UiHost + 'static>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let source = DragSourceResponse::default();

    let _: bool = drag_preview_ghost(
        ui,
        "ghost.default",
        source,
        fret_ui_kit::ui::container(|cx| vec![cx.text("Ghost")]),
    );
    let _: bool = drag_preview_ghost_with_options(
        ui,
        "ghost.options",
        source,
        DragPreviewGhostOptions::default(),
        fret_ui_kit::ui::container(|cx| vec![cx.text("Ghost")]),
    );
    let _: bool =
        publish_cross_window_drag_preview_ghost(ui, "ghost.cross_window", source, |_cx| {
            fret_ui_kit::ui::container(|cx| vec![cx.text("Ghost")])
        });
    let _: bool = publish_cross_window_drag_preview_ghost_with_options(
        ui,
        "ghost.cross_window.options",
        source,
        DragPreviewGhostOptions::default(),
        |_cx| fret_ui_kit::ui::container(|cx| vec![cx.text("Ghost")]),
    );

    ui.with_cx_mut(|cx| {
        let _: bool = render_cross_window_drag_preview_ghosts(cx);
    });
}

#[test]
fn drag_preview_option_defaults_compile() {
    let options = DragPreviewGhostOptions::default();
    assert!(options.enabled);
    assert_eq!(
        options.offset,
        fret_core::Point::new(fret_core::Px(12.0), fret_core::Px(12.0))
    );
    assert!((options.opacity - 0.9).abs() <= f32::EPSILON);
    assert!(options.test_id.is_none());
}
