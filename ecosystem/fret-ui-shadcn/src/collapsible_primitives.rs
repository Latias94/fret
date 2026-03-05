//! shadcn/ui v4 `Collapsible` primitives (Radix-aligned composition surface).
//!
//! Upstream reference:
//! - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/collapsible.tsx`

use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ElementKind, LayoutStyle, PressableProps, SemanticsDecoration, SpacerProps,
};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::collapsible as radix_collapsible;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};

use crate::layout as shadcn_layout;

fn hidden_element<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.spacer(SpacerProps {
        layout: LayoutStyle::default(),
        min: Px(0.0),
    })
}

fn apply_disabled_to_trigger(mut trigger: AnyElement, disabled: bool) -> AnyElement {
    if !disabled {
        return trigger;
    }

    trigger.children = trigger
        .children
        .into_iter()
        .map(|child| apply_disabled_to_trigger(child, disabled))
        .collect();

    match &mut trigger.kind {
        ElementKind::Pressable(props) => {
            props.enabled = false;
            props.focusable = false;
        }
        ElementKind::Semantics(props) => {
            props.disabled = true;
            props.focusable = false;
        }
        _ => {}
    }

    trigger
}

fn toggle_on_activate(open: Model<bool>) -> OnActivate {
    Arc::new(move |host, _cx, _reason| {
        let _ = host.models_mut().update(&open, |v| *v = !*v);
    })
}

#[derive(Clone)]
struct CollapsibleScope {
    open: Model<bool>,
    disabled: bool,
    content_id: fret_ui::elements::GlobalElementId,
}

#[derive(Default)]
struct CollapsibleScopeState {
    scope: Option<CollapsibleScope>,
}

fn collapsible_scope_inherited<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<CollapsibleScope> {
    cx.inherited_state_where::<CollapsibleScopeState>(|st| st.scope.is_some())
        .and_then(|st| st.scope.clone())
}

#[track_caller]
fn with_collapsible_scope_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    scope: CollapsibleScope,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(CollapsibleScopeState::default, |st| st.scope.take());
    cx.with_state(CollapsibleScopeState::default, |st| {
        st.scope = Some(scope);
    });
    let out = f(cx);
    cx.with_state(CollapsibleScopeState::default, |st| {
        st.scope = prev;
    });
    out
}

/// A composable collapsible root inspired by shadcn/ui v4 and Radix `Collapsible.Root`.
pub struct Collapsible {
    open: Option<Model<bool>>,
    default_open: bool,
    disabled: bool,
    gap: Space,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Collapsible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Collapsible")
            .field("open", &self.open.is_some())
            .field("default_open", &self.default_open)
            .field("disabled", &self.disabled)
            .field("gap", &self.gap)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Collapsible {
    pub fn new() -> Self {
        Self {
            open: None,
            default_open: false,
            disabled: false,
            gap: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Sets the controlled `open` model (Radix `open`).
    pub fn open(mut self, open: Model<bool>) -> Self {
        self.open = Some(open);
        self
    }

    /// Sets the uncontrolled initial open value (Radix `defaultOpen`).
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Vertical spacing between root children.
    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = fret_ui::Theme::global(&*cx.app).snapshot();

            let open_root = radix_collapsible::CollapsibleRoot::new()
                .open(self.open)
                .default_open(self.default_open);
            let open = open_root.use_open_model(cx).model();
            let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);

            let content_id = cx.keyed("collapsible-content", |cx| cx.root_id());
            let scope = CollapsibleScope {
                open,
                disabled: self.disabled,
                content_id,
            };

            let props = decl_style::container_props(
                &theme,
                self.chrome,
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .merge(self.layout),
            );
            let gap = self.gap;

            let root = with_collapsible_scope_provider(cx, scope, |cx| {
                let children = children(cx);
                shadcn_layout::container_vstack_gap(cx, props, gap, children)
            });

            root.attach_semantics(SemanticsDecoration {
                role: Some(SemanticsRole::Generic),
                disabled: Some(self.disabled),
                expanded: Some(is_open),
                ..Default::default()
            })
        })
    }
}

