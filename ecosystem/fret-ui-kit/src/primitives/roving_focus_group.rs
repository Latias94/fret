//! RovingFocusGroup (Radix-aligned outcomes).
//!
//! Radix's roving focus group provides keyboard focus movement within a set of items, typically
//! without adding every item to the Tab order. In Fret, the runtime substrate (`fret-ui`) provides
//! roving focus as a mechanism (`RovingFlex` + action hooks). This module provides a stable,
//! Radix-named entry point for composing common policies (APG navigation + typeahead).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_ui::action::{
    ActionCx, OnRovingActiveChange, OnRovingNavigate, OnRovingTypeahead, RovingNavigateResult,
    RovingTypeaheadCx, UiActionHost,
};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::headless::{roving_focus, typeahead};
use crate::primitives::direction as direction_prim;
use crate::primitives::direction::LayoutDirection;

pub use fret_ui::element::{RovingFlexProps, RovingFocusProps};

/// Returns the first enabled index (Radix roving-focus fallback outcome).
///
/// This is a thin facade over the headless index-math helper so shadcn/recipe layers can depend on
/// the Radix-named `primitives` boundary rather than `headless` internals.
pub fn first_enabled(disabled: &[bool]) -> Option<usize> {
    roving_focus::first_enabled(disabled)
}

/// Returns the last enabled index (Radix roving-focus fallback outcome).
pub fn last_enabled(disabled: &[bool]) -> Option<usize> {
    roving_focus::last_enabled(disabled)
}

/// Returns the next enabled index relative to `current`, respecting `wrap`.
pub fn next_enabled(disabled: &[bool], current: usize, forward: bool, wrap: bool) -> Option<usize> {
    roving_focus::next_enabled(disabled, current, forward, wrap)
}

/// Returns the active index for a string-keyed roving group, preferring `selected` when enabled.
///
/// This matches the common Radix outcome where the "tab stop" item aligns to the currently
/// selected value when present, falling back to the first enabled item.
pub fn active_index_from_str_keys(
    keys: &[Arc<str>],
    selected: Option<&str>,
    disabled: &[bool],
) -> Option<usize> {
    roving_focus::active_index_from_str_keys(keys, selected, disabled)
}

#[derive(Debug, Clone)]
pub enum TypeaheadPolicy {
    None,
    /// Match on the first non-whitespace character of each label.
    FirstChar {
        labels: Arc<[Arc<str>]>,
    },
    /// Match on an accumulated prefix buffer that expires after `timeout_ticks`.
    Prefix {
        labels: Arc<[Arc<str>]>,
        timeout_ticks: u64,
    },
    /// Match on an accumulated prefix buffer that always wraps around the list.
    ///
    /// This is closer to Radix Menu's typeahead outcome: the "next match" search wraps even when
    /// roving navigation does not loop.
    PrefixAlwaysWrap {
        labels: Arc<[Arc<str>]>,
        timeout_ticks: u64,
    },
}

/// Render a `RovingFlex` container with an APG-aligned default navigation policy, plus an optional
/// typeahead policy.
#[track_caller]
pub fn roving_focus_group_apg<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: RovingFlexProps,
    typeahead: TypeaheadPolicy,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    roving_focus_group_apg_with_direction(cx, props, typeahead, LayoutDirection::default(), f)
}

/// Like `roving_focus_group_apg`, but respects Radix `dir` behavior for horizontal navigation.
#[track_caller]
pub fn roving_focus_group_apg_with_direction<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: RovingFlexProps,
    typeahead: TypeaheadPolicy,
    dir: LayoutDirection,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    cx.roving_flex(props, |cx| {
        nav_apg_with_direction(cx, dir);
        match typeahead {
            TypeaheadPolicy::None => {}
            TypeaheadPolicy::FirstChar { labels } => typeahead_first_char_arc_str(cx, labels),
            TypeaheadPolicy::Prefix {
                labels,
                timeout_ticks,
            } => typeahead_prefix_arc_str(cx, labels, timeout_ticks),
            TypeaheadPolicy::PrefixAlwaysWrap {
                labels,
                timeout_ticks,
            } => typeahead_prefix_arc_str_always_wrap(cx, labels, timeout_ticks),
        }
        f(cx)
    })
}

