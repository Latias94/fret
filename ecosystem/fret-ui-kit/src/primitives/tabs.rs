//! Tabs primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing tabs behavior in recipes.
//! It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/tabs/src/tabs.tsx`

use std::sync::Arc;

use fret_core::{Modifiers, MouseButton, PointerType, SemanticsRole};
use fret_ui::element::{AnyElement, LayoutStyle, PressableA11y, SemanticsProps};
use fret_ui::{ElementContext, UiHost};

/// Matches Radix Tabs `orientation` outcome: horizontal (default) vs vertical layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Matches Radix Tabs `activationMode` outcome:
/// - `Automatic`: moving focus (arrow keys) activates the tab.
/// - `Manual`: moving focus does not activate; activation happens on click/Enter/Space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsActivationMode {
    #[default]
    Automatic,
    Manual,
}

/// Mirrors the Radix `TabsTrigger` `onMouseDown` behavior:
/// - left mouse down selects the tab,
/// - other mouse buttons / ctrl-click do not select and should avoid focusing the trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsTriggerPointerDownAction {
    Select,
    PreventFocus,
    Ignore,
}

/// Decide what to do with a pointer down event on a tabs trigger.
///
/// Radix selects on `onMouseDown` (left button only, no ctrl key) and prevents focus for other
/// mouse downs. Touch/pen are ignored here so they can keep the click-like "activate on up"
/// behavior.
pub fn tabs_trigger_pointer_down_action(
    pointer_type: PointerType,
    button: MouseButton,
    modifiers: Modifiers,
    disabled: bool,
) -> TabsTriggerPointerDownAction {
    match pointer_type {
        PointerType::Touch | PointerType::Pen => TabsTriggerPointerDownAction::Ignore,
        PointerType::Mouse | PointerType::Unknown => {
            if disabled {
                return TabsTriggerPointerDownAction::PreventFocus;
            }

            if button == MouseButton::Left && !modifiers.ctrl {
                TabsTriggerPointerDownAction::Select
            } else {
                TabsTriggerPointerDownAction::PreventFocus
            }
        }
    }
}

/// A11y metadata for a tab-like pressable.
pub fn tab_a11y(label: Option<Arc<str>>, selected: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::Tab),
        label,
        selected,
        ..Default::default()
    }
}

/// A11y metadata for a tab-like pressable with collection position metadata.
///
/// This aligns with the APG/Radix expectation that tabs participate in a logical set and can
/// expose their 1-based position within that set.
pub fn tab_a11y_with_collection(
    label: Option<Arc<str>>,
    selected: bool,
    pos_in_set: Option<u32>,
    set_size: Option<u32>,
) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::Tab),
        label,
        selected,
        pos_in_set,
        set_size,
        ..Default::default()
    }
}

/// Maps a selected `value` (string key) to the active index, skipping disabled items.
///
/// This is the Radix outcome "value controls which trigger is active", expressed in Fret terms.
pub fn active_index_from_values(
    values: &[Arc<str>],
    selected: Option<&str>,
    disabled: &[bool],
) -> Option<usize> {
    crate::headless::roving_focus::active_index_from_str_keys(values, selected, disabled)
}

/// Builds semantics props for a `TabPanel` node.
pub fn tab_panel_semantics_props(
    layout: LayoutStyle,
    label: Option<Arc<str>>,
    labelled_by_element: Option<u64>,
) -> SemanticsProps {
    SemanticsProps {
        layout,
        role: SemanticsRole::TabPanel,
        label,
        labelled_by_element,
        ..Default::default()
    }
}

/// Builds a tab panel subtree, optionally force-mounting it behind an interactivity gate.
///
/// This is a Radix-aligned outcome wrapper for `TabsContent forceMount`:
/// - When `force_mount=false`, inactive panels are not mounted.
/// - When `force_mount=true`, inactive panels remain mounted but are not present/interactive.
#[track_caller]
pub fn tab_panel_with_gate<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    active: bool,
    force_mount: bool,
    layout: LayoutStyle,
    label: Option<Arc<str>>,
    labelled_by_element: Option<u64>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> Option<AnyElement> {
    if !active && !force_mount {
        return None;
    }

    let panel = |cx: &mut ElementContext<'_, H>| {
        cx.semantics(
            tab_panel_semantics_props(layout, label, labelled_by_element),
            children,
        )
    };

    if force_mount {
        Some(cx.interactivity_gate(active, active, |cx| vec![panel(cx)]))
    } else {
        Some(panel(cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_trigger_pointer_down_selects_on_left_mouse_down() {
        let action = tabs_trigger_pointer_down_action(
            PointerType::Mouse,
            MouseButton::Left,
            Modifiers::default(),
            false,
        );
        assert_eq!(action, TabsTriggerPointerDownAction::Select);
    }

    #[test]
    fn tabs_trigger_pointer_down_prevents_focus_on_ctrl_click() {
        let mut modifiers = Modifiers::default();
        modifiers.ctrl = true;

        let action = tabs_trigger_pointer_down_action(
            PointerType::Mouse,
            MouseButton::Left,
            modifiers,
            false,
        );
        assert_eq!(action, TabsTriggerPointerDownAction::PreventFocus);
    }

    #[test]
    fn tabs_trigger_pointer_down_ignores_touch_to_preserve_click_like_activation() {
        let action = tabs_trigger_pointer_down_action(
            PointerType::Touch,
            MouseButton::Left,
            Modifiers::default(),
            false,
        );
        assert_eq!(action, TabsTriggerPointerDownAction::Ignore);
    }

    #[test]
    fn tab_panel_semantics_props_sets_role_and_labelled_by() {
        let props =
            tab_panel_semantics_props(LayoutStyle::default(), Some(Arc::from("Panel")), Some(123));
        assert_eq!(props.role, SemanticsRole::TabPanel);
        assert_eq!(props.label.as_deref(), Some("Panel"));
        assert_eq!(props.labelled_by_element, Some(123));
    }
}
