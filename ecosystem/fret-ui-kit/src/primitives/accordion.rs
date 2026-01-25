//! Accordion primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing accordion behavior in recipes.
//! It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/accordion/src/accordion.tsx`

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, LayoutStyle, PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps,
    SemanticsProps, StackProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::declarative::action_hooks::ActionHooksExt as _;
use crate::primitives::trigger_a11y;
use crate::primitives::{direction as direction_prim, direction::LayoutDirection};

/// Matches Radix Accordion `type` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccordionKind {
    Single,
    Multiple,
}

/// Matches Radix Accordion `orientation` outcome: vertical (default) vs horizontal layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccordionOrientation {
    #[default]
    Vertical,
    Horizontal,
}

/// A11y metadata for an accordion trigger.
pub fn accordion_trigger_a11y(label: Arc<str>, open: bool) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::Button),
        label: Some(label),
        expanded: Some(open),
        ..Default::default()
    }
}

/// Stamps Radix-like trigger relationships:
/// - `controls_element` mirrors `aria-controls` (by element id).
///
/// In Radix Accordion, the trigger points at its content by id. In Fret we model this via a
/// portable element-id relationship that resolves into `SemanticsNode.controls` when the content
/// is mounted.
pub fn apply_accordion_trigger_controls(
    trigger: AnyElement,
    content_element: GlobalElementId,
) -> AnyElement {
    trigger_a11y::apply_trigger_controls(trigger, Some(content_element))
}

/// Derive the "tab stop" index for a single-select accordion:
/// the open enabled item, or the first enabled item.
pub fn tab_stop_index_single(
    values: &[Arc<str>],
    open: Option<&str>,
    disabled: &[bool],
) -> Option<usize> {
    if let Some(open) = open {
        if let Some(active) =
            crate::headless::roving_focus::active_index_from_str_keys(values, Some(open), disabled)
        {
            return Some(active);
        }
    }
    crate::headless::roving_focus::first_enabled(disabled)
}

/// Derive the "tab stop" index for a multi-select accordion:
/// the first open+enabled item, or the first enabled item.
pub fn tab_stop_index_multiple(
    values: &[Arc<str>],
    open: &[Arc<str>],
    disabled: &[bool],
) -> Option<usize> {
    let first_open_enabled = values.iter().enumerate().find_map(|(idx, v)| {
        let enabled = !disabled.get(idx).copied().unwrap_or(true);
        let is_open = open.iter().any(|s| s.as_ref() == v.as_ref());
        (enabled && is_open).then_some(idx)
    });
    first_open_enabled.or_else(|| crate::headless::roving_focus::first_enabled(disabled))
}

/// Returns a single-select open item model that behaves like Radix `useControllableState` (`value` /
/// `defaultValue`).
pub fn accordion_use_single_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Option<Arc<str>>>>,
    default_value: impl FnOnce() -> Option<Arc<str>>,
) -> crate::primitives::controllable_state::ControllableModel<Option<Arc<str>>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

/// Returns a multi-select open items model that behaves like Radix `useControllableState` (`value` /
/// `defaultValue`).
pub fn accordion_use_multiple_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Vec<Arc<str>>>>,
    default_value: impl FnOnce() -> Vec<Arc<str>>,
) -> crate::primitives::controllable_state::ControllableModel<Vec<Arc<str>>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

/// A composable, Radix-shaped accordion configuration surface (`AccordionRoot` / `AccordionItem` /
/// `AccordionTrigger` / `AccordionContent`).
///
/// Unlike Radix React, Fret does not use context objects; the "composition" surface is expressed as
/// small Rust builders that thread the shared models and option values through closures.
#[derive(Debug, Clone)]
pub struct AccordionRoot {
    kind: AccordionKind,
    single_model: Option<Model<Option<Arc<str>>>>,
    multiple_model: Option<Model<Vec<Arc<str>>>>,
    collapsible: bool,
    disabled: bool,
    loop_navigation: bool,
    orientation: AccordionOrientation,
    dir: Option<LayoutDirection>,
}

