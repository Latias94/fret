use fret_core::{Color, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, GridProps, GridTrackSizing, SpacingLength};
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::{CHECKERBOARD_DARK_RGB, CHECKERBOARD_LIGHT_RGB};
use super::fill_preview_layout;

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

pub(in crate::controls::color_edit) fn checkerboard_cell_color(row: usize, col: usize) -> Color {
    let rgb = if (row + col).is_multiple_of(2) {
        CHECKERBOARD_LIGHT_RGB
    } else {
        CHECKERBOARD_DARK_RGB
    };
    Color::from_srgb_hex_rgb(rgb)
}
