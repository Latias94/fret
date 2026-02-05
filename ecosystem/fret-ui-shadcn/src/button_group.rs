//! shadcn/ui `button-group` recipe surface.
//!
//! Upstream reference (v4):
//! - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/button-group.tsx`
//!
//! This is a layout + chrome composition helper:
//! - nested groups use `gap-2` (8px) between groups,
//! - direct children merge borders and corner radii (`border-*-0`, `rounded-*-none`),
//! - separators participate as normal children (e.g. split buttons).

use std::sync::Arc;

use fret_core::{Axis, Corners, Edges, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, LayoutStyle, Length, SemanticsDecoration, SizeStyle,
    TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{LayoutRefinement, MetricRef, Radius, Space};

use crate::{Button, InputGroup, Separator};

#[derive(Debug, Clone, Copy, Default)]
struct BorderWidthOverride {
    top: Option<Px>,
    right: Option<Px>,
    bottom: Option<Px>,
    left: Option<Px>,
}

#[derive(Debug, Clone)]
pub struct ButtonGroupText {
    text: Arc<str>,
    layout: LayoutRefinement,
    border_width_override: BorderWidthOverride,
    corner_radii_override: Option<Corners>,
}

impl ButtonGroupText {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            layout: LayoutRefinement::default(),
            border_width_override: BorderWidthOverride::default(),
            corner_radii_override: None,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn border_left_width_override(mut self, border: Px) -> Self {
        self.border_width_override.left = Some(border);
        self
    }

    pub fn border_right_width_override(mut self, border: Px) -> Self {
        self.border_width_override.right = Some(border);
        self
    }

    pub fn border_top_width_override(mut self, border: Px) -> Self {
        self.border_width_override.top = Some(border);
        self
    }

    pub fn border_bottom_width_override(mut self, border: Px) -> Self {
        self.border_width_override.bottom = Some(border);
        self
    }

    pub fn corner_radii_override(mut self, corners: Corners) -> Self {
        self.corner_radii_override = Some(corners);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let bg = theme.color_required("muted");
        let fg = theme.color_required("foreground");
        let border_color = theme.color_required("border");
        let radius = MetricRef::radius(Radius::Md).resolve(&theme);

        let mut border = Edges::all(Px(1.0));
        if let Some(w) = self.border_width_override.top {
            border.top = w;
        }
        if let Some(w) = self.border_width_override.right {
            border.right = w;
        }
        if let Some(w) = self.border_width_override.bottom {
            border.bottom = w;
        }
        if let Some(w) = self.border_width_override.left {
            border.left = w;
        }

        let corner_radii = self
            .corner_radii_override
            .unwrap_or_else(|| Corners::all(radius));

        let layout = decl_style::layout_style(&theme, self.layout);
        let px_4 = MetricRef::space(Space::N4).resolve(&theme);
        let text_px = theme.metric_required("metric.font.size");
        let line_height = theme.metric_required("metric.font.line_height");
        let text = self.text;

        cx.container(
            ContainerProps {
                layout,
                background: Some(bg),
                shadow: Some(decl_style::shadow_xs(&theme, radius)),
                border,
                border_color: Some(border_color),
                focus_ring: None,
                focus_border_color: None,
                focus_within: false,
                corner_radii,
                ..Default::default()
            },
            move |cx| {
                let content = cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: Axis::Horizontal,
                        gap: Px(8.0),
                        padding: Edges {
                            top: Px(0.0),
                            right: px_4,
                            bottom: Px(0.0),
                            left: px_4,
                        },
                        justify: fret_ui::element::MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        vec![cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text: text.clone(),
                            style: Some(TextStyle {
                                font: FontId::default(),
                                size: text_px,
                                weight: FontWeight::MEDIUM,
                                slant: Default::default(),
                                line_height: Some(line_height),
                                letter_spacing_em: None,
                            }),
                            color: Some(fg),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        })]
                    },
                );
                vec![content]
            },
        )
    }
}

