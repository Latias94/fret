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

pub use fret_ui::element::{RovingFlexProps, RovingFocusProps};

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
}

/// Render a `RovingFlex` container with an APG-aligned default navigation policy, plus an optional
/// typeahead policy.
#[track_caller]
pub fn roving_focus_group_apg<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: RovingFlexProps,
    typeahead: TypeaheadPolicy,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    cx.roving_flex(props, |cx| {
        nav_apg(cx);
        match typeahead {
            TypeaheadPolicy::None => {}
            TypeaheadPolicy::FirstChar { labels } => typeahead_first_char_arc_str(cx, labels),
            TypeaheadPolicy::Prefix {
                labels,
                timeout_ticks,
            } => typeahead_prefix_arc_str(cx, labels, timeout_ticks),
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
    cx.roving_add_on_navigate(Arc::new(|_host, _cx, it| {
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
            (fret_core::Axis::Horizontal, KeyCode::ArrowRight) => Some(true),
            (fret_core::Axis::Horizontal, KeyCode::ArrowLeft) => Some(false),
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
