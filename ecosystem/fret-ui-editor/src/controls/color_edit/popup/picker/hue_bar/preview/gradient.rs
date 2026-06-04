use fret_core::{Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, GridProps, GridTrackSizing, SpacingLength};
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::super::HUE_BAR_STEPS;
use super::super::super::super::super::model::{HsvColor, hsv_to_color_preserving_alpha};
use super::super::super::super::preview::fill_preview_layout;

pub(super) fn vertical_hue_gradient_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: 1,
            rows: Some(HUE_BAR_STEPS as u16),
            template_columns: Some(vec![GridTrackSizing::Flex(1.0)]),
            template_rows: Some(
                (0..HUE_BAR_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..HUE_BAR_STEPS)
                .map(|idx| {
                    let hue = idx as f32 / HUE_BAR_STEPS as f32;
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(hsv_to_color_preserving_alpha(
                                HsvColor {
                                    hue,
                                    saturation: 1.0,
                                    value: 1.0,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}