#[derive(Debug, Clone)]
pub enum ButtonGroupItem {
    Button(Button),
    Text(ButtonGroupText),
    InputGroup(InputGroup),
    Group(Box<ButtonGroup>),
    Separator(Separator),
    Element(AnyElement),
}

impl From<Button> for ButtonGroupItem {
    fn from(value: Button) -> Self {
        Self::Button(value)
    }
}

impl From<ButtonGroupText> for ButtonGroupItem {
    fn from(value: ButtonGroupText) -> Self {
        Self::Text(value)
    }
}

impl From<InputGroup> for ButtonGroupItem {
    fn from(value: InputGroup) -> Self {
        Self::InputGroup(value)
    }
}

impl From<ButtonGroup> for ButtonGroupItem {
    fn from(value: ButtonGroup) -> Self {
        Self::Group(Box::new(value))
    }
}

impl From<Separator> for ButtonGroupItem {
    fn from(value: Separator) -> Self {
        Self::Separator(value)
    }
}

impl From<AnyElement> for ButtonGroupItem {
    fn from(value: AnyElement) -> Self {
        Self::Element(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct ButtonGroup {
    items: Vec<ButtonGroupItem>,
    orientation: ButtonGroupOrientation,
    layout: LayoutRefinement,
    a11y_label: Option<Arc<str>>,
}

impl ButtonGroup {
    pub fn new(items: impl IntoIterator<Item = ButtonGroupItem>) -> Self {
        let items = items.into_iter().collect();
        Self {
            items,
            orientation: ButtonGroupOrientation::default(),
            layout: LayoutRefinement::default(),
            a11y_label: None,
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ButtonGroupItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn orientation(mut self, orientation: ButtonGroupOrientation) -> Self {
        self.orientation = orientation;
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

        let gap = if has_nested_group { Px(8.0) } else { Px(0.0) };
        let direction = match self.orientation {
            ButtonGroupOrientation::Horizontal => Axis::Horizontal,
            ButtonGroupOrientation::Vertical => Axis::Vertical,
        };

        let layout = decl_style::layout_style(&theme, self.layout);
        let props = FlexProps {
            layout,
            direction,
            gap,
            padding: Edges::all(Px(0.0)),
            justify: fret_ui::element::MainAlign::Start,
            align: fret_ui::element::CrossAlign::Stretch,
            wrap: false,
        };

        let items = self.items;
        let orientation = self.orientation;
        let a11y_label = self.a11y_label;

        let group = cx.flex(props, move |cx| {
            let radius = Theme::global(&*cx.app).metric_required("metric.radius.md");
            let len = items.len();

            items
                .into_iter()
                .enumerate()
                .map(|(idx, item)| match item {
                    ButtonGroupItem::Button(button) => {
                        let is_first = idx == 0;
                        let is_last = idx + 1 == len;

                        let corners = match orientation {
                            ButtonGroupOrientation::Horizontal => Corners {
                                top_left: if is_first { radius } else { Px(0.0) },
                                bottom_left: if is_first { radius } else { Px(0.0) },
                                top_right: if is_last { radius } else { Px(0.0) },
                                bottom_right: if is_last { radius } else { Px(0.0) },
                            },
                            ButtonGroupOrientation::Vertical => Corners {
                                top_left: if is_first { radius } else { Px(0.0) },
                                top_right: if is_first { radius } else { Px(0.0) },
                                bottom_right: if is_last { radius } else { Px(0.0) },
                                bottom_left: if is_last { radius } else { Px(0.0) },
                            },
                        };

                        let button = match orientation {
                            ButtonGroupOrientation::Horizontal => {
                                if is_first {
                                    button
                                } else {
                                    button.border_left_width_override(Px(0.0))
                                }
                            }
                            ButtonGroupOrientation::Vertical => {
                                if is_first {
                                    button
                                } else {
                                    button.border_top_width_override(Px(0.0))
                                }
                            }
                        };

                        button.corner_radii_override(corners).into_element(cx)
                    }
                    ButtonGroupItem::Text(text) => {
                        let is_first = idx == 0;
                        let is_last = idx + 1 == len;

                        let corners = match orientation {
                            ButtonGroupOrientation::Horizontal => Corners {
                                top_left: if is_first { radius } else { Px(0.0) },
                                bottom_left: if is_first { radius } else { Px(0.0) },
                                top_right: if is_last { radius } else { Px(0.0) },
                                bottom_right: if is_last { radius } else { Px(0.0) },
                            },
                            ButtonGroupOrientation::Vertical => Corners {
                                top_left: if is_first { radius } else { Px(0.0) },
                                top_right: if is_first { radius } else { Px(0.0) },
                                bottom_right: if is_last { radius } else { Px(0.0) },
                                bottom_left: if is_last { radius } else { Px(0.0) },
                            },
                        };

                        let text = match orientation {
                            ButtonGroupOrientation::Horizontal => {
                                if is_first {
                                    text
                                } else {
                                    text.border_left_width_override(Px(0.0))
                                }
                            }
                            ButtonGroupOrientation::Vertical => {
                                if is_first {
                                    text
                                } else {
                                    text.border_top_width_override(Px(0.0))
                                }
                            }
                        };

                        text.corner_radii_override(corners).into_element(cx)
                    }
                    ButtonGroupItem::InputGroup(group) => {
                        let is_first = idx == 0;
                        let is_last = idx + 1 == len;

                        let corners = match orientation {
                            ButtonGroupOrientation::Horizontal => Corners {
                                top_left: if is_first { radius } else { Px(0.0) },
                                bottom_left: if is_first { radius } else { Px(0.0) },
                                top_right: if is_last { radius } else { Px(0.0) },
                                bottom_right: if is_last { radius } else { Px(0.0) },
                            },
                            ButtonGroupOrientation::Vertical => Corners {
                                top_left: if is_first { radius } else { Px(0.0) },
                                top_right: if is_first { radius } else { Px(0.0) },
                                bottom_right: if is_last { radius } else { Px(0.0) },
                                bottom_left: if is_last { radius } else { Px(0.0) },
                            },
                        };

                        let group = match orientation {
                            ButtonGroupOrientation::Horizontal => {
                                if is_first {
                                    group
                                } else {
                                    group.border_left_width_override(Px(0.0))
                                }
                            }
                            ButtonGroupOrientation::Vertical => {
                                if is_first {
                                    group
                                } else {
                                    group.border_top_width_override(Px(0.0))
                                }
                            }
                        };

                        let group = match orientation {
                            ButtonGroupOrientation::Horizontal => {
                                group.refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                            }
                            ButtonGroupOrientation::Vertical => group,
                        };

                        group.corner_radii_override(corners).into_element(cx)
                    }
                    ButtonGroupItem::Group(group) => group.into_element(cx),
                    ButtonGroupItem::Separator(separator) => {
                        separator.flex_stretch_cross_axis(true).into_element(cx)
                    }
                    ButtonGroupItem::Element(element) => element,
                })
                .collect::<Vec<_>>()
        });

        let mut decoration = SemanticsDecoration::default().role(fret_core::SemanticsRole::Group);
        if let Some(a11y_label) = a11y_label {
            decoration = decoration.label(a11y_label);
        }
        group.attach_semantics(decoration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::ElementKind;

    #[test]
    fn button_group_stamps_role_without_layout_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ButtonGroup::new([ButtonGroupItem::from(cx.text("A"))])
                .a11y_label("Actions")
                .into_element(cx)
        });

        assert!(
            !matches!(element.kind, ElementKind::Semantics(_)),
            "expected ButtonGroup to avoid `Semantics` wrappers; use `attach_semantics` instead"
        );
        assert_eq!(
            element.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(fret_core::SemanticsRole::Group)
        );
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.label.as_deref()),
            Some("Actions")
        );
    }
}
