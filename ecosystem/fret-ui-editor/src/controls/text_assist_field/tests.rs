use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::element::{AnyElement, ElementKind};
use fret_ui::elements::with_element_cx;
use fret_ui_kit::headless::text_assist::TextAssistItem;

use super::{
    TextAssistField, TextAssistFieldOptions, TextAssistFieldSurface,
    should_clear_text_assist_dismissal_on_focus_gain, should_render_inline_empty_label,
    text_assist_field_expanded, text_assist_max_content_height,
};

const TEXT_ASSIST_BODY_RS: &str = include_str!("element/body.rs");
const TEXT_ASSIST_OVERLAY_RS: &str = include_str!("overlay.rs");

fn render_text_assist_field(
    surface: TextAssistFieldSurface,
    query_value: &str,
    items: Arc<[TextAssistItem]>,
) -> AnyElement {
    let mut app = App::new();
    let window = AppWindowId::default();
    let query = app.models_mut().insert(query_value.to_string());
    let dismissed_query = app.models_mut().insert(String::new());
    let active_item_id = app.models_mut().insert(None::<Arc<str>>);

    with_element_cx(
        &mut app,
        window,
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        ),
        "text-assist-field",
        |cx| {
            TextAssistField::new(
                query.clone(),
                dismissed_query.clone(),
                active_item_id.clone(),
                items.clone(),
            )
            .options(TextAssistFieldOptions {
                surface,
                ..Default::default()
            })
            .into_element(cx)
        },
    )
}

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
fn inline_surface_without_panel_or_empty_label_returns_the_field_root() {
    let items: Arc<[TextAssistItem]> = Vec::new().into();

    let inline = render_text_assist_field(TextAssistFieldSurface::Inline, "", items);

    assert!(!matches!(inline.kind, ElementKind::Flex(_)));
}

#[test]
fn anchored_overlay_surface_without_panel_or_empty_label_returns_the_field_root() {
    let items: Arc<[TextAssistItem]> = Vec::new().into();

    let overlay = render_text_assist_field(TextAssistFieldSurface::AnchoredOverlay, "", items);

    assert!(!matches!(overlay.kind, ElementKind::Flex(_)));
}

#[test]
fn inline_surface_with_empty_label_keeps_the_shell_visible() {
    let items: Arc<[TextAssistItem]> = Vec::new().into();

    let inline = render_text_assist_field(TextAssistFieldSurface::Inline, "cube", items);

    assert!(matches!(inline.kind, ElementKind::Flex(_)));
    assert_eq!(inline.children.len(), 2);
    assert!(matches!(inline.children[1].kind, ElementKind::Text(_)));
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
