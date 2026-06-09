use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, RingPlacement, RingStyle, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::readout::{
    editor_theme_preset_picker_row_label_text_props,
    editor_theme_preset_picker_row_status_text_props,
};
use crate::theme::EditorThemePresetV1;

mod behavior;
mod visual;

#[allow(clippy::too_many_arguments)]
pub(super) fn preset_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<EditorThemePresetV1>,
    preset: EditorThemePresetV1,
    selected: bool,
    index: usize,
    total: usize,
    item_prefix: Option<Arc<str>>,
    enabled: bool,
    focusable: bool,
    row_height: Px,
    padding_x: Px,
    border: Color,
    ring: Color,
    fg: Color,
    muted_fg: Color,
    subtle_bg: Color,
    accent: Color,
    text_px: Px,
) -> AnyElement {
    let item_test_id = item_prefix
        .as_ref()
        .map(|prefix| Arc::<str>::from(format!("{prefix}.{}", preset.key())));
    let label = Arc::<str>::from(preset.label());
    let on_activate = behavior::theme_preset_row_activate(model, preset);

    let mut row = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable,
            focus_ring: Some(RingStyle {
                placement: RingPlacement::Outset,
                width: Px(1.0),
                offset: Px(1.0),
                color: ring,
                offset_color: None,
                corner_radii: Corners::all(Px(3.0)),
            }),
            a11y: PressableA11y {
                role: Some(SemanticsRole::ListBoxOption),
                label: Some(label.clone()),
                test_id: item_test_id.clone(),
                selected,
                pos_in_set: Some((index as u32) + 1),
                set_size: Some(total as u32),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, state| {
            cx.pressable_add_on_activate(on_activate.clone());

            let visual = visual::theme_preset_row_visual(visual::ThemePresetRowVisualInput {
                selected,
                enabled,
                hovered: state.hovered,
                hovered_raw: state.hovered_raw,
                pressed: state.pressed,
                fg,
                muted_fg,
                subtle_bg,
                accent,
                border,
            });
            let status_text = preset.picker_status_label();

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
                    padding: Edges::symmetric(padding_x, Px(0.0)).into(),
                    background: Some(visual.background),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(visual.border_color),
                    corner_radii: Corners::all(Px(3.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: Axis::Horizontal,
                            gap: SpacingLength::Px(Px(8.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                cx.text_props(editor_theme_preset_picker_row_label_text_props(
                                    label.clone(),
                                    visual.text_color,
                                    row_height,
                                    text_px,
                                )),
                                cx.text_props(editor_theme_preset_picker_row_status_text_props(
                                    Arc::from(status_text),
                                    visual.status_color,
                                    row_height,
                                    text_px,
                                )),
                            ]
                        },
                    )]
                },
            )]
        },
    );

    if let Some(test_id) = item_test_id {
        row = row.test_id(test_id);
    }

    row
}
