//! Optional state integration for editor controls.
//!
//! This module is feature-gated to avoid forcing a single state stack on all users.

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::controls::{FieldStatus, FieldStatusBadge};

#[cfg(feature = "state-query")]
use fret_query::{QueryState, QueryStatus};

#[cfg(feature = "state-selector")]
use fret_selector::ui::SelectorElementContextExt as _;

#[cfg(feature = "state-selector")]
use std::any::Any;

/// Render a `FieldStatusBadge` for a selector-derived status.
///
/// This mirrors the pattern used by `fret-ui-shadcn`: keep selector integration in a small,
/// opt-in helper so core controls remain state-stack agnostic.
#[cfg(feature = "state-selector")]
#[track_caller]
pub fn use_selector_field_status_badge<H, Deps>(
    cx: &mut ElementContext<'_, H>,
    deps: impl FnOnce(&mut ElementContext<'_, H>) -> Deps,
    compute: impl FnOnce(&mut ElementContext<'_, H>) -> Option<FieldStatus>,
) -> Option<AnyElement>
where
    H: UiHost,
    Deps: Any + PartialEq,
{
    let status = cx.use_selector(deps, compute);
    status.map(|s| FieldStatusBadge::new(s).into_element(cx))
}

/// Map a query state to a compact field status outcome for inspector UIs.
///
/// Policy:
/// - `Loading` and `Error` are surfaced as badges (optional).
/// - `Idle` and `Success` return `None` (common inspector behavior: don't show anything when OK).
#[cfg(feature = "state-query")]
pub fn query_field_status<T>(state: &QueryState<T>) -> Option<FieldStatus> {
    match state.status {
        QueryStatus::Idle => None,
        QueryStatus::Loading => Some(FieldStatus::Loading),
        QueryStatus::Success => None,
        QueryStatus::Error => state
            .error
            .as_ref()
            .map(|e| FieldStatus::Error(e.to_string().into()))
            .or_else(|| Some(FieldStatus::Error("unknown error".into()))),
    }
}

/// Render a `FieldStatusBadge` for a query-derived status (if any).
#[cfg(feature = "state-query")]
pub fn query_field_status_badge<H: UiHost, T>(
    cx: &mut ElementContext<'_, H>,
    state: &QueryState<T>,
) -> Option<AnyElement> {
    query_field_status(state).map(|s| FieldStatusBadge::new(s).into_element(cx))
}
