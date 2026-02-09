use std::sync::Arc;

use fret_core::{Px, Rect};
use fret_ui::element::{AnyElement, LayoutQueryRegionProps};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

/// Tailwind-compatible width breakpoints.
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
pub struct ContainerQueryHysteresis {
    pub up: Px,
    pub down: Px,
}

impl Default for ContainerQueryHysteresis {
    fn default() -> Self {
        // Keep the default small: enough to avoid single-pixel oscillation without delaying
        // responsive behavior too much in editor-grade resize drags.
        Self {
            up: Px(8.0),
            down: Px(8.0),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ContainerBreakpointsState {
    /// 0 = base value, i>0 selects `breakpoints[i-1]`.
    active_index: usize,
    initialized: bool,
}

/// Marks a subtree as a container-query region.
///
/// This is a mechanism-only wrapper: it is paint- and input-transparent, but records committed
/// bounds that can be read via [`ElementContext::layout_query_bounds`] (ADR 1170).
#[track_caller]
pub fn container_query_region_with_id<H, I>(
    cx: &mut ElementContext<'_, H>,
    name: impl Into<Arc<str>>,
    mut props: LayoutQueryRegionProps,
    f: impl FnOnce(&mut ElementContext<'_, H>, GlobalElementId) -> I,
) -> AnyElement
where
    H: UiHost,
    I: IntoIterator<Item = AnyElement>,
{
    props.name = Some(name.into());
    cx.layout_query_region_with_id(props, f)
}

#[track_caller]
pub fn container_query_region<H, I>(
    cx: &mut ElementContext<'_, H>,
    name: impl Into<Arc<str>>,
    props: LayoutQueryRegionProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    H: UiHost,
    I: IntoIterator<Item = AnyElement>,
{
    container_query_region_with_id(cx, name, props, |cx, _id| f(cx))
}

/// Resolves a breakpoint-driven variant based on the committed width of a query region.
///
/// Contract notes:
///
/// - Observations are frame-lagged (read last committed bounds only).
/// - Width changes participate in invalidation via `layout_query_bounds` (ADR 1170 D4).
/// - Hysteresis is applied to avoid oscillation when layout branches affect container size.
///
/// Breakpoint table semantics:
///
/// - `base` is returned when no breakpoints match.
/// - Each `(min_width, value)` activates when `width >= min_width`.
/// - The table must be sorted by ascending `min_width`.
#[track_caller]
pub fn container_breakpoints<H, T: Copy>(
    cx: &mut ElementContext<'_, H>,
    region: GlobalElementId,
    invalidation: Invalidation,
    base: T,
    breakpoints: &[(Px, T)],
    hysteresis: ContainerQueryHysteresis,
) -> T
where
    H: UiHost,
{
    // Ensure each callsite gets its own stable element identity for hysteresis state.
    cx.scope(|cx| {
        let rect: Option<Rect> = cx.layout_query_bounds(region, invalidation);
        let Some(width) = rect.map(|r| r.size.width) else {
            return base;
        };

        cx.with_state(ContainerBreakpointsState::default, |st| {
            if !st.initialized {
                st.active_index = 0;
                for (i, (min_width, _)) in breakpoints.iter().enumerate() {
                    if width.0 >= min_width.0 {
                        st.active_index = i + 1;
                    }
                }
                st.initialized = true;
            }

            loop {
                if st.active_index >= breakpoints.len() {
                    break;
                }
                let next_min_width = breakpoints[st.active_index].0;
                if width.0 >= next_min_width.0 + hysteresis.up.0 {
                    st.active_index = st.active_index.saturating_add(1);
                    continue;
                }
                break;
            }

            loop {
                if st.active_index == 0 {
                    break;
                }
                let cur_min_width = breakpoints[st.active_index - 1].0;
                if width.0 < cur_min_width.0 - hysteresis.down.0 {
                    st.active_index = st.active_index.saturating_sub(1);
                    continue;
                }
                break;
            }

            if st.active_index == 0 {
                return base;
            }
            breakpoints
                .get(st.active_index - 1)
                .map(|(_, v)| *v)
                .unwrap_or(base)
        })
    })
}