/// Render a `RovingFlex` container with an APG-aligned navigation policy that also supports the
/// common "entry" behavior used by menus: when no item is currently active, Arrow/PageUp/PageDown
/// jump to first/last enabled item.
#[track_caller]
pub fn roving_focus_group_apg_entry_fallback<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: RovingFlexProps,
    typeahead: TypeaheadPolicy,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    roving_focus_group_apg_entry_fallback_with_direction(
        cx,
        props,
        typeahead,
        LayoutDirection::default(),
        f,
    )
}

/// Like `roving_focus_group_apg_entry_fallback`, but respects Radix `dir` behavior for horizontal navigation.
#[track_caller]
pub fn roving_focus_group_apg_entry_fallback_with_direction<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: RovingFlexProps,
    typeahead: TypeaheadPolicy,
    dir: LayoutDirection,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    cx.roving_flex(props, |cx| {
        nav_apg_entry_fallback_with_direction(cx, dir);
        match typeahead {
            TypeaheadPolicy::None => {}
            TypeaheadPolicy::FirstChar { labels } => typeahead_first_char_arc_str(cx, labels),
            TypeaheadPolicy::Prefix {
                labels,
                timeout_ticks,
            } => typeahead_prefix_arc_str(cx, labels, timeout_ticks),
            TypeaheadPolicy::PrefixAlwaysWrap {
                labels,
                timeout_ticks,
            } => typeahead_prefix_arc_str_always_wrap(cx, labels, timeout_ticks),
        }
        f(cx)
    })
}

/// Installs an `on_active_change` handler for the current roving container.
pub fn on_active_change<H: UiHost>(cx: &mut ElementContext<'_, H>, handler: OnRovingActiveChange) {
    cx.roving_on_active_change(handler);
}

/// Installs a keyboard navigation handler for the current roving container.
pub fn on_navigate<H: UiHost>(cx: &mut ElementContext<'_, H>, handler: OnRovingNavigate) {
    cx.roving_on_navigate(handler);
}

/// Installs a typeahead handler for the current roving container.
pub fn on_typeahead<H: UiHost>(cx: &mut ElementContext<'_, H>, handler: OnRovingTypeahead) {
    cx.roving_on_typeahead(handler);
}

/// Installs an APG-aligned default navigation policy (Arrow keys + Home/End).
///
/// The runtime forwards key events to this handler and performs the focus request when a target
/// index is returned.
pub fn nav_apg<H: UiHost>(cx: &mut ElementContext<'_, H>) {
    nav_apg_with_direction(cx, direction_prim::use_direction_in_scope(cx, None));
}

/// Like `nav_apg`, but respects Radix `dir` behavior for horizontal navigation.
///
/// In RTL, Left/Right arrow semantics are flipped.
pub fn nav_apg_with_direction<H: UiHost>(cx: &mut ElementContext<'_, H>, dir: LayoutDirection) {
    cx.roving_add_on_navigate(Arc::new(move |_host, _cx, it| {
        use fret_core::KeyCode;

        match it.key {
            KeyCode::Home => {
                return RovingNavigateResult::Handled {
                    target: roving_focus::first_enabled(&it.disabled),
                };
            }
            KeyCode::End => {
                return RovingNavigateResult::Handled {
                    target: roving_focus::last_enabled(&it.disabled),
                };
            }
            _ => {}
        }

        let Some(current) = it.current else {
            return RovingNavigateResult::NotHandled;
        };

        let forward = match (it.axis, it.key) {
            (fret_core::Axis::Vertical, KeyCode::ArrowDown) => Some(true),
            (fret_core::Axis::Vertical, KeyCode::ArrowUp) => Some(false),
            (fret_core::Axis::Horizontal, KeyCode::ArrowRight | KeyCode::ArrowLeft) => {
                horizontal_forward_for_key(it.key, dir)
            }
            _ => None,
        };

        let Some(forward) = forward else {
            return RovingNavigateResult::NotHandled;
        };

        RovingNavigateResult::Handled {
            target: roving_focus::next_enabled(&it.disabled, current, forward, it.wrap),
        }
    }));
}

