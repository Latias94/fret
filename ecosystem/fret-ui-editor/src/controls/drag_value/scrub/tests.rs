use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Color, Point, Px, Rect, Size};
use fret_ui::element::{ElementKind, Length, SpacingEdges, SpacingLength};
use fret_ui::elements;

use super::{DragValueScrubFrameArgs, drag_value_scrub_frame};
use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

fn test_chrome() -> ResolvedEditorFrameChrome {
    ResolvedEditorFrameChrome {
        padding: fret_core::Edges::all(Px(4.0)),
        radius: Px(4.0),
        border_width: Px(1.0),
        bg: Color::from_srgb_hex_rgb(0x11_11_11),
        border: Color::from_srgb_hex_rgb(0x22_22_22),
        border_focus: Color::from_srgb_hex_rgb(0x33_33_33),
        fg: Color::from_srgb_hex_rgb(0xee_ee_ee),
        text_px: Px(12.0),
    }
}

#[test]
fn scrub_frame_without_affixes_mounts_value_text_directly() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let density = EditorDensity::default();
    let chrome = test_chrome();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        drag_value_scrub_frame(
            cx,
            DragValueScrubFrameArgs {
                density,
                scrub_chrome: chrome,
                hovered: false,
                pressed: false,
                focused: false,
                value_text: Arc::from("42%"),
                prefix: None,
                suffix: None,
                scrub_test_id: Some(Arc::from("drag.value")),
                prefix_test_id: None,
                suffix_test_id: None,
                value_test_id: Some(Arc::from("drag.value.value")),
            },
        )
    });

    let ElementKind::Container(frame) = &el.kind else {
        panic!("expected scrub frame to build a Container root");
    };
    assert_eq!(
        frame.padding,
        chrome.padding.into(),
        "plain scrub frames should own content padding so the value text can mount directly"
    );
    assert_eq!(frame.layout.size.width, Length::Fill);
    assert_eq!(frame.layout.size.height, Length::Fill);
    assert_eq!(
        frame.layout.size.min_height,
        Some(Length::Px(density.row_height))
    );

    let ElementKind::Text(_) = &el.children[0].kind else {
        panic!("expected scrub frame child to mount the value text directly");
    };
}

#[test]
fn scrub_frame_with_affixes_keeps_segment_padding_layers() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let density = EditorDensity::default();
    let chrome = test_chrome();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        drag_value_scrub_frame(
            cx,
            DragValueScrubFrameArgs {
                density,
                scrub_chrome: chrome,
                hovered: false,
                pressed: false,
                focused: false,
                value_text: Arc::from("42"),
                prefix: Some(Arc::from("$")),
                suffix: Some(Arc::from("ms")),
                scrub_test_id: Some(Arc::from("drag.value")),
                prefix_test_id: Some(Arc::from("drag.value.prefix")),
                suffix_test_id: Some(Arc::from("drag.value.suffix")),
                value_test_id: Some(Arc::from("drag.value.value")),
            },
        )
    });

    let ElementKind::Container(frame) = &el.kind else {
        panic!("expected scrub frame to build a Container root");
    };
    assert_eq!(
        frame.padding,
        SpacingEdges::all(SpacingLength::Px(Px(0.0))),
        "segmented scrub frames keep padding on each segment so dividers reach the frame edge"
    );

    assert_eq!(
        el.children.len(),
        1,
        "affix scrub frames should keep the joined row shell"
    );
    let row = &el.children[0];
    assert_eq!(
        row.children.len(),
        5,
        "prefix, divider, value, divider, and suffix should remain separate segments"
    );

    let ElementKind::Container(value_segment) = &row.children[2].kind else {
        panic!("expected the value segment to remain a padded Container");
    };
    assert_eq!(value_segment.padding, chrome.padding.into());

    let ElementKind::Text(_) = &row.children[2].children[0].kind else {
        panic!("expected the value segment to mount the text directly");
    };
}
