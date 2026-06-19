use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::ColorRef;

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::icons::{editor_icon, editor_icon_with};
use crate::primitives::visuals::{editor_icon_button_bg, editor_icon_button_border};

use super::editor_input_group_segment;

pub(crate) fn editor_icon_button_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    enabled_for_paint: bool,
    a11y_label: Arc<str>,
    icon: fret_icons::IconId,
    icon_size: Option<Px>,
    test_id: Option<Arc<str>>,
    on_activate: OnActivate,
) -> AnyElement {
    let affordance_extent = density.affordance_extent();

    let mut el = cx.pressable(
        PressableProps {
            enabled: enabled_for_paint,
            focusable: false,
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(affordance_extent),
                    height: Length::Px(density.row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            a11y: PressableA11y {
                label: Some(a11y_label),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_activate(on_activate.clone());

            let theme = Theme::global(&*cx.app);
            let hovered = st.hovered || st.hovered_raw;
            let pressed = st.pressed;
            let bg = editor_icon_button_bg(theme, enabled_for_paint, hovered, pressed);
            let border = editor_icon_button_border(theme, enabled_for_paint, hovered, pressed);
            let border_width = if border.is_some() { Px(1.0) } else { Px(0.0) };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    background: bg,
                    border: Edges::all(border_width),
                    border_color: border,
                    corner_radii: Corners::all(Px(0.0)),
                    ..Default::default()
                },
                move |cx| vec![editor_icon(cx, density, icon, icon_size)],
            )]
        },
    );

    if let Some(test_id) = test_id.as_ref() {
        el = el.test_id(test_id.clone());
    }

    el
}

pub(crate) fn editor_clear_button_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    enabled_for_paint: bool,
    a11y_label: Arc<str>,
    test_id: Option<Arc<str>>,
    on_activate: OnActivate,
) -> AnyElement {
    editor_icon_button_segment(
        cx,
        density,
        enabled_for_paint,
        a11y_label,
        fret_icons::ids::ui::CLOSE,
        Some(Px(11.0)),
        test_id,
        on_activate,
    )
}

pub(crate) fn editor_clear_button_segment_multiline<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    chrome: ResolvedEditorFrameChrome,
    enabled_for_paint: bool,
    a11y_label: Arc<str>,
    test_id: Option<Arc<str>>,
    on_activate: OnActivate,
) -> AnyElement {
    let affordance_extent = density.affordance_extent();
    let button = editor_clear_button_segment(
        cx,
        density,
        enabled_for_paint,
        a11y_label,
        test_id,
        on_activate,
    );

    editor_input_group_segment(
        cx,
        LayoutStyle {
            size: SizeStyle {
                width: Length::Px(affordance_extent),
                height: Length::Fill,
                ..Default::default()
            },
            ..Default::default()
        },
        Edges {
            top: chrome.padding.top,
            right: Px(0.0),
            bottom: Px(0.0),
            left: Px(0.0),
        },
        button,
    )
}

pub(crate) fn editor_icon_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    icon: fret_icons::IconId,
    icon_size: Option<Px>,
    color: Option<ColorRef>,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(density.hit_thickness),
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        move |cx| {
            vec![editor_icon_with(
                cx,
                density,
                icon,
                icon_size,
                color.clone(),
            )]
        },
    )
}