/// Like `nav_apg`, but also provides a menu-friendly entry fallback:
/// when no item is active, ArrowDown/PageUp/Home select the first enabled item and
/// ArrowUp/PageDown/End select the last enabled item.
pub fn nav_apg_entry_fallback<H: UiHost>(cx: &mut ElementContext<'_, H>) {
    nav_apg_entry_fallback_with_direction(cx, direction_prim::use_direction_in_scope(cx, None));
}

/// Like `nav_apg_entry_fallback`, but respects Radix `dir` behavior for horizontal navigation.
///
/// In RTL, Left/Right arrow semantics are flipped.
pub fn nav_apg_entry_fallback_with_direction<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    dir: LayoutDirection,
) {
    cx.roving_add_on_navigate(Arc::new(move |_host, _cx, it| {
        use fret_core::KeyCode;

        match it.key {
            KeyCode::Home => {
                return RovingNavigateResult::Handled {
                    target: roving_focus::first_enabled(&it.disabled),
                };
            }
            KeyCode::End => {
                return RovingNavigateResult::Handled {
                    target: roving_focus::last_enabled(&it.disabled),
                };
            }
            _ => {}
        }

        if it.current.is_none() {
            let target = entry_fallback_target_for_key(it.axis, it.key, dir, &it.disabled);

            return RovingNavigateResult::Handled { target };
        }

        let Some(current) = it.current else {
            return RovingNavigateResult::NotHandled;
        };

        let forward = match (it.axis, it.key) {
            (fret_core::Axis::Vertical, KeyCode::ArrowDown) => Some(true),
            (fret_core::Axis::Vertical, KeyCode::ArrowUp) => Some(false),
            (fret_core::Axis::Horizontal, KeyCode::ArrowRight | KeyCode::ArrowLeft) => {
                horizontal_forward_for_key(it.key, dir)
            }
            _ => None,
        };

        let Some(forward) = forward else {
            return RovingNavigateResult::NotHandled;
        };

        RovingNavigateResult::Handled {
            target: roving_focus::next_enabled(&it.disabled, current, forward, it.wrap),
        }
    }));
}

fn horizontal_forward_for_key(key: fret_core::KeyCode, dir: LayoutDirection) -> Option<bool> {
    match (key, dir) {
        (fret_core::KeyCode::ArrowRight, LayoutDirection::Ltr) => Some(true),
        (fret_core::KeyCode::ArrowLeft, LayoutDirection::Ltr) => Some(false),
        (fret_core::KeyCode::ArrowRight, LayoutDirection::Rtl) => Some(false),
        (fret_core::KeyCode::ArrowLeft, LayoutDirection::Rtl) => Some(true),
        _ => None,
    }
}

