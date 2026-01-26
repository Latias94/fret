//! Toolbar primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/toolbar/src/toolbar.tsx`
//!
//! Radix `Toolbar` is primarily a composition of:
//! - `RovingFocusGroup` (orientation + loop navigation),
//! - `Separator` orientation mapping,
//! - `ToggleGroup` (with `rovingFocus=false` because the toolbar owns roving focus).
//!
//! This module exposes thin helpers for those outcomes. Visual styling and full a11y semantics
//! (e.g. `role="toolbar"`) are intentionally deferred to recipes and the a11y roadmap.

use std::sync::Arc;

use fret_ui::element::{AnyElement, RovingFlexProps};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::direction::LayoutDirection;
use crate::primitives::roving_focus_group::{self, TypeaheadPolicy};
use crate::primitives::separator::SeparatorOrientation;

/// Matches Radix Toolbar `orientation` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolbarOrientation {
    #[default]
    Horizontal,
    Vertical,
}

pub fn toolbar_separator_orientation(orientation: ToolbarOrientation) -> SeparatorOrientation {
    match orientation {
        ToolbarOrientation::Horizontal => SeparatorOrientation::Vertical,
        ToolbarOrientation::Vertical => SeparatorOrientation::Horizontal,
    }
}

/// Build `RovingFlexProps` for a toolbar roving focus group.
///
/// Radix defaults:
/// - `orientation = horizontal`
/// - `loop = true`
pub fn toolbar_roving_flex_props(
    orientation: ToolbarOrientation,
    loop_navigation: bool,
    disabled: Arc<[bool]>,
) -> RovingFlexProps {
    let mut props = RovingFlexProps::default();
    props.flex.direction = match orientation {
        ToolbarOrientation::Horizontal => fret_core::Axis::Horizontal,
        ToolbarOrientation::Vertical => fret_core::Axis::Vertical,
    };
    props.roving.wrap = loop_navigation;
    props.roving.disabled = disabled;
    props
}

/// Render a toolbar-like roving focus group with an APG-aligned navigation policy.
#[track_caller]
pub fn toolbar_roving_group_apg<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: RovingFlexProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    roving_focus_group::roving_focus_group_apg(cx, props, TypeaheadPolicy::None, children)
}

/// Like `toolbar_roving_group_apg`, but respects Radix `dir` behavior for horizontal navigation.
#[track_caller]
pub fn toolbar_roving_group_apg_with_direction<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: RovingFlexProps,
    dir: LayoutDirection,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    roving_focus_group::roving_focus_group_apg_with_direction(
        cx,
        props,
        TypeaheadPolicy::None,
        dir,
        children,
    )
}