impl AccordionRoot {
    pub fn single(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            kind: AccordionKind::Single,
            single_model: Some(model),
            multiple_model: None,
            collapsible: false,
            disabled: false,
            loop_navigation: true,
            orientation: AccordionOrientation::default(),
            dir: None,
        }
    }

    /// Creates an accordion root with a controlled/uncontrolled selection model (Radix `value` /
    /// `defaultValue`).
    ///
    /// Notes:
    /// - The internal model (uncontrolled mode) is stored in element state at the call site.
    /// - Call this from a stable subtree (key the root node if you need state to survive reordering).
    pub fn single_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        controlled: Option<Model<Option<Arc<str>>>>,
        default_value: impl FnOnce() -> Option<Arc<str>>,
    ) -> Self {
        let model = accordion_use_single_model(cx, controlled, default_value).model();
        Self::single(model)
    }

    pub fn multiple(model: Model<Vec<Arc<str>>>) -> Self {
        Self {
            kind: AccordionKind::Multiple,
            single_model: None,
            multiple_model: Some(model),
            collapsible: false,
            disabled: false,
            loop_navigation: true,
            orientation: AccordionOrientation::default(),
            dir: None,
        }
    }

    /// Creates an accordion root with a controlled/uncontrolled selection model (Radix `value` /
    /// `defaultValue`).
    ///
    /// Notes:
    /// - The internal model (uncontrolled mode) is stored in element state at the call site.
    /// - Call this from a stable subtree (key the root node if you need state to survive reordering).
    pub fn multiple_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        controlled: Option<Model<Vec<Arc<str>>>>,
        default_value: impl FnOnce() -> Vec<Arc<str>>,
    ) -> Self {
        let model = accordion_use_multiple_model(cx, controlled, default_value).model();
        Self::multiple(model)
    }

    pub fn kind(&self) -> AccordionKind {
        self.kind
    }

    /// In Radix single mode, `collapsible` controls whether the open item can be closed.
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    /// Controls the keyboard navigation axis for the accordion triggers.
    ///
    /// This mirrors Radix `orientation` and should match the visual layout direction.
    pub fn orientation(mut self, orientation: AccordionOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Overrides the local direction for horizontal keyboard navigation.
    ///
    /// Mirrors Radix `dir` behavior: in RTL, Left/Right arrow semantics are flipped.
    pub fn dir(mut self, dir: Option<LayoutDirection>) -> Self {
        self.dir = dir;
        self
    }

    pub fn list(self, values: Arc<[Arc<str>]>, disabled: Arc<[bool]>) -> AccordionList {
        AccordionList::new(self, values, disabled)
    }

    pub fn item(&self, value: impl Into<Arc<str>>) -> AccordionItem {
        AccordionItem::new(value)
    }

    pub fn trigger(&self, value: impl Into<Arc<str>>) -> AccordionTrigger {
        AccordionTrigger::new(value)
    }

    pub fn content(&self, value: impl Into<Arc<str>>) -> AccordionContent {
        AccordionContent::new(value)
    }

    pub fn is_item_open<H: UiHost>(&self, cx: &mut ElementContext<'_, H>, value: &str) -> bool {
        match self.kind {
            AccordionKind::Single => cx
                .watch_model(
                    self.single_model
                        .as_ref()
                        .expect("AccordionRoot single model"),
                )
                .layout()
                .cloned()
                .flatten()
                .as_deref()
                .is_some_and(|v| v == value),
            AccordionKind::Multiple => cx
                .watch_model(
                    self.multiple_model
                        .as_ref()
                        .expect("AccordionRoot multiple model"),
                )
                .layout()
                .cloned()
                .unwrap_or_default()
                .iter()
                .any(|v| v.as_ref() == value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{ElementKind, PressableProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn accordion_use_single_model_prefers_controlled_and_does_not_call_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(Some(Arc::from("a")));
        let called = Cell::new(0);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let out = accordion_use_single_model(cx, Some(controlled.clone()), || {
                called.set(called.get() + 1);
                None
            });
            assert!(out.is_controlled());
            assert_eq!(out.model(), controlled);
        });

        assert_eq!(called.get(), 0);
    }

    #[test]
    fn accordion_use_multiple_model_prefers_controlled_and_does_not_call_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(vec![Arc::from("a")]);
        let called = Cell::new(0);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let out = accordion_use_multiple_model(cx, Some(controlled.clone()), || {
                called.set(called.get() + 1);
                Vec::new()
            });
            assert!(out.is_controlled());
            assert_eq!(out.model(), controlled);
        });

        assert_eq!(called.get(), 0);
    }

    #[test]
    fn apply_accordion_trigger_controls_sets_controls_on_pressable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    layout: LayoutStyle::default(),
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );
            let content = GlobalElementId(0xbeef);
            let trigger = apply_accordion_trigger_controls(trigger, content);
            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable");
            };
            assert_eq!(a11y.controls_element, Some(content.0));
        });
    }

    #[test]
    fn accordion_list_orientation_vertical_sets_roving_axis_vertical() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let open: Model<Option<Arc<str>>> = app.models_mut().insert(None);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let root = AccordionRoot::single(open).orientation(AccordionOrientation::Vertical);
            let values: Arc<[Arc<str>]> = Arc::from(vec![Arc::from("a")].into_boxed_slice());
            let disabled: Arc<[bool]> = Arc::from(vec![false].into_boxed_slice());
            let list = root.list(values, disabled);
            let el = list.into_element(cx, RovingFlexProps::default(), |_cx| Vec::new());

            let ElementKind::Semantics(_) = &el.kind else {
                panic!("expected semantics wrapper");
            };
            let child = el.children.first().expect("semantics child");
            let ElementKind::RovingFlex(props) = &child.kind else {
                panic!("expected roving flex child");
            };
            assert_eq!(props.flex.direction, fret_core::Axis::Vertical);
        });
    }

    #[test]
    fn accordion_list_orientation_horizontal_sets_roving_axis_horizontal() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let open: Model<Option<Arc<str>>> = app.models_mut().insert(None);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let root = AccordionRoot::single(open).orientation(AccordionOrientation::Horizontal);
            let values: Arc<[Arc<str>]> = Arc::from(vec![Arc::from("a")].into_boxed_slice());
            let disabled: Arc<[bool]> = Arc::from(vec![false].into_boxed_slice());
            let list = root.list(values, disabled);
            let el = list.into_element(cx, RovingFlexProps::default(), |_cx| Vec::new());

            let ElementKind::Semantics(_) = &el.kind else {
                panic!("expected semantics wrapper");
            };
            let child = el.children.first().expect("semantics child");
            let ElementKind::RovingFlex(props) = &child.kind else {
                panic!("expected roving flex child");
            };
            assert_eq!(props.flex.direction, fret_core::Axis::Horizontal);
        });
    }
}

