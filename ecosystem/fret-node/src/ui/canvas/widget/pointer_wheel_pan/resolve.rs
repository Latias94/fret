use crate::ui::canvas::widget::*;

pub(super) struct ResolvedScrollPan {
    pub dx: f32,
    pub dy: f32,
    pub speed: f32,
}

pub(super) fn resolve_scroll_pan(
    snapshot: &ViewSnapshot,
    platform: fret_runtime::Platform,
    delta: Point,
    modifiers: fret_core::Modifiers,
) -> ResolvedScrollPan {
    let mode = snapshot.interaction.pan_on_scroll_mode;
    let speed = snapshot.interaction.pan_on_scroll_speed.max(0.0);
    let dy_for_shift = delta.y.0;

    let mut dx = delta.x.0;
    let mut dy = delta.y.0;
    match mode {
        crate::io::NodeGraphPanOnScrollMode::Free => {}
        crate::io::NodeGraphPanOnScrollMode::Horizontal => {
            dy = 0.0;
        }
        crate::io::NodeGraphPanOnScrollMode::Vertical => {
            dx = 0.0;
        }
    }

    if platform != fret_runtime::Platform::Macos
        && modifiers.shift
        && !matches!(mode, crate::io::NodeGraphPanOnScrollMode::Vertical)
    {
        dx = dy_for_shift;
        dy = 0.0;
    }

    ResolvedScrollPan { dx, dy, speed }
}
