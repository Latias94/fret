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

            let active_bg = mix_color(subtle_bg, accent, 0.42);
            let hover_bg = mix_color(subtle_bg, accent, 0.18);
            let pressed_bg = mix_color(subtle_bg, accent, 0.32);
            let background = if selected {
                active_bg
            } else if state.pressed {
                pressed_bg
            } else if state.hovered || state.hovered_raw {
                hover_bg
            } else {
                subtle_bg
            };
            let text_color = if enabled {
                fg
            } else {
                mix_color(muted_fg, subtle_bg, 0.35)
            };
            let border_color = if selected { accent } else { border };
            let status_text = preset.picker_status_label();
            let status_color = if selected { accent } else { muted_fg };

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
                    background: Some(background),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
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
                                    text_color,
                                    row_height,
                                    text_px,
                                )),
                                cx.text_props(editor_theme_preset_picker_row_status_text_props(
                                    Arc::from(status_text),
                                    status_color,
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

fn mix_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}
