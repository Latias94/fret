use super::{
    TextAssistFieldSurface, should_render_inline_empty_label, text_assist_max_content_height,
};
use fret_core::Px;

const TEXT_ASSIST_BODY_RS: &str = include_str!("element/body.rs");
const TEXT_ASSIST_OVERLAY_RS: &str = include_str!("overlay.rs");

#[test]
fn empty_label_is_inline_only() {
    assert!(should_render_inline_empty_label(
        TextAssistFieldSurface::Inline,
        "cube",
        0,
    ));
    assert!(!should_render_inline_empty_label(
        TextAssistFieldSurface::AnchoredOverlay,
        "cube",
        0,
    ));
}

#[test]
fn anchored_overlay_defaults_to_capped_content_height() {
    let max_height =
        text_assist_max_content_height(TextAssistFieldSurface::AnchoredOverlay, None, Px(28.0));
    assert_eq!(max_height, Some(Px(178.0)));
}

#[test]
fn anchored_overlay_never_falls_back_to_inline_layout_flow() {
    assert!(TEXT_ASSIST_BODY_RS.contains("request_text_assist_overlay("));
    assert!(!TEXT_ASSIST_BODY_RS.contains("None => Some(panel)"));
    assert!(!TEXT_ASSIST_OVERLAY_RS.contains("-> Option<AnyElement>"));
    assert!(!TEXT_ASSIST_OVERLAY_RS.contains("return Some(panel)"));
    assert!(TEXT_ASSIST_OVERLAY_RS.contains("cx.app.request_redraw(cx.window);"));
}
