//! shadcn/ui `button-group` recipe surface.
//!
//! Upstream reference (v4):
//! - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/button-group.tsx`
//!
//! This is a layout + chrome composition helper:
//! - nested groups use `gap-2` (8px) between groups,
//! - leaf groups merge button borders and corner radii (`border-l-0`, `rounded-*-none`).

use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_ui::element::{AnyElement, FlexProps, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::style as decl_style;

use crate::Button;

#[derive(Debug, Clone)]
pub enum ButtonGroupItem {
    Button(Button),
    Group(Box<ButtonGroup>),
}

impl From<Button> for ButtonGroupItem {
    fn from(value: Button) -> Self {
        Self::Button(value)
    }
}

impl From<ButtonGroup> for ButtonGroupItem {
    fn from(value: ButtonGroup) -> Self {
        Self::Group(Box::new(value))
    }
}

#[derive(Debug, Clone)]
pub struct ButtonGroup {
    items: Vec<ButtonGroupItem>,
    layout: LayoutRefinement,
    a11y_label: Option<Arc<str>>,
}

impl ButtonGroup {
    pub fn new(items: Vec<ButtonGroupItem>) -> Self {
        Self {
            items,
            layout: LayoutRefinement::default(),
            a11y_label: None,
        }
    }

    pub fn items(mut self, items: Vec<ButtonGroupItem>) -> Self {
        self.items = items;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let has_nested_group = self
            .items
            .iter()
            .any(|i| matches!(i, ButtonGroupItem::Group(_)));
        let is_leaf_group = !has_nested_group
            && self
                .items
                .iter()
                .all(|i| matches!(i, ButtonGroupItem::Button(_)));

        let gap = if has_nested_group { Px(8.0) } else { Px(0.0) };

        let layout = decl_style::layout_style(&theme, self.layout);
        let props = FlexProps {
            layout,
            direction: fret_core::Axis::Horizontal,
            gap,
            padding: Edges::all(Px(0.0)),
            justify: fret_ui::element::MainAlign::Start,
            align: fret_ui::element::CrossAlign::Stretch,
            wrap: false,
        };

        let items = self.items;
        let a11y_label = self.a11y_label.clone();

        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                label: a11y_label,
                ..Default::default()
            },
            move |cx| {
                vec![cx.flex(props, move |cx| {
                    if is_leaf_group {
                        let radius = Theme::global(&*cx.app).metric_required("metric.radius.md");
                        let border_w = Px(1.0);
                        let len = items.len();

                        items
                            .into_iter()
                            .enumerate()
                            .map(|(idx, item)| match item {
                                ButtonGroupItem::Button(button) => {
                                    let is_first = idx == 0;
                                    let is_last = idx + 1 == len;

                                    let corners = Corners {
                                        top_left: if is_first { radius } else { Px(0.0) },
                                        bottom_left: if is_first { radius } else { Px(0.0) },
                                        top_right: if is_last { radius } else { Px(0.0) },
                                        bottom_right: if is_last { radius } else { Px(0.0) },
                                    };
                                    let border = Edges {
                                        top: border_w,
                                        right: border_w,
                                        bottom: border_w,
                                        left: if is_first { border_w } else { Px(0.0) },
                                    };

                                    button
                                        .corner_radii_override(corners)
                                        .border_override(border)
                                        .into_element(cx)
                                }
                                ButtonGroupItem::Group(group) => group.into_element(cx),
                            })
                            .collect()
                    } else {
                        items
                            .into_iter()
                            .map(|item| match item {
                                ButtonGroupItem::Button(button) => button.into_element(cx),
                                ButtonGroupItem::Group(group) => group.into_element(cx),
                            })
                            .collect()
                    }
                })]
            },
        )
    }
}
