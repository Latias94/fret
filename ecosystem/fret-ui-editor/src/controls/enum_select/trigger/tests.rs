use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Color, Edges, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::element::{ElementKind, LayoutStyle, Length, SizeStyle};
use fret_ui::elements::with_element_cx;
use fret_ui_kit::primitives::combobox as kit_combobox;

use crate::controls::enum_select::trigger::{EnumSelectTriggerArgs, enum_select_trigger};
use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;

fn test_bounds() -> Rect {
    Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(56.0)),
    )
}

fn trigger_args(
    open: Model<bool>,
    open_change_reason: Model<Option<kit_combobox::ComboboxOpenChangeReason>>,
) -> EnumSelectTriggerArgs {
    let layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            ..Default::default()
        },
        ..Default::default()
    };
    let density = EditorDensity::default();
    let frame_chrome = ResolvedEditorFrameChrome {
        padding: Edges::all(Px(0.0)),
        radius: Px(6.0),
        border_width: Px(1.0),
        bg: Color::from_srgb_hex_rgb(0x22_22_22),
        border: Color::from_srgb_hex_rgb(0x55_55_55),
        border_focus: Color::from_srgb_hex_rgb(0x88_88_88),
        fg: Color::from_srgb_hex_rgb(0xEE_EE_EE),
        text_px: Px(12.0),
    };

    EnumSelectTriggerArgs {
        layout,
        enabled: true,
        focusable: true,
        a11y_label: Some(Arc::from("Mode")),
        density,
        frame_chrome,
        ring: Color::from_srgb_hex_rgb(0xAA_AA_AA),
        is_open: false,
        trigger_text: Arc::from("Lit"),
        open,
        open_change_reason,
    }
}

#[test]
fn enum_select_trigger_caret_is_mounted_without_an_inner_flex_shell() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let open = app.models_mut().insert(false);
    let open_change_reason = app
        .models_mut()
        .insert(None::<kit_combobox::ComboboxOpenChangeReason>);

    let trigger = with_element_cx(
        &mut app,
        window,
        test_bounds(),
        "enum-select-trigger",
        |cx| enum_select_trigger(cx, trigger_args(open.clone(), open_change_reason.clone())),
    );

    let ElementKind::Pressable(_) = &trigger.kind else {
        panic!("enum select trigger should remain a pressable root");
    };
    assert_eq!(trigger.children.len(), 1);

    let frame = &trigger.children[0];
    let ElementKind::Container(_) = &frame.kind else {
        panic!("enum select trigger should keep the frame container");
    };
    let row = &frame.children[0];
    let ElementKind::Flex(_) = &row.kind else {
        panic!("enum select trigger should keep the row shell");
    };
    let caret = &row.children[2];
    let ElementKind::Flex(_) = &caret.kind else {
        panic!("caret should mount as a direct flex center shell");
    };
    assert_eq!(caret.children.len(), 1);
    assert!(matches!(
        caret.children[0].kind,
        ElementKind::SvgIcon(_) | ElementKind::SvgImage(_)
    ));
}
