//! Menu content composition helpers (Radix-aligned outcomes).
//!
//! Radix `Menu` (`@radix-ui/react-menu`) embeds roving focus + typeahead behavior inside the menu
//! implementation rather than exporting a dedicated package. In Fret we keep the reusable
//! mechanisms (`RovingFlex`, typeahead) in `fret-ui` + `headless`, and expose stable composition
//! points via primitives.
//!
//! This module provides menu-specific naming aliases so wrappers like `DropdownMenu` and
//! `ContextMenu` can stay conceptually aligned with Radix (`MenuContent`).

use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::roving_focus_group;

pub use crate::primitives::roving_focus_group::{
    RovingFlexProps, RovingFocusProps, TypeaheadPolicy,
};

/// Render a menu "content" list with APG-aligned roving focus and an explicit typeahead policy.
#[track_caller]
pub fn menu_roving_group_apg<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: RovingFlexProps,
    typeahead: TypeaheadPolicy,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    roving_focus_group::roving_focus_group_apg_entry_fallback(cx, props, typeahead, f)
}

/// Convenience helper for the most common menu behavior: prefix-buffer typeahead.
#[track_caller]
pub fn menu_roving_group_apg_prefix_typeahead<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: RovingFlexProps,
    labels: Arc<[Arc<str>]>,
    timeout_ticks: u64,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    menu_roving_group_apg(
        cx,
        props,
        TypeaheadPolicy::PrefixAlwaysWrap {
            labels,
            timeout_ticks,
        },
        f,
    )
}