fn entry_fallback_target_for_key(
    axis: fret_core::Axis,
    key: fret_core::KeyCode,
    dir: LayoutDirection,
    disabled: &[bool],
) -> Option<usize> {
    use fret_core::KeyCode;

    match (axis, key) {
        (fret_core::Axis::Vertical, KeyCode::ArrowDown | KeyCode::PageUp | KeyCode::Home) => {
            roving_focus::first_enabled(disabled)
        }
        (fret_core::Axis::Vertical, KeyCode::ArrowUp | KeyCode::PageDown | KeyCode::End) => {
            roving_focus::last_enabled(disabled)
        }
        (fret_core::Axis::Horizontal, KeyCode::ArrowRight | KeyCode::ArrowLeft) => {
            match horizontal_forward_for_key(key, dir) {
                Some(true) => roving_focus::first_enabled(disabled),
                Some(false) => roving_focus::last_enabled(disabled),
                None => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtl_flips_horizontal_arrow_semantics() {
        use fret_core::KeyCode;

        assert_eq!(
            horizontal_forward_for_key(KeyCode::ArrowRight, LayoutDirection::Ltr),
            Some(true)
        );
        assert_eq!(
            horizontal_forward_for_key(KeyCode::ArrowLeft, LayoutDirection::Ltr),
            Some(false)
        );
        assert_eq!(
            horizontal_forward_for_key(KeyCode::ArrowRight, LayoutDirection::Rtl),
            Some(false)
        );
        assert_eq!(
            horizontal_forward_for_key(KeyCode::ArrowLeft, LayoutDirection::Rtl),
            Some(true)
        );
    }

    #[test]
    fn rtl_flips_entry_fallback_target() {
        use fret_core::{Axis, KeyCode};

        let disabled: [bool; 3] = [false, false, false];

        assert_eq!(
            entry_fallback_target_for_key(
                Axis::Horizontal,
                KeyCode::ArrowRight,
                LayoutDirection::Ltr,
                &disabled
            ),
            Some(0)
        );
        assert_eq!(
            entry_fallback_target_for_key(
                Axis::Horizontal,
                KeyCode::ArrowLeft,
                LayoutDirection::Ltr,
                &disabled
            ),
            Some(2)
        );
        assert_eq!(
            entry_fallback_target_for_key(
                Axis::Horizontal,
                KeyCode::ArrowRight,
                LayoutDirection::Rtl,
                &disabled
            ),
            Some(2)
        );
        assert_eq!(
            entry_fallback_target_for_key(
                Axis::Horizontal,
                KeyCode::ArrowLeft,
                LayoutDirection::Rtl,
                &disabled
            ),
            Some(0)
        );
    }
}

/// Install a first-character typeahead policy for the current roving container.
pub fn typeahead_first_char_arc_str<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    labels: Arc<[Arc<str>]>,
) {
    struct TypeaheadFirstCharState {
        labels: Rc<RefCell<Arc<[Arc<str>]>>>,
        handler: OnRovingTypeahead,
    }

    let handler = cx.with_state(
        || {
            let labels_cell: Rc<RefCell<Arc<[Arc<str>]>>> = Rc::new(RefCell::new(labels.clone()));
            let labels_read = labels_cell.clone();

            #[allow(clippy::arc_with_non_send_sync)]
            let handler: OnRovingTypeahead =
                Arc::new(move |_host: &mut dyn UiActionHost, _cx: ActionCx, it| {
                    let labels = labels_read.borrow();
                    let is_disabled = |idx: usize| it.disabled.get(idx).copied().unwrap_or(false);
                    let matches = |idx: usize| -> bool {
                        if is_disabled(idx) {
                            return false;
                        }
                        let Some(label) = labels.get(idx) else {
                            return false;
                        };
                        let label = label.as_ref().trim_start();
                        let Some(first) = label.chars().next() else {
                            return false;
                        };
                        first.to_ascii_lowercase() == it.input
                    };

                    let start = it.current.map(|i| i.saturating_add(1)).unwrap_or(0);
                    if it.wrap {
                        for offset in 0..it.len {
                            let idx = (start + offset) % it.len;
                            if matches(idx) {
                                return Some(idx);
                            }
                        }
                        None
                    } else {
                        (start..it.len).find(|&idx| matches(idx))
                    }
                });

            TypeaheadFirstCharState {
                labels: labels_cell,
                handler,
            }
        },
        |state| {
            *state.labels.borrow_mut() = labels.clone();
            state.handler.clone()
        },
    );

    cx.roving_add_on_typeahead(handler);
}

/// Install a prefix-buffer typeahead policy for the current roving container.
pub fn typeahead_prefix_arc_str<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    labels: Arc<[Arc<str>]>,
    timeout_ticks: u64,
) {
    struct TypeaheadPrefixState {
        timeout_ticks: u64,
        labels: Rc<RefCell<Arc<[Arc<str>]>>>,
        handler: OnRovingTypeahead,
    }

    fn make_state(labels: Arc<[Arc<str>]>, timeout_ticks: u64) -> TypeaheadPrefixState {
        let labels_cell: Rc<RefCell<Arc<[Arc<str>]>>> = Rc::new(RefCell::new(labels));
        let buffer: Rc<RefCell<typeahead::TypeaheadBuffer>> =
            Rc::new(RefCell::new(typeahead::TypeaheadBuffer::new(timeout_ticks)));

        let labels_read = labels_cell.clone();
        let buffer_read = buffer.clone();

        #[allow(clippy::arc_with_non_send_sync)]
        let handler: OnRovingTypeahead = Arc::new(
            move |_host: &mut dyn UiActionHost, _cx: ActionCx, it: RovingTypeaheadCx| {
                let mut buf = buffer_read.borrow_mut();
                buf.push_char(it.input, it.tick);
                let query = buf.query(it.tick)?;

                let labels = labels_read.borrow();
                typeahead::match_prefix_arc_str(
                    labels.as_ref(),
                    it.disabled.as_ref(),
                    query,
                    it.current,
                    it.wrap,
                )
            },
        );

        TypeaheadPrefixState {
            timeout_ticks,
            labels: labels_cell,
            handler,
        }
    }

    let handler = cx.with_state(
        || make_state(labels.clone(), timeout_ticks),
        |state| {
            if state.timeout_ticks != timeout_ticks {
                *state = make_state(labels.clone(), timeout_ticks);
            }
            *state.labels.borrow_mut() = labels.clone();
            state.handler.clone()
        },
    );

    cx.roving_add_on_typeahead(handler);
}

/// Install a prefix-buffer typeahead policy for the current roving container that always wraps.
///
/// This matches Radix Menu's `getNextMatch` behavior (it searches circularly) even when roving
/// navigation uses `loop=false`.
pub fn typeahead_prefix_arc_str_always_wrap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    labels: Arc<[Arc<str>]>,
    timeout_ticks: u64,
) {
    struct TypeaheadPrefixState {
        timeout_ticks: u64,
        labels: Rc<RefCell<Arc<[Arc<str>]>>>,
        handler: OnRovingTypeahead,
    }

    fn make_state(labels: Arc<[Arc<str>]>, timeout_ticks: u64) -> TypeaheadPrefixState {
        let labels_cell: Rc<RefCell<Arc<[Arc<str>]>>> = Rc::new(RefCell::new(labels));
        let buffer: Rc<RefCell<typeahead::TypeaheadBuffer>> =
            Rc::new(RefCell::new(typeahead::TypeaheadBuffer::new(timeout_ticks)));

        let labels_read = labels_cell.clone();
        let buffer_read = buffer.clone();

        #[allow(clippy::arc_with_non_send_sync)]
        let handler: OnRovingTypeahead = Arc::new(
            move |_host: &mut dyn UiActionHost, _cx: ActionCx, it: RovingTypeaheadCx| {
                let mut buf = buffer_read.borrow_mut();
                buf.push_char(it.input, it.tick);
                let query = buf.query(it.tick)?;

                let labels = labels_read.borrow();
                typeahead::match_prefix_arc_str(
                    labels.as_ref(),
                    it.disabled.as_ref(),
                    query,
                    it.current,
                    true,
                )
            },
        );

        TypeaheadPrefixState {
            timeout_ticks,
            labels: labels_cell,
            handler,
        }
    }

    let handler = cx.with_state(
        || make_state(labels.clone(), timeout_ticks),
        |state| {
            if state.timeout_ticks != timeout_ticks {
                *state = make_state(labels.clone(), timeout_ticks);
            }
            *state.labels.borrow_mut() = labels.clone();
            state.handler.clone()
        },
    );

    cx.roving_add_on_typeahead(handler);
}
