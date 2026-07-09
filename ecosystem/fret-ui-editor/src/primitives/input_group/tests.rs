use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Color, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::element::{ElementKind, Length};
use fret_ui::elements;

use super::editor_input_value_text;
use super::joined::{editor_joined_input_frame, editor_joined_input_frame_segments_with_overrides};
use super::segments::{
    editor_clear_button_segment, editor_icon_button_segment, editor_input_group_row,
};
use crate::primitives::EditorDensity;
use crate::primitives::style::EditorStyle;
use fret_ui::Theme;

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

#[test]
fn editor_input_value_text_is_single_line_and_shrinkable() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let color = Color::from_srgb_hex_rgb(0xDD_EE_FF);

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        editor_input_value_text(
            cx,
            EditorDensity::default(),
            Px(12.0),
            Arc::from("123456789.123456789"),
            color,
            Length::Fill,
        )
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected editor_input_value_text(...) to build a Text element");
    };

    assert_eq!(props.color, Some(color));
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.size.height, Length::Fill);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.layout.flex.grow, 1.0);
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
}

#[test]
fn editor_icon_button_segment_has_no_inner_flex_shell() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let density = EditorDensity::default();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        editor_icon_button_segment(
            cx,
            density,
            true,
            Arc::from("Clear"),
            fret_icons::ids::ui::CLOSE,
            Some(Px(11.0)),
            Some(Arc::from("test.clear")),
            Arc::new(|_, _, _| {}),
        )
    });

    let ElementKind::Pressable(_) = &el.kind else {
        panic!("expected editor_icon_button_segment(...) to build a Pressable element");
    };
    assert_eq!(el.children.len(), 1);
    assert!(!matches!(el.children[0].kind, ElementKind::Flex(_)));
    assert!(matches!(el.children[0].kind, ElementKind::Container(_)));
}

#[test]
fn editor_input_group_row_with_single_child_returns_the_child_directly() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        let value = cx.text("Value");
        editor_input_group_row(cx, Px(0.0), vec![value])
    });

    assert!(matches!(el.kind, ElementKind::Text(_)));
    assert!(el.children.is_empty());
}

#[test]
fn editor_clear_button_segment_multiline_remains_segment_wrapped() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let density = EditorDensity::default();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        editor_clear_button_segment(
            cx,
            density,
            true,
            Arc::from("Clear text"),
            Some(Arc::from("test.clear")),
            Arc::new(|_, _, _| {}),
        )
    });

    let ElementKind::Pressable(_) = &el.kind else {
        panic!("expected editor_clear_button_segment(...) to build a Pressable element");
    };
    assert_eq!(el.children.len(), 1);
    assert!(matches!(el.children[0].kind, ElementKind::Container(_)));
}

fn unwrap_joined_frame_content(
    root: &fret_ui::element::AnyElement,
) -> &fret_ui::element::AnyElement {
    let pointer = &root.children[0];
    let frame = &pointer.children[0];
    &frame.children[0]
}

#[test]
fn joined_input_frame_without_segments_keeps_input_directly_inside_frame() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let (density, chrome) = {
        let theme = Theme::global(&app);
        let style = EditorStyle::resolve(theme);
        (style.density, style.frame_chrome_small())
    };

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        editor_joined_input_frame(
            cx,
            Default::default(),
            density,
            chrome,
            true,
            false,
            None,
            |cx| cx.text("Value"),
            |_cx| Vec::new(),
        )
    });

    assert!(matches!(el.kind, ElementKind::HoverRegion(_)));
    let content = unwrap_joined_frame_content(&el);
    assert!(
        matches!(content.kind, ElementKind::Text(_)),
        "joined input frame without segments should mount the input directly, got {:?}",
        content.kind
    );
}

#[test]
fn joined_input_frame_with_trailing_segments_keeps_row_shell() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let (density, chrome) = {
        let theme = Theme::global(&app);
        let style = EditorStyle::resolve(theme);
        (style.density, style.frame_chrome_small())
    };

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        editor_joined_input_frame_segments_with_overrides(
            cx,
            Default::default(),
            density,
            chrome,
            true,
            false,
            None,
            |_cx, _focused| super::EditorInputGroupFrameOverrides::none(),
            |_cx| Vec::new(),
            |cx| cx.text("Value"),
            |cx| vec![cx.text("X")],
        )
    });

    assert!(matches!(el.kind, ElementKind::HoverRegion(_)));
    let content = unwrap_joined_frame_content(&el);
    assert!(
        matches!(content.kind, ElementKind::Flex(_)),
        "joined input frame with trailing segments should keep the row shell, got {:?}",
        content.kind
    );
}