#[derive(Debug, Clone)]
pub struct AccordionList {
    root: AccordionRoot,
    values: Arc<[Arc<str>]>,
    disabled: Arc<[bool]>,
    layout: LayoutStyle,
}

impl AccordionList {
    pub fn new(root: AccordionRoot, values: Arc<[Arc<str>]>, disabled: Arc<[bool]>) -> Self {
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

    pub fn tab_stop_index<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Option<usize> {
        let disabled: Vec<bool> = self.disabled.iter().copied().collect();
        match self.root.kind {
            AccordionKind::Single => {
                let open = cx
                    .watch_model(
                        self.root
                            .single_model
                            .as_ref()
                            .expect("AccordionRoot single model"),
                    )
                    .layout()
                    .cloned()
                    .flatten();
                tab_stop_index_single(&self.values, open.as_deref(), &disabled)
            }
            AccordionKind::Multiple => {
                let open = cx
                    .watch_model(
                        self.root
                            .multiple_model
                            .as_ref()
                            .expect("AccordionRoot multiple model"),
                    )
                    .layout()
                    .cloned()
                    .unwrap_or_default();
                tab_stop_index_multiple(&self.values, &open, &disabled)
            }
        }
    }

    /// Renders an accordion list semantics root containing a roving-focus group.
    ///
    /// Notes:
    /// - This does not apply any visual skin. Pass `flex` / `layout` via `RovingFlexProps`.
    /// - Accordion selection is activation-driven: focus movement does not toggle open state.
    #[track_caller]
    pub fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        mut props: RovingFlexProps,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        props.flex.direction = match self.root.orientation {
            AccordionOrientation::Vertical => fret_core::Axis::Vertical,
            AccordionOrientation::Horizontal => fret_core::Axis::Horizontal,
        };
        props.roving = RovingFocusProps {
            enabled: props.roving.enabled && !self.root.disabled,
            wrap: self.root.loop_navigation,
            disabled: self.disabled.clone(),
        };

        let layout = self.layout;
        let dir = self.root.dir;
        let root_disabled = self.root.disabled;

        if let Some(dir) = dir {
            direction_prim::with_direction_provider(cx, dir, move |cx| {
                cx.semantics(
                    SemanticsProps {
                        layout,
                        role: SemanticsRole::List,
                        disabled: root_disabled,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.roving_flex(props, move |cx| {
                            cx.roving_nav_apg();
                            f(cx)
                        })]
                    },
                )
            })
        } else {
            cx.semantics(
                SemanticsProps {
                    layout,
                    role: SemanticsRole::List,
                    disabled: root_disabled,
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.roving_flex(props, move |cx| {
                        cx.roving_nav_apg();
                        f(cx)
                    })]
                },
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccordionItem {
    value: Arc<str>,
}

impl AccordionItem {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn value(&self) -> Arc<str> {
        self.value.clone()
    }
}

#[derive(Debug, Clone)]
pub struct AccordionTrigger {
    value: Arc<str>,
    label: Option<Arc<str>>,
    disabled: bool,
    tab_stop: bool,
}

impl AccordionTrigger {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: None,
            disabled: false,
            tab_stop: false,
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

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        root: &AccordionRoot,
        mut props: PressableProps,
        f: impl FnOnce(&mut ElementContext<'_, H>, bool) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        let value = self.value.clone();
        let label = self.label.clone().unwrap_or_else(|| value.clone());
        let disabled = self.disabled || root.disabled;
        let tab_stop = self.tab_stop;

        cx.pressable_with_id_props(move |cx, st, _id| {
            match root.kind {
                AccordionKind::Single => {
                    let model = root
                        .single_model
                        .as_ref()
                        .expect("AccordionRoot single model")
                        .clone();
                    let value = value.clone();
                    let collapsible = root.collapsible;
                    cx.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
                        let value = value.clone();
                        let _ = host.models_mut().update(&model, |open| {
                            let is_same = open.as_deref().is_some_and(|cur| cur == value.as_ref());
                            if is_same {
                                if collapsible {
                                    *open = None;
                                }
                            } else {
                                *open = Some(value);
                            }
                        });
                    }));
                }
                AccordionKind::Multiple => {
                    let model = root
                        .multiple_model
                        .as_ref()
                        .expect("AccordionRoot multiple model")
                        .clone();
                    cx.pressable_toggle_vec_arc_str(&model, value.clone());
                }
            }

            let open = root.is_item_open(cx, value.as_ref());

            props.enabled = !disabled;
            props.focusable = (!disabled) && (tab_stop || st.focused);
            props.a11y = accordion_trigger_a11y(label.clone(), open);

            (props, f(cx, open))
        })
    }
}

#[derive(Debug, Clone)]
pub struct AccordionContent {
    value: Arc<str>,
    force_mount: bool,
}

impl AccordionContent {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            force_mount: false,
        }
    }

    pub fn force_mount(mut self, force_mount: bool) -> Self {
        self.force_mount = force_mount;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        root: &AccordionRoot,
        f: impl FnOnce(&mut ElementContext<'_, H>, bool) -> I,
    ) -> Option<AnyElement>
    where
        I: IntoIterator<Item = AnyElement>,
    {
        let open = root.is_item_open(cx, self.value.as_ref());
        if !open && !self.force_mount {
            return None;
        }
        Some(cx.stack_props(
            StackProps {
                layout: LayoutStyle::default(),
            },
            move |cx| f(cx, open),
        ))
    }
}
