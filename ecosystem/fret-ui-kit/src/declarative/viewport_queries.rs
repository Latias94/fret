use fret_core::Px;
use fret_ui::{ElementContext, Invalidation, UiHost};

/// Tailwind-compatible viewport width breakpoints.
///
/// These are provided as a convenience for shadcn-aligned recipes. Consumers are free to define
/// their own breakpoint tables.
pub mod tailwind {
    use fret_core::Px;

    pub const SM: Px = Px(640.0);
    pub const MD: Px = Px(768.0);
    pub const LG: Px = Px(1024.0);
    pub const XL: Px = Px(1280.0);
    pub const XXL: Px = Px(1536.0);
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportQueryHysteresis {
    pub up: Px,
    pub down: Px,
}

impl Default for ViewportQueryHysteresis {
    fn default() -> Self {
        // Keep the default small: enough to avoid single-pixel oscillation without delaying
        // responsive behavior too much in desktop resize drags.
        Self {
            up: Px(8.0),
            down: Px(8.0),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ViewportBreakpointsState {
    /// 0 = base value, i>0 selects `breakpoints[i-1]`.
    active_index: usize,
    initialized: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct ViewportWidthAtLeastState {
    active: bool,
    initialized: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct ViewportHeightAtLeastState {
    active: bool,
    initialized: bool,
}

fn viewport_breakpoints_init_active_index<T: Copy>(width: Px, breakpoints: &[(Px, T)]) -> usize {
    let mut active_index = 0;
    for (i, (min_width, _)) in breakpoints.iter().enumerate() {
        if width.0 >= min_width.0 {
            active_index = i + 1;
        }
    }
    active_index
}

fn viewport_breakpoints_apply_hysteresis<T: Copy>(
    width: Px,
    breakpoints: &[(Px, T)],
    hysteresis: ViewportQueryHysteresis,
    mut active_index: usize,
) -> usize {
    loop {
        if active_index >= breakpoints.len() {
            break;
        }
        let next_min_width = breakpoints[active_index].0;
        if width.0 >= next_min_width.0 + hysteresis.up.0 {
            active_index = active_index.saturating_add(1);
            continue;
        }
        break;
    }

    loop {
        if active_index == 0 {
            break;
        }
        let cur_min_width = breakpoints[active_index - 1].0;
        if width.0 < cur_min_width.0 - hysteresis.down.0 {
            active_index = active_index.saturating_sub(1);
            continue;
        }
        break;
    }

    active_index
}

fn viewport_dimension_at_least_init(dimension: Px, threshold: Px) -> bool {
    dimension.0 >= threshold.0
}

fn viewport_dimension_at_least_apply_hysteresis(
    dimension: Px,
    threshold: Px,
    hysteresis: ViewportQueryHysteresis,
    active: bool,
) -> bool {
    if !active && dimension.0 >= threshold.0 + hysteresis.up.0 {
        return true;
    }
    if active && dimension.0 < threshold.0 - hysteresis.down.0 {
        return false;
    }
    active
}

/// Viewport-driven responsive breakpoints, based on the committed per-window environment snapshot.
///
/// This is intended for **device/viewport** decisions (ADR 1171). For panel-width responsiveness
/// inside docking/panels, prefer container queries (ADR 1170).
#[track_caller]
pub fn viewport_breakpoints<H, T: Copy>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    base: T,
    breakpoints: &[(Px, T)],
    hysteresis: ViewportQueryHysteresis,
) -> T
where
    H: UiHost,
{
    // Ensure each callsite gets its own stable element identity for hysteresis state.
    cx.scope(|cx| {
        let width = cx.environment_viewport_width(invalidation);
        cx.with_state(ViewportBreakpointsState::default, |st| {
            if !st.initialized {
                st.active_index = viewport_breakpoints_init_active_index(width, breakpoints);
                st.initialized = true;
            }

            st.active_index = viewport_breakpoints_apply_hysteresis(
                width,
                breakpoints,
                hysteresis,
                st.active_index,
            );

            breakpoints
                .get(st.active_index.saturating_sub(1))
                .map(|(_, v)| *v)
                .unwrap_or(base)
        })
    })
}

/// Viewport-driven boolean query with hysteresis.
///
/// Returns `true` when viewport width is (stably) at least `threshold` (Tailwind-style min-width).
#[track_caller]
pub fn viewport_width_at_least<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    threshold: Px,
    hysteresis: ViewportQueryHysteresis,
) -> bool {
    cx.scope(|cx| {
        let width = cx.environment_viewport_width(invalidation);
        cx.with_state(ViewportWidthAtLeastState::default, |st| {
            if !st.initialized {
                st.active = viewport_dimension_at_least_init(width, threshold);
                st.initialized = true;
            }

            st.active = viewport_dimension_at_least_apply_hysteresis(
                width, threshold, hysteresis, st.active,
            );
            st.active
        })
    })
}

/// Viewport-driven boolean query with hysteresis.
///
/// Returns `true` when viewport height is (stably) at least `threshold` (Tailwind-style min-height).
#[track_caller]
pub fn viewport_height_at_least<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    threshold: Px,
    hysteresis: ViewportQueryHysteresis,
) -> bool {
    cx.scope(|cx| {
        let height = cx.environment_viewport_height(invalidation);
        cx.with_state(ViewportHeightAtLeastState::default, |st| {
            if !st.initialized {
                st.active = viewport_dimension_at_least_init(height, threshold);
                st.initialized = true;
            }

            st.active = viewport_dimension_at_least_apply_hysteresis(
                height, threshold, hysteresis, st.active,
            );
            st.active
        })
    })
}

/// Viewport-driven responsive breakpoints (height), based on the committed per-window environment snapshot.
///
/// This is useful for viewport-driven recipes that depend on height constraints (e.g. short viewports).
#[track_caller]
pub fn viewport_height_breakpoints<H, T: Copy>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    base: T,
    breakpoints: &[(Px, T)],
    hysteresis: ViewportQueryHysteresis,
) -> T
where
    H: UiHost,
{
    cx.scope(|cx| {
        let height = cx.environment_viewport_height(invalidation);
        cx.with_state(ViewportBreakpointsState::default, |st| {
            if !st.initialized {
                st.active_index = viewport_breakpoints_init_active_index(height, breakpoints);
                st.initialized = true;
            }

            st.active_index = viewport_breakpoints_apply_hysteresis(
                height,
                breakpoints,
                hysteresis,
                st.active_index,
            );

            breakpoints
                .get(st.active_index.saturating_sub(1))
                .map(|(_, v)| *v)
                .unwrap_or(base)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_breakpoints_hysteresis_transitions() {
        let breakpoints = &[(Px(640.0), 1u8), (Px(768.0), 2u8), (Px(1024.0), 3u8)];
        let hysteresis = ViewportQueryHysteresis::default();

        let mut active_index = viewport_breakpoints_init_active_index(Px(700.0), breakpoints);
        assert_eq!(active_index, 1);

        active_index = viewport_breakpoints_apply_hysteresis(
            Px(768.0 + hysteresis.up.0 - 1.0),
            breakpoints,
            hysteresis,
            active_index,
        );
        assert_eq!(
            active_index, 1,
            "should not transition up until past hysteresis"
        );

        active_index = viewport_breakpoints_apply_hysteresis(
            Px(768.0 + hysteresis.up.0 + 1.0),
            breakpoints,
            hysteresis,
            active_index,
        );
        assert_eq!(active_index, 2);

        active_index = viewport_breakpoints_apply_hysteresis(
            Px(768.0 - hysteresis.down.0 + 1.0),
            breakpoints,
            hysteresis,
            active_index,
        );
        assert_eq!(
            active_index, 2,
            "should not transition down until past hysteresis"
        );

        active_index = viewport_breakpoints_apply_hysteresis(
            Px(768.0 - hysteresis.down.0 - 1.0),
            breakpoints,
            hysteresis,
            active_index,
        );
        assert_eq!(active_index, 1);
    }

    #[test]
    fn viewport_width_at_least_hysteresis_transitions() {
        let threshold = Px(768.0);
        let hysteresis = ViewportQueryHysteresis::default();

        let mut active = viewport_dimension_at_least_init(Px(700.0), threshold);
        assert!(!active);

        active = viewport_dimension_at_least_apply_hysteresis(
            Px(768.0 + hysteresis.up.0 - 1.0),
            threshold,
            hysteresis,
            active,
        );
        assert!(!active);

        active = viewport_dimension_at_least_apply_hysteresis(
            Px(768.0 + hysteresis.up.0 + 1.0),
            threshold,
            hysteresis,
            active,
        );
        assert!(active);

        active = viewport_dimension_at_least_apply_hysteresis(
            Px(768.0 - hysteresis.down.0 + 1.0),
            threshold,
            hysteresis,
            active,
        );
        assert!(active);

        active = viewport_dimension_at_least_apply_hysteresis(
            Px(768.0 - hysteresis.down.0 - 1.0),
            threshold,
            hysteresis,
            active,
        );
        assert!(!active);
    }

    #[test]
    fn viewport_height_breakpoints_hysteresis_transitions() {
        let breakpoints = &[(Px(640.0), 1u8), (Px(768.0), 2u8), (Px(1024.0), 3u8)];
        let hysteresis = ViewportQueryHysteresis::default();

        let mut active_index = viewport_breakpoints_init_active_index(Px(700.0), breakpoints);
        assert_eq!(active_index, 1);

        active_index = viewport_breakpoints_apply_hysteresis(
            Px(768.0 + hysteresis.up.0 - 1.0),
            breakpoints,
            hysteresis,
            active_index,
        );
        assert_eq!(active_index, 1);

        active_index = viewport_breakpoints_apply_hysteresis(
            Px(768.0 + hysteresis.up.0 + 1.0),
            breakpoints,
            hysteresis,
            active_index,
        );
        assert_eq!(active_index, 2);
    }

    #[test]
    fn viewport_height_at_least_hysteresis_transitions() {
        let threshold = Px(768.0);
        let hysteresis = ViewportQueryHysteresis::default();

        let mut active = viewport_dimension_at_least_init(Px(700.0), threshold);
        assert!(!active);

        active = viewport_dimension_at_least_apply_hysteresis(
            Px(768.0 + hysteresis.up.0 - 1.0),
            threshold,
            hysteresis,
            active,
        );
        assert!(!active);

        active = viewport_dimension_at_least_apply_hysteresis(
            Px(768.0 + hysteresis.up.0 + 1.0),
            threshold,
            hysteresis,
            active,
        );
        assert!(active);
    }
}
