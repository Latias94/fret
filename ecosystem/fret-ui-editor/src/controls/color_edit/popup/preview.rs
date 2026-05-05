use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, GridProps, GridTrackSizing, InsetStyle, LayoutStyle, Length,
    Overflow, PositionStyle, SizeStyle, SpacingLength, StackProps,
};
use fret_ui::{ElementContext, UiHost};

use super::super::{CHECKERBOARD_DARK_RGB, CHECKERBOARD_LIGHT_RGB};

pub(in crate::controls::color_edit) fn color_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    radius: Px,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: fill_preview_layout(),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        move |cx| {
            vec![cx.stack_props(
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
            )]
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
