//! shadcn/ui `Collapsible` (headless).

use std::sync::Arc;

use fret_components_ui::LayoutRefinement;
use fret_components_ui::declarative::action_hooks::ActionHooksExt as _;
use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::{AnyElement, PressableA11y, PressableProps, SemanticsProps, StackProps};
use fret_ui::{ElementCx, UiHost};

#[derive(Clone)]
pub struct Collapsible {
    open: Model<bool>,
    disabled: bool,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Collapsible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Collapsible")
            .field("open", &"<model>")
            .field("disabled", &self.disabled)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Collapsible {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            disabled: false,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementCx<'_, H>,
        trigger: impl FnOnce(&mut ElementCx<'_, H>, bool) -> AnyElement,
        content: impl FnOnce(&mut ElementCx<'_, H>) -> AnyElement,
    ) -> AnyElement {
        cx.scope(|cx| {
            cx.observe_model(&self.open, Invalidation::Layout);
            let is_open = cx.app.models().get_copied(&self.open).unwrap_or(false);

            let trigger = trigger(cx, is_open);
            let content = is_open.then(|| content(cx));
            let layout = self.layout;

            let stack = cx.stack_props(
                StackProps {
                    layout: fret_components_ui::declarative::style::layout_style(
                        fret_ui::Theme::global(&*cx.app),
                        layout,
                    ),
                },
                move |_cx| {
                    let mut children = Vec::new();
                    children.push(trigger);
                    if let Some(content) = content {
                        children.push(content);
                    }
                    children
                },
            );

            cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Generic,
                    disabled: self.disabled,
                    expanded: Some(is_open),
                    ..Default::default()
                },
                move |_cx| vec![stack],
            )
        })
    }
}

#[derive(Clone)]
pub struct CollapsibleTrigger {
    open: Model<bool>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for CollapsibleTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollapsibleTrigger")
            .field("open", &"<model>")
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl CollapsibleTrigger {
    pub fn new(open: Model<bool>, children: Vec<AnyElement>) -> Self {
        Self {
            open,
            disabled: false,
            a11y_label: None,
            children,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>, is_open: bool) -> AnyElement {
        let open = self.open;
        let disabled = self.disabled;
        let children = self.children;
        let a11y_label = self.a11y_label;

        cx.pressable(
            PressableProps {
                enabled: !disabled,
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    label: a11y_label,
                    expanded: Some(is_open),
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, _state| {
                cx.pressable_toggle_bool(&open);
                children
            },
        )
    }
}

#[derive(Clone)]
pub struct CollapsibleContent {
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for CollapsibleContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollapsibleContent")
            .field("layout", &self.layout)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl CollapsibleContent {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            layout: LayoutRefinement::default(),
            children,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let layout = self.layout;
        let children = self.children;

        cx.stack_props(
            StackProps {
                layout: fret_components_ui::declarative::style::layout_style(
                    fret_ui::Theme::global(&*cx.app),
                    layout,
                ),
            },
            move |_cx| children,
        )
    }
}

pub fn collapsible<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    open: Model<bool>,
    trigger: impl FnOnce(&mut ElementCx<'_, H>, bool) -> AnyElement,
    content: impl FnOnce(&mut ElementCx<'_, H>) -> AnyElement,
) -> AnyElement {
    Collapsible::new(open).into_element(cx, trigger, content)
}
