mod fill;

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::{editor_border, editor_muted_foreground};
use crate::primitives::input_group::derived_test_id;
use crate::primitives::readout::editor_preview_caption_text_props;

use super::super::model::format_hex;
use super::super::{ColorEditAlphaPreview, ColorEditPopupSidePreview};

#[cfg(test)]
pub(in crate::controls::color_edit) use fill::{checkerboard_cell_color, opaque_preview_color};
pub(in crate::controls::color_edit::popup) use fill::{checkerboard_grid, fill_preview_layout};
pub(in crate::controls::color_edit) use fill::{
    color_preview_stack, preview_color_for_alpha_visibility,
};

pub(in crate::controls::color_edit) const SIDE_PREVIEW_SWATCH_WIDTH: Px = Px(72.0);
pub(in crate::controls::color_edit) const SIDE_PREVIEW_SWATCH_HEIGHT: Px = Px(48.0);

pub(super) fn color_side_preview<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    original: Option<Color>,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    mode: ColorEditPopupSidePreview,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let original_cell = mode
        .shows_original()
        .then(|| original)
        .flatten()
        .map(|original| {
            original_reference_preview_cell(
                cx,
                original,
                model,
                draft,
                error,
                show_alpha,
                enabled,
                alpha_preview,
                derived_test_id(test_id.as_ref(), "original"),
            )
        });
    let current_cell = current_preview_cell(
        cx,
        current,
        show_alpha,
        alpha_preview,
        derived_test_id(test_id.as_ref(), "current"),
    );

    let mut preview = cx.flex(
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
            gap: SpacingLength::Px(Px(6.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |_cx| {
            let mut out = vec![current_cell];
            if let Some(original_cell) = original_cell {
                out.push(original_cell);
            }
            out
        },
    );

    if let Some(test_id) = test_id {
        preview = preview.test_id(test_id);
    }
    preview
}

fn current_preview_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    show_alpha: bool,
    alpha_preview: ColorEditAlphaPreview,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    preview_cell_container(
        cx,
        "Current",
        preview_color_for_alpha_visibility(color, show_alpha),
        show_alpha,
        alpha_preview,
        test_id,
    )
}

fn original_reference_preview_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    original: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let restore: OnActivate =
        Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
            let current = host
                .models_mut()
                .get_copied(&model)
                .unwrap_or(Color::TRANSPARENT);
            let next = restore_reference_color(original, current, show_alpha);
            let formatted = format_hex(next, show_alpha);

            let _ = host.models_mut().update(&model, |color| *color = next);
            let _ = host
                .models_mut()
                .update(&draft, |text| *text = formatted.as_ref().to_string());
            let _ = host.models_mut().update(&error, |value| *value = None);
            host.request_redraw(action_cx.window);
        });

    let color = preview_color_for_alpha_visibility(original, show_alpha);
    let mut cell = cx.pressable(
        PressableProps {
            layout: preview_cell_layout(),
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Button),
                label: Some(Arc::from("Original color")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, _st| {
            cx.pressable_add_on_activate(restore.clone());
            vec![preview_cell_content(
                cx,
                "Original",
                color,
                show_alpha,
                alpha_preview,
            )]
        },
    );

    if let Some(test_id) = test_id {
        cell = cell.test_id(test_id);
    }
    cell.a11y_value(format_hex(color, show_alpha))
}

fn preview_cell_container<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    color: Color,
    show_alpha: bool,
    alpha_preview: ColorEditAlphaPreview,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let mut cell = cx.container(
        ContainerProps {
            layout: preview_cell_layout(),
            ..Default::default()
        },
        move |cx| {
            vec![preview_cell_content(
                cx,
                label,
                color,
                show_alpha,
                alpha_preview,
            )]
        },
    );

    if let Some(test_id) = test_id {
        cell = cell.test_id(test_id);
    }
    cell
}

fn preview_cell_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    color: Color,
    show_alpha: bool,
    alpha_preview: ColorEditAlphaPreview,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let border = editor_border(theme);
    let text_color = editor_muted_foreground(theme);

    cx.flex(
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
            gap: SpacingLength::Px(Px(4.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            vec![
                cx.text_props(editor_preview_caption_text_props(
                    Arc::from(label),
                    text_color,
                )),
                cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(SIDE_PREVIEW_SWATCH_HEIGHT),
                                min_height: Some(Length::Px(SIDE_PREVIEW_SWATCH_HEIGHT)),
                                ..Default::default()
                            },
                            overflow: Overflow::Clip,
                            ..Default::default()
                        },
                        border: Edges::all(Px(1.0)),
                        border_color: Some(border),
                        corner_radii: Corners::all(Px(5.0)),
                        padding: Edges::all(Px(1.0)).into(),
                        ..Default::default()
                    },
                    move |cx| vec![color_preview_stack(cx, color, Px(5.0), alpha_preview)],
                )
                .a11y_value(format_hex(color, show_alpha)),
            ]
        },
    )
}

fn preview_cell_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Px(SIDE_PREVIEW_SWATCH_WIDTH),
            height: Length::Auto,
            min_width: Some(Length::Px(SIDE_PREVIEW_SWATCH_WIDTH)),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub(in crate::controls::color_edit) fn restore_reference_color(
    reference: Color,
    current: Color,
    show_alpha: bool,
) -> Color {
    if show_alpha {
        reference
    } else {
        let mut next = reference;
        next.a = current.a;
        next
    }
}
