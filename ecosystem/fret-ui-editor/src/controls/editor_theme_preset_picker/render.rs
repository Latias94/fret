use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, RingPlacement, RingStyle, SemanticsProps, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::readout::{
    editor_theme_preset_picker_header_text_props, editor_theme_preset_picker_row_label_text_props,
    editor_theme_preset_picker_row_status_text_props,
};
use crate::theme::EditorThemePresetV1;

pub(super) struct EditorThemePresetPickerRenderInput {
    pub(super) selected: EditorThemePresetV1,
    pub(super) label: Arc<str>,
    pub(super) item_prefix: Option<Arc<str>>,
    pub(super) options: super::EditorThemePresetPickerOptions,
    pub(super) model: Model<EditorThemePresetV1>,
    pub(super) total: usize,
    pub(super) row_height: Px,
    pub(super) padding_x: Px,
    pub(super) border: Color,
    pub(super) ring: Color,
    pub(super) fg: Color,
    pub(super) muted_fg: Color,
    pub(super) subtle_bg: Color,
    pub(super) accent: Color,
    pub(super) text_px: Px,
}

pub(super) fn build_editor_theme_preset_picker_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: EditorThemePresetPickerRenderInput,
) -> AnyElement {
    let EditorThemePresetPickerRenderInput {
        selected,
        label,
        item_prefix,
        options,
        model,
        total,
        row_height,
        padding_x,
        border,
        ring,
        fg,
        muted_fg,
        subtle_bg,
        accent,
        text_px,
    } = input;

    cx.semantics(
        SemanticsProps {
            layout: options.layout,
            role: SemanticsRole::ListBox,
            label: Some(label.clone()),
            test_id: options.test_id.clone(),
            ..Default::default()
        },
        move |cx| {
            let mut rows = Vec::with_capacity(total + 1);
            rows.push(header_text(cx, label.clone(), muted_fg, text_px));
            rows.extend(
                super::super::super::theme::EDITOR_THEME_PRESETS_V1
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(index, preset)| {
                        preset_row(
                            cx,
                            model.clone(),
                            preset,
                            selected == preset,
                            index,
                            total,
                            item_prefix.clone(),
                            options.enabled,
                            options.focusable,
                            row_height,
                            padding_x,
                            border,
                            ring,
                            fg,
                            muted_fg,
                            subtle_bg,
                            accent,
                            text_px,
                        )
                    }),
            );

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::all(Px(3.0)).into(),
                    background: Some(subtle_bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border),
                    corner_radii: Corners::all(Px(4.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: Axis::Vertical,
                            gap: SpacingLength::Px(Px(3.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            ..Default::default()
                        },
                        move |_cx| rows,
                    )]
                },
            )]
        },
    )
}

fn header_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    color: Color,
    text_px: Px,
) -> AnyElement {
    cx.text_props(editor_theme_preset_picker_header_text_props(
        label, color, text_px,
    ))
}

#[allow(clippy::too_many_arguments)]
fn preset_row<H: UiHost>(
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
    let model_for_activate = model.clone();

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
            let on_activate: OnActivate =
                Arc::new(move |host, action_cx, _reason: ActivateReason| {
                    let _ = host
                        .models_mut()
                        .update(&model_for_activate, |value| *value = preset);
                    host.request_redraw(action_cx.window);
                });
            cx.pressable_add_on_activate(on_activate);

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
