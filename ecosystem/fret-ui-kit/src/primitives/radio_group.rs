//! RadioGroup primitives (Radix-aligned outcomes).
//!
//! Radix `RadioGroup` composes:
//! - a group-level semantics container, and
//! - radio button items that expose checked/disabled state.
//!
//! In Fret, roving focus + selection policy is composed by wrappers (recipe layer) using
//! `RovingFlex` + action hooks; this module provides stable, Radix-named building blocks for
//! semantics/a11y and a composable configuration surface for recipes.
//!
//! This file is part of the stable primitives surface (do not move without an ADR update).

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::element::{
    PressableA11y, PressableProps, PressableState, RovingFlexProps, RovingFocusProps,
    SemanticsProps,
};
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt as _;
use crate::declarative::action_hooks::ActionHooksExt as _;
use crate::declarative::collection_semantics::CollectionSemanticsExt as _;

/// Semantics wrapper props for a radio group container.
pub fn radio_group_semantics(
    label: Option<Arc<str>>,
    disabled: bool,
    required: bool,
) -> SemanticsProps {
    SemanticsProps {
        role: SemanticsRole::RadioGroup,
        label,
        disabled,
        required,
        ..Default::default()
    }
}

/// A11y metadata for a radio button-like pressable.
pub fn radio_button_a11y(label: Option<Arc<str>>, checked: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::RadioButton),
        label,
        checked: Some(checked),
        ..Default::default()
    }
}

/// Returns a selection model for a radio group that behaves like Radix `useControllableState`
/// (`value` / `defaultValue`).
pub fn radio_group_use_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Option<Arc<str>>>>,
    default_value: impl FnOnce() -> Option<Arc<str>>,
) -> crate::primitives::controllable_state::ControllableModel<Option<Arc<str>>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

/// Matches Radix RadioGroup `orientation` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadioGroupOrientation {
    #[default]
    Vertical,
    Horizontal,
}

/// A Radix-shaped `RadioGroup` root configuration surface.
#[derive(Debug, Clone)]
pub struct RadioGroupRoot {
    model: Model<Option<Arc<str>>>,
    disabled: bool,
    required: bool,
    orientation: RadioGroupOrientation,
    loop_navigation: bool,
    a11y_label: Option<Arc<str>>,
}

impl RadioGroupRoot {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model,
            disabled: false,
            required: false,
            orientation: RadioGroupOrientation::default(),
            loop_navigation: true,
            a11y_label: None,
        }
    }

    /// Creates a radio group with a controlled/uncontrolled selection model (Radix `value` /
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
        let model = radio_group_use_model(cx, controlled, default_value).model();
        Self::new(model)
    }

    pub fn model(&self) -> Model<Option<Arc<str>>> {
        self.model.clone()
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn orientation(mut self, orientation: RadioGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// When `true` (default), roving navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn list(self, values: Arc<[Arc<str>]>, disabled: Arc<[bool]>) -> RadioGroupList {
        RadioGroupList::new(self, values, disabled)
    }

    pub fn item(&self, value: impl Into<Arc<str>>) -> RadioGroupItem {
        RadioGroupItem::new(value)
    }
}

#[derive(Debug, Clone)]
pub struct RadioGroupList {
    root: RadioGroupRoot,
    values: Arc<[Arc<str>]>,
    disabled: Arc<[bool]>,
}

impl RadioGroupList {
    pub fn new(root: RadioGroupRoot, values: Arc<[Arc<str>]>, disabled: Arc<[bool]>) -> Self {
        Self {
            root,
            values,
            disabled,
        }
    }

