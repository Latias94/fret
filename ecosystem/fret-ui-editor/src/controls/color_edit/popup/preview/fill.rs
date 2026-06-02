use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, GridProps, GridTrackSizing, InsetStyle, LayoutStyle, Length,
    Overflow, PositionStyle, SizeStyle, SpacingLength, StackProps,
};
use fret_ui::{ElementContext, UiHost};

use super::super::super::{CHECKERBOARD_DARK_RGB, CHECKERBOARD_LIGHT_RGB, ColorEditAlphaPreview};

pub(in crate::controls::color_edit) fn color_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    radius: Px,
    alpha_preview: ColorEditAlphaPreview,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: fill_preview_layout(),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        move |cx| match alpha_preview {
            ColorEditAlphaPreview::Checkerboard => {
                vec![checkerboard_preview_fill(cx, color, radius)]
            }
            ColorEditAlphaPreview::Opaque => {
                vec![solid_preview_fill(cx, opaque_preview_color(color), radius)]
            }
            ColorEditAlphaPreview::NoBackground => vec![solid_preview_fill(cx, color, radius)],
            ColorEditAlphaPreview::Half => vec![half_alpha_preview_fill(cx, color, radius)],
        },
    )
}

fn checkerboard_preview_fill<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    radius: Px,
) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            let checkerboard = checkerboard_grid(cx);
            let overlay = cx.container(
                ContainerProps {
                    layout: fill_absolute_preview_layout(),
                    background: Some(color),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                |_cx| vec![],
            );
            vec![checkerboard, overlay]
        },
    )
}

fn solid_preview_fill<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    radius: Px,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: fill_preview_layout(),
            background: Some(color),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        |_cx| vec![],
    )
}

fn half_alpha_preview_fill<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    radius: Px,
) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: 2,
            rows: Some(1),
            template_columns: Some(vec![GridTrackSizing::Flex(1.0), GridTrackSizing::Flex(1.0)]),
            template_rows: Some(vec![GridTrackSizing::Flex(1.0)]),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        move |cx| {
            vec![
                solid_preview_fill(cx, opaque_preview_color(color), radius),
                checkerboard_preview_fill(cx, color, radius),
            ]
        },
    )
}

pub(in crate::controls::color_edit::popup) fn checkerboard_grid<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: 2,
            rows: Some(2),
            template_columns: Some(vec![GridTrackSizing::Flex(1.0), GridTrackSizing::Flex(1.0)]),
            template_rows: Some(vec![GridTrackSizing::Flex(1.0), GridTrackSizing::Flex(1.0)]),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..4)
                .map(|idx| {
                    let row = idx / 2;
                    let col = idx % 2;
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(checkerboard_cell_color(row, col)),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

pub(in crate::controls::color_edit::popup) fn fill_preview_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            ..Default::default()
        },
        overflow: Overflow::Clip,
        ..Default::default()
    }
}

fn fill_absolute_preview_layout() -> LayoutStyle {
    let mut layout = fill_preview_layout();
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top: Some(Px(0.0)).into(),
        right: Some(Px(0.0)).into(),
        bottom: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
    };
    layout
}

pub(in crate::controls::color_edit) fn checkerboard_cell_color(row: usize, col: usize) -> Color {
    let rgb = if (row + col).is_multiple_of(2) {
        CHECKERBOARD_LIGHT_RGB
    } else {
        CHECKERBOARD_DARK_RGB
    };
    Color::from_srgb_hex_rgb(rgb)
}

pub(in crate::controls::color_edit) fn opaque_preview_color(mut color: Color) -> Color {
    color.a = 1.0;
    color
}

pub(in crate::controls::color_edit) fn preview_color_for_alpha_visibility(
    color: Color,
    show_alpha: bool,
) -> Color {
    if show_alpha {
        color
    } else {
        opaque_preview_color(color)
    }
}
