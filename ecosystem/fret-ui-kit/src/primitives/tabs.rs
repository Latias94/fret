//! Tabs primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing tabs behavior in recipes.
//! It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/tabs/src/tabs.tsx`

use std::sync::Arc;

use fret_core::{Modifiers, MouseButton, PointerType, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, LayoutStyle, PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps,
    SemanticsProps,
};
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::declarative::action_hooks::ActionHooksExt as _;

/// Returns a selected-value model that behaves like Radix `useControllableState` (`value` /
/// `defaultValue`).
pub fn tabs_use_value_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Option<Arc<str>>>>,
    default_value: impl FnOnce() -> Option<Arc<str>>,
) -> crate::primitives::controllable_state::ControllableModel<Option<Arc<str>>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

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

/// A11y metadata for a `TabsList` container.
pub fn tab_list_semantics_props(layout: LayoutStyle) -> SemanticsProps {
    SemanticsProps {
        layout,
        role: SemanticsRole::TabList,
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

/// A composable, Radix-shaped tabs configuration surface (`TabsRoot` / `TabsList` / `TabsTrigger` /
/// `TabsContent`).
///
/// Unlike Radix React, Fret does not use context objects; the "composition" surface is expressed as
/// small Rust builders that thread the shared models and option values through closures.
#[derive(Debug, Clone)]
pub struct TabsRoot {
    model: Model<Option<Arc<str>>>,
    disabled: bool,
    orientation: TabsOrientation,
    activation_mode: TabsActivationMode,
    loop_navigation: bool,
}

impl TabsRoot {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model,
            disabled: false,
            orientation: TabsOrientation::default(),
            activation_mode: TabsActivationMode::default(),
            loop_navigation: true,
        }
    }

    pub fn model(&self) -> Model<Option<Arc<str>>> {
        self.model.clone()
    }

    /// Creates a tabs root with a controlled/uncontrolled selection model (Radix `value` /
    /// `defaultValue`).
    ///
    /// Notes:
    /// - The internal model (uncontrolled mode) is stored in element state at the call site.
    /// - Call this from a stable subtree (key the root node if you need state to survive reordering).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        controlled: Option<Model<Option<Arc<str>>>>,
        default_value: impl FnOnce() -> Option<Arc<str>>,
    ) -> Self {
        let model = tabs_use_value_model(cx, controlled, default_value).model();
        Self::new(model)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn orientation(mut self, orientation: TabsOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn activation_mode(mut self, activation_mode: TabsActivationMode) -> Self {
        self.activation_mode = activation_mode;
        self
    }

    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn list(self, values: Arc<[Arc<str>]>, disabled: Arc<[bool]>) -> TabsList {
        TabsList::new(self, values, disabled)
    }

    pub fn trigger(&self, value: impl Into<Arc<str>>) -> TabsTrigger {
        TabsTrigger::new(value)
    }

    pub fn content(&self, value: impl Into<Arc<str>>) -> TabsContent {
        TabsContent::new(value)
    }
}

#[derive(Debug, Clone)]
pub struct TabsList {
    root: TabsRoot,
    values: Arc<[Arc<str>]>,
    disabled: Arc<[bool]>,
    layout: LayoutStyle,
}

impl TabsList {
    pub fn new(root: TabsRoot, values: Arc<[Arc<str>]>, disabled: Arc<[bool]>) -> Self {
        Self {
            root,
            values,
            disabled,
            layout: LayoutStyle::default(),
        }
    }

    pub fn layout(mut self, layout: LayoutStyle) -> Self {
        self.layout = layout;
        self
    }

    /// Renders a tabs list semantics root containing a roving-focus group.
    ///
    /// Notes:
    /// - This does not apply any visual skin. Pass `flex` / `layout` via `RovingFlexProps`.
    /// - This installs APG navigation and, in automatic mode, updates `TabsRoot.model` when the
    ///   active tab changes.
    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        mut props: RovingFlexProps,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        let model = self.root.model.clone();
        let activation_mode = self.root.activation_mode;
        let disabled_for_roving = self.disabled.clone();
        let values_for_roving = self.values.clone();

        props.flex.direction = match self.root.orientation {
            TabsOrientation::Horizontal => fret_core::Axis::Horizontal,
            TabsOrientation::Vertical => fret_core::Axis::Vertical,
        };
        props.roving = RovingFocusProps {
            enabled: props.roving.enabled && !self.root.disabled,
            wrap: self.root.loop_navigation,
            disabled: disabled_for_roving,
        };

        let layout = self.layout;
        cx.semantics(tab_list_semantics_props(layout), move |cx| {
            vec![cx.roving_flex(props, move |cx| {
                cx.roving_nav_apg();
                if activation_mode == TabsActivationMode::Automatic {
                    cx.roving_select_option_arc_str(&model, values_for_roving.clone());
                }
                f(cx)
            })]
        })
    }
}

