use fret_core::{Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, GridProps, GridTrackSizing, SpacingLength};
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::super::ALPHA_BAR_STEPS;
use super::super::super::super::super::model::{color_from_rgb_preserving_alpha, unit_from_step};
use super::super::super::super::preview::fill_preview_layout;

pub(super) fn vertical_alpha_gradient_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    rgb: u32,
) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: 1,
            rows: Some(ALPHA_BAR_STEPS as u16),
            template_columns: Some(vec![GridTrackSizing::Flex(1.0)]),
            template_rows: Some(
                (0..ALPHA_BAR_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..ALPHA_BAR_STEPS)
                .map(|idx| {
                    let alpha = 1.0 - unit_from_step(idx, ALPHA_BAR_STEPS);
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(color_from_rgb_preserving_alpha(rgb, alpha)),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

pub(super) fn alpha_gradient_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    rgb: u32,
) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: ALPHA_BAR_STEPS as u16,
            rows: Some(1),
            template_columns: Some(
                (0..ALPHA_BAR_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            template_rows: Some(vec![GridTrackSizing::Flex(1.0)]),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..ALPHA_BAR_STEPS)
                .map(|idx| {
                    let alpha = (idx + 1) as f32 / ALPHA_BAR_STEPS as f32;
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(color_from_rgb_preserving_alpha(rgb, alpha)),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}
