use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::{editor_border, editor_muted_foreground};
use crate::primitives::readout::editor_preview_caption_text_props;

use super::super::super::super::{ColorEditAlphaPreview, model::format_hex};
use super::super::fill::{color_preview_stack, preview_color_for_alpha_visibility};

pub(in crate::controls::color_edit) const SIDE_PREVIEW_SWATCH_WIDTH: Px = Px(72.0);
pub(in crate::controls::color_edit) const SIDE_PREVIEW_SWATCH_HEIGHT: Px = Px(48.0);

pub(super) fn current_preview_cell<H: UiHost>(
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

pub(super) fn preview_cell_content<H: UiHost>(
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

pub(super) fn preview_cell_layout() -> LayoutStyle {
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