#[derive(Debug, Clone)]
pub struct TabsTrigger {
    value: Arc<str>,
    label: Option<Arc<str>>,
    disabled: bool,
    index: Option<usize>,
    tab_stop: bool,
    set_size: Option<u32>,
}

impl TabsTrigger {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: None,
            disabled: false,
            index: None,
            tab_stop: false,
            set_size: None,
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Optional 0-based index used to populate collection metadata (`pos_in_set`).
    pub fn index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    /// Whether this trigger is the current "tab stop" in roving focus terms.
    ///
    /// When `true`, `PressableProps.focusable` will be enabled even when the element isn't focused.
    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    /// Optional set size used to populate collection metadata (`set_size`).
    pub fn set_size(mut self, set_size: Option<u32>) -> Self {
        self.set_size = set_size;
        self
    }

    /// Renders a `TabsTrigger` as a pressable, wiring Radix-like pointer and activation behavior.
    ///
    /// - Selects the tab on left mouse down (no ctrl key), matching Radix's `onMouseDown`.
    /// - Activates selection on pressable "activate" as well (Enter/Space and click-like pointer up).
    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        root: &TabsRoot,
        mut props: PressableProps,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        let model = root.model.clone();
        let value = self.value.clone();
        let label = self.label.clone();
        let disabled = self.disabled || root.disabled;
        let tab_stop = self.tab_stop;
        let pos_in_set = self
            .index
            .and_then(|idx| u32::try_from(idx.saturating_add(1)).ok());
        let set_size = self.set_size;

        cx.pressable_with_id_props(move |cx, st, _id| {
            let value_for_pointer = value.clone();
            let model_for_pointer = model.clone();

            cx.pressable_add_on_pointer_down(Arc::new(move |host, _cx, down| {
                use fret_ui::action::PressablePointerDownResult as R;

                match tabs_trigger_pointer_down_action(
                    down.pointer_type,
                    down.button,
                    down.modifiers,
                    disabled,
                ) {
                    TabsTriggerPointerDownAction::Select => {
                        let _ = host
                            .models_mut()
                            .update(&model_for_pointer, |v| *v = Some(value_for_pointer.clone()));
                        R::Continue
                    }
                    TabsTriggerPointerDownAction::PreventFocus => {
                        host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                        R::SkipDefault
                    }
                    TabsTriggerPointerDownAction::Ignore => R::Continue,
                }
            }));

            // Ensure Enter/Space activation (and click-like pointer-up) also selects this tab.
            cx.pressable_set_option_arc_str(&model, value.clone());

            let selected_value = cx.watch_model(&model).layout().cloned().flatten();
            let selected = selected_value.as_deref() == Some(value.as_ref());

            props.enabled = !disabled;
            // Roving focus: only the tab stop participates in the default tab order.
            props.focusable = (!disabled) && (tab_stop || st.focused);
            props.a11y = tab_a11y_with_collection(label.clone(), selected, pos_in_set, set_size);

            (props, f(cx))
        })
    }
}

#[derive(Debug, Clone)]
pub struct TabsContent {
    value: Arc<str>,
    label: Option<Arc<str>>,
    labelled_by_element: Option<u64>,
    force_mount: bool,
    layout: LayoutStyle,
}

impl TabsContent {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: None,
            labelled_by_element: None,
            force_mount: false,
            layout: LayoutStyle::default(),
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn labelled_by_element(mut self, labelled_by_element: Option<u64>) -> Self {
        self.labelled_by_element = labelled_by_element;
        self
    }

    pub fn force_mount(mut self, force_mount: bool) -> Self {
        self.force_mount = force_mount;
        self
    }

    pub fn layout(mut self, layout: LayoutStyle) -> Self {
        self.layout = layout;
        self
    }

    /// Renders a `TabsContent` (tab panel) subtree if it is active or force-mounted.
    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        root: &TabsRoot,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> Option<AnyElement> {
        let selected_value: Option<Arc<str>> =
            cx.watch_model(&root.model).layout().cloned().flatten();
        let active = selected_value.as_deref() == Some(self.value.as_ref());
        tab_panel_with_gate(
            cx,
            active,
            self.force_mount,
            self.layout,
            self.label,
            self.labelled_by_element,
            f,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn tabs_use_value_model_prefers_controlled_and_does_not_call_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(Some(Arc::from("a")));
        let called = Cell::new(0);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let out = tabs_use_value_model(cx, Some(controlled.clone()), || {
                called.set(called.get() + 1);
                None
            });
            assert!(out.is_controlled());
            assert_eq!(out.model(), controlled);
        });

        assert_eq!(called.get(), 0);
    }

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
