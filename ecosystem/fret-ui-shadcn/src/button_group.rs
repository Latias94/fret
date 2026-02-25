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
    AnyElement, FlexProps, LayoutStyle, Length, SemanticsDecoration, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography::{self, TextIntent};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};

use crate::{Button, InputGroup, Separator, SeparatorOrientation};

#[derive(Debug, Clone, Copy, Default)]
struct BorderWidthOverride {
    top: Option<Px>,
    right: Option<Px>,
    bottom: Option<Px>,
    left: Option<Px>,
}

#[derive(Debug)]
pub struct ButtonGroupText {
    content: ButtonGroupTextContent,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
    border_width_override: BorderWidthOverride,
    corner_radii_override: Option<Corners>,
    test_id: Option<Arc<str>>,
}

#[derive(Debug)]
enum ButtonGroupTextContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

impl ButtonGroupText {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: ButtonGroupTextContent::Text(text.into()),
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
            border_width_override: BorderWidthOverride::default(),
            corner_radii_override: None,
            test_id: None,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.content = ButtonGroupTextContent::Children(children.into_iter().collect());
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

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let bg = theme.color_token("muted");
        let fg = theme.color_token("foreground");
        let border_color = theme.color_token("border");
        let radius = MetricRef::radius(Radius::Md).resolve(&theme);

        let base_chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border_color))
            .shadow_xs()
            .radius(MetricRef::radius(Radius::Md))
            .text_color(ColorRef::Color(fg));
        let chrome = base_chrome.merge(self.chrome);
        let text_color = chrome
            .text_color
            .clone()
            .unwrap_or_else(|| ColorRef::Color(theme.color_token("foreground")))
            .resolve(&theme);

        let corner_radii = self
            .corner_radii_override
            .unwrap_or_else(|| Corners::all(radius));

        let content = self.content;
        let test_id = self.test_id;

        let mut props = decl_style::container_props(&theme, chrome, self.layout);
        props.corner_radii = corner_radii;
        props.snap_to_device_pixels = true;

        if let Some(w) = self.border_width_override.top {
            props.border.top = w;
        }
        if let Some(w) = self.border_width_override.right {
            props.border.right = w;
        }
        if let Some(w) = self.border_width_override.bottom {
            props.border.bottom = w;
        }
        if let Some(w) = self.border_width_override.left {
            props.border.left = w;
        }

        let px_4 = MetricRef::space(Space::N4).resolve(&theme);
        let text_px = theme.metric_token("metric.font.size");
        let line_height = theme.metric_token("metric.font.line_height");

        let mut el = cx.container(props, move |cx| {
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
                    gap: Px(8.0).into(),
                    padding: Edges {
                        top: Px(0.0),
                        right: px_4,
                        bottom: Px(0.0),
                        left: px_4,
                    }
                    .into(),
                    justify: fret_ui::element::MainAlign::Start,
                    align: fret_ui::element::CrossAlign::Center,
                    wrap: false,
                },
                move |cx| match content {
                    ButtonGroupTextContent::Text(text) => {
                        let style = typography::with_intent(
                            TextStyle {
                                font: FontId::default(),
                                size: text_px,
                                weight: FontWeight::MEDIUM,
                                line_height: Some(line_height),
                                ..Default::default()
                            },
                            TextIntent::Control,
                        );

                        vec![cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text,
                            style: Some(style),
                            color: Some(text_color),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                            align: fret_core::TextAlign::Start,
                            ink_overflow: Default::default(),
                        })]
                    }
                    ButtonGroupTextContent::Children(children) => children,
                },
            );
            vec![content]
        });

        if let Some(test_id) = test_id {
            el = el.attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id(test_id),
            );
        }

        el
    }
}

#[derive(Debug, Clone)]
pub struct ButtonGroupSeparator {
    orientation: SeparatorOrientation,
    thickness: Option<Px>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl ButtonGroupSeparator {
    pub fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::Vertical,
            thickness: None,
            layout: LayoutRefinement::default(),
            test_id: None,
        }
    }

    pub fn orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn thickness(mut self, thickness: Px) -> Self {
        self.thickness = Some(thickness);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (bg, thickness, mut layout) = {
            let theme = Theme::global(&*cx.app);

            let bg = theme
                .color_by_key("input")
                .unwrap_or_else(|| theme.color_token("input"));
            let thickness = self.thickness.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.separator.px")
                    .unwrap_or(Px(1.0))
            });
            let layout = decl_style::layout_style(theme, self.layout);

            (bg, thickness, layout)
        };

        match self.orientation {
            SeparatorOrientation::Horizontal => {
                layout.size = SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(thickness),
                    min_height: Some(Length::Px(thickness)),
                    max_height: Some(Length::Px(thickness)),
                    ..layout.size
                };
            }
            SeparatorOrientation::Vertical => {
                layout.size = SizeStyle {
                    width: Length::Px(thickness),
                    // Match shadcn `self-stretch` behavior for vertical separators.
                    height: Length::Auto,
                    min_width: Some(Length::Px(thickness)),
                    max_width: Some(Length::Px(thickness)),
                    ..layout.size
                };
            }
        }

        let mut el = cx.container(
            fret_ui::element::ContainerProps {
                layout,
                background: Some(bg),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        if let Some(test_id) = self.test_id {
            el = el.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }

        el
    }
}

#[derive(Debug)]
pub enum ButtonGroupItem {
    Button(Button),
    Text(ButtonGroupText),
    InputGroup(InputGroup),
    Group(Box<ButtonGroup>),
    GroupSeparator(ButtonGroupSeparator),
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

impl From<ButtonGroupSeparator> for ButtonGroupItem {
    fn from(value: ButtonGroupSeparator) -> Self {
        Self::GroupSeparator(value)
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

#[derive(Debug)]
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

    #[track_caller]
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
            gap: gap.into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: fret_ui::element::MainAlign::Start,
            align: fret_ui::element::CrossAlign::Stretch,
            wrap: false,
        };

        let items = self.items;
        let orientation = self.orientation;
        let a11y_label = self.a11y_label;

        let group = cx.flex(props, move |cx| {
            let radius = Theme::global(&*cx.app).metric_token("metric.radius.md");
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
                    ButtonGroupItem::GroupSeparator(separator) => separator.into_element(cx),
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
