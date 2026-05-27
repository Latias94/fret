use std::sync::Arc;

use fret_core::{Corners, Px};
use fret_ui::element::{AnyElement, ContainerProps, Length, MainAlign};
use fret_ui::{ElementContext, UiHost};

pub(super) fn slider_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    current: f32,
    progress: f32,
    palette: super::super::control_chrome::ImUiControlPalette,
) -> Vec<AnyElement> {
    let mut track = ContainerProps::default();
    track.layout.size.width = Length::Fill;
    track.layout.size.height = Length::Px(super::super::control_chrome::SLIDER_TRACK_HEIGHT);
    track.background = Some(palette.subtle_background);
    track.corner_radii =
        Corners::all(Px(super::super::control_chrome::SLIDER_TRACK_HEIGHT.0 / 2.0));

    let mut fill = ContainerProps::default();
    fill.layout.size.width = Length::Fraction(progress);
    fill.layout.size.height = Length::Fill;
    fill.background = Some(palette.accent_background);
    fill.corner_radii = track.corner_radii;

    let value_badge = super::super::control_chrome::pill(
        cx,
        Arc::from(format!("{current:.2}")),
        palette.accent_background,
        palette.accent_foreground,
    );

    vec![cx.flex(
        super::super::control_chrome::fill_stack_props(),
        move |cx| {
            let mut out = Vec::new();
            out.push(cx.flex(
                super::super::control_chrome::fill_row_props(MainAlign::SpaceBetween),
                move |cx| {
                    let mut row = Vec::new();
                    if !label.is_empty() {
                        row.push(super::super::control_chrome::caption_text(
                            cx,
                            label.clone(),
                            palette,
                        ));
                    }
                    row.push(value_badge);
                    row
                },
            ));
            out.push(cx.container(track, move |cx| vec![cx.container(fill, |_cx| vec![])]));
            out
        },
    )]
}

pub(super) fn slider_progress(current: f32, min: f32, max: f32) -> f32 {
    let range = max - min;
    if !range.is_finite() || range.abs() <= f32::EPSILON {
        return 1.0;
    }

    ((current - min) / range).clamp(0.0, 1.0)
}