/// A collapsible trigger inspired by Radix `CollapsibleTrigger`.
pub struct CollapsibleTrigger {
    children: Vec<AnyElement>,
    as_child: bool,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
}

impl std::fmt::Debug for CollapsibleTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollapsibleTrigger")
            .field("children_len", &self.children.len())
            .field("as_child", &self.as_child)
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .finish()
    }
}

impl CollapsibleTrigger {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            as_child: false,
            disabled: false,
            a11y_label: None,
        }
    }

    /// When `true`, reuses the single child element as the trigger (Radix `asChild`).
    pub fn as_child(mut self, as_child: bool) -> Self {
        self.as_child = as_child;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A11y label used when `as_child == false` (wrapper pressable path).
    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(scope) = collapsible_scope_inherited(cx) else {
            return hidden_element(cx);
        };

        let open = scope.open;
        let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);
        let disabled = self.disabled || scope.disabled;

        if self.as_child && self.children.len() == 1 {
            let mut child = self
                .children
                .into_iter()
                .next()
                .expect("children len checked above");

            if !disabled {
                cx.pressable_add_on_activate_for(child.id, toggle_on_activate(open.clone()));
            }

            child = radix_collapsible::apply_collapsible_trigger_controls_expanded(
                child,
                scope.content_id,
                is_open,
            );
            return apply_disabled_to_trigger(child, disabled);
        }

        let children = self.children;
        let a11y_label = self.a11y_label;
        let content_id = scope.content_id;

        let trigger = cx.pressable(
            PressableProps {
                enabled: !disabled,
                a11y: radix_collapsible::collapsible_trigger_a11y(a11y_label, is_open),
                ..Default::default()
            },
            move |cx, _state| {
                if !disabled {
                    cx.pressable_toggle_bool(&open);
                }
                children
            },
        );

        radix_collapsible::apply_collapsible_trigger_controls_expanded(trigger, content_id, is_open)
    }
}

/// A collapsible content block inspired by Radix `CollapsibleContent`.
pub struct CollapsibleContent {
    children: Vec<AnyElement>,
    force_mount: bool,
    gap: Space,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for CollapsibleContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollapsibleContent")
            .field("children_len", &self.children.len())
            .field("force_mount", &self.force_mount)
            .field("gap", &self.gap)
            .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl CollapsibleContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            force_mount: false,
            gap: Space::N0,
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// When `true`, the content subtree stays mounted while closed (Radix `forceMount`).
    pub fn force_mount(mut self, force_mount: bool) -> Self {
        self.force_mount = force_mount;
        self
    }

    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(scope) = collapsible_scope_inherited(cx) else {
            return hidden_element(cx);
        };

        let open = scope.open;
        let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);
        if !is_open && !self.force_mount {
            return hidden_element(cx);
        }

        let gap = self.gap;
        let chrome = self.chrome;
        let layout = self.layout;
        let children = self.children;
        let force_mount = self.force_mount;
        let test_id = self.test_id;

        let mut el = cx.keyed("collapsible-content", move |cx| {
            let theme = fret_ui::Theme::global(&*cx.app).snapshot();
            let mut wrapper_layout = LayoutRefinement::default().w_full().min_w_0().merge(layout);
            if force_mount && !is_open {
                wrapper_layout = wrapper_layout.h_px(Px(0.0)).overflow_hidden();
            }
            let props = decl_style::container_props(&theme, chrome, wrapper_layout);

            let inner = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(gap)
                    .items_stretch()
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                move |_cx| children,
            );
            let children = if force_mount && !is_open {
                vec![cx.interactivity_gate(true, false, move |_cx| vec![inner])]
            } else {
                vec![inner]
            };

            let id = cx.root_id();
            AnyElement::new(id, ElementKind::Container(props), children)
        });

        if let Some(test_id) = test_id {
            el = el.test_id(test_id);
        }
        el
    }
}
