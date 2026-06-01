use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    SemanticsProps, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

mod row;

use crate::primitives::readout::editor_theme_preset_picker_header_text_props;
use crate::theme::EditorThemePresetV1;
use row::preset_row;

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
