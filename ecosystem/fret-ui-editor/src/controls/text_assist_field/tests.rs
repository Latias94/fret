use super::{
    TextAssistFieldSurface, should_clear_text_assist_dismissal_on_focus_gain,
    should_render_inline_empty_label, text_assist_field_expanded, text_assist_max_content_height,
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
fn anchored_overlay_requires_input_focus_before_expanding() {
    assert!(text_assist_field_expanded(
        TextAssistFieldSurface::Inline,
        "ca",
        "",
        2,
        false,
    ));
    assert!(!text_assist_field_expanded(
        TextAssistFieldSurface::AnchoredOverlay,
        "ca",
        "",
        2,
        false,
    ));
    assert!(text_assist_field_expanded(
        TextAssistFieldSurface::AnchoredOverlay,
        "ca",
        "",
        2,
        true,
    ));
}

#[test]
fn anchored_overlay_never_falls_back_to_inline_layout_flow() {
    assert!(TEXT_ASSIST_BODY_RS.contains("request_text_assist_overlay("));
    assert!(!TEXT_ASSIST_BODY_RS.contains("None => Some(panel)"));
    assert!(!TEXT_ASSIST_OVERLAY_RS.contains("-> Option<AnyElement>"));
    assert!(!TEXT_ASSIST_OVERLAY_RS.contains("return Some(panel)"));
    assert!(TEXT_ASSIST_OVERLAY_RS.contains("cx.app.request_redraw(cx.window);"));
}

#[test]
fn focus_gain_clears_same_query_dismissal_when_matches_remain() {
    assert!(should_clear_text_assist_dismissal_on_focus_gain(
        "ca", "ca", 2, false, true,
    ));
}

#[test]
fn focus_gain_keeps_dismissal_without_a_reopen_edge_or_matches() {
    assert!(!should_clear_text_assist_dismissal_on_focus_gain(
        "ca", "ca", 2, true, true,
    ));
    assert!(!should_clear_text_assist_dismissal_on_focus_gain(
        "ca", "ca", 2, false, false,
    ));
    assert!(!should_clear_text_assist_dismissal_on_focus_gain(
        "ca", "c", 2, false, true,
    ));
    assert!(!should_clear_text_assist_dismissal_on_focus_gain(
        "ca", "ca", 0, false, true,
    ));
    assert!(!should_clear_text_assist_dismissal_on_focus_gain(
        " ", " ", 2, false, true,
    ));
}