    /// Renders a radio group semantics root containing a roving-focus group.
    ///
    /// Notes:
    /// - This does not apply any visual skin. Pass `flex` / `layout` via `RovingFlexProps`.
    /// - This installs APG navigation and selects on roving focus changes, matching Radix behavior.
    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        mut props: RovingFlexProps,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<fret_ui::element::AnyElement>,
    ) -> fret_ui::element::AnyElement {
        let model = self.root.model.clone();
        let disabled_for_roving = self.disabled.clone();
        let values_for_roving = self.values.clone();
        let disabled_group = self.root.disabled;
        let required = self.root.required;
        let label = self.root.a11y_label.clone();

        props.flex.direction = match self.root.orientation {
            RadioGroupOrientation::Vertical => fret_core::Axis::Vertical,
            RadioGroupOrientation::Horizontal => fret_core::Axis::Horizontal,
        };
        props.roving = RovingFocusProps {
            enabled: props.roving.enabled && !disabled_group,
            wrap: self.root.loop_navigation,
            disabled: disabled_for_roving,
        };

        cx.semantics(
            radio_group_semantics(label, disabled_group, required),
            move |cx| {
                vec![cx.roving_flex(props, move |cx| {
                    cx.roving_nav_apg();
                    cx.roving_select_option_arc_str(&model, values_for_roving.clone());
                    f(cx)
                })]
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct RadioGroupItem {
    value: Arc<str>,
    label: Option<Arc<str>>,
    disabled: bool,
    index: Option<usize>,
    tab_stop: bool,
    set_size: Option<u32>,
}

impl RadioGroupItem {
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

    /// Whether this item is the current "tab stop" in roving focus terms.
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

    /// Renders a radio group item as a pressable, wiring Radix-like selection behavior.
    ///
    /// - Arrow-key roving selects via [`RadioGroupList::into_element`].
    /// - Click/Space activation selects via `pressable_set_option_arc_str`.
    /// - Enter key presses are consumed to match Radix/WAI-ARIA expectations.
    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        root: &RadioGroupRoot,
        props: PressableProps,
        f: impl FnOnce(
            &mut ElementContext<'_, H>,
            PressableState,
            bool,
        ) -> Vec<fret_ui::element::AnyElement>,
    ) -> fret_ui::element::AnyElement {
        self.into_element_with_props_hook(cx, root, props, move |cx, st, _id, checked, _props| {
            f(cx, st, checked)
        })
    }

    /// Like [`RadioGroupItem::into_element`], but allows the caller to mutate the computed
    /// `PressableProps` after Radix-like selection/a11y wiring has been applied.
    ///
    /// This is primarily intended for recipe-layer motion/visual tweaks that need access to
    /// the item's `GlobalElementId` and the current pressable state.
    #[track_caller]
    pub fn into_element_with_props_hook<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        root: &RadioGroupRoot,
        props: PressableProps,
        f: impl FnOnce(
            &mut ElementContext<'_, H>,
            PressableState,
            fret_ui::elements::GlobalElementId,
            bool,
            &mut PressableProps,
        ) -> Vec<fret_ui::element::AnyElement>,
    ) -> fret_ui::element::AnyElement {
        let mut props = props;
        let model = root.model.clone();
        let value = self.value.clone();
        let label = self.label.clone();
        let disabled = self.disabled || root.disabled;
        let tab_stop = self.tab_stop;
        let index = self.index;
        let set_size = self.set_size;
        let inherited_invalid = props.a11y.invalid;

        cx.pressable_with_id_props(move |cx, st, id| {
            cx.key_add_on_key_down_for(
                id,
                crate::primitives::keyboard::consume_enter_key_handler(),
            );

            cx.pressable_set_option_arc_str(&model, value.clone());

            let selected_value: Option<Arc<str>> = cx.watch_model(&model).cloned().flatten();
            let checked = selected_value.as_deref() == Some(value.as_ref());

            props.enabled = !disabled;
            props.focusable = (!disabled) && (tab_stop || st.focused);
            let mut a11y = radio_button_a11y(label.clone(), checked);
            if let (Some(index), Some(set_size)) = (index, set_size)
                && let Ok(count) = usize::try_from(set_size)
            {
                a11y = a11y.with_collection_position(index, count);
            }
            a11y.invalid = inherited_invalid;
            props.a11y = a11y;

            let children = f(cx, st, id, checked, &mut props);
            (props, children)
        })
    }
}
