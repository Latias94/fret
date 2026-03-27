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

use fret_core::{Axis, Corners, Edges, FontId, FontWeight, Px, TextStyle};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, SemanticsDecoration, SizeStyle,
};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography::{self, TextIntent};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

use crate::button::Button;
use crate::input::Input;
use crate::input_group::InputGroup;
use crate::select::Select;
use crate::separator::{Separator, SeparatorOrientation};

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

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: ButtonGroupTextContent::Children(children.into_iter().collect()),
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
        let theme = Theme::global(&*cx.app).snapshot();

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
                    layout: LayoutStyle::default(),
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

                        let mut el = ui::text(text)
                            .text_size_px(style.size)
                            .font_weight(style.weight)
                            .text_color(ColorRef::Color(text_color))
                            .fixed_line_box_px(line_height)
                            .line_box_in_bounds()
                            .nowrap();
                        if let Some(letter_spacing_em) = style.letter_spacing_em {
                            el = el.letter_spacing_em(letter_spacing_em);
                        }
                        vec![el.into_element(cx)]
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
            // Match shadcn/ui v4 `Separator` defaults (`shrink-0`) so the divider does not collapse
            // under tight horizontal constraints.
            layout: LayoutRefinement::default().flex_shrink_0(),
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

pub enum ButtonGroupItem {
    Button(Button),
    Text(ButtonGroupText),
    Input(Input),
    InputGroup(InputGroup),
    Select(Select),
    Group(Box<ButtonGroup>),
    GroupSeparator(ButtonGroupSeparator),
    Separator(Separator),
    Element(AnyElement),
}

impl std::fmt::Debug for ButtonGroupItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Button(_) => f.debug_tuple("Button").finish(),
            Self::Text(_) => f.debug_tuple("Text").finish(),
            Self::Input(_) => f.debug_tuple("Input").finish(),
            Self::InputGroup(_) => f.debug_tuple("InputGroup").finish(),
            Self::Select(_) => f.debug_tuple("Select").finish(),
            Self::Group(_) => f.debug_tuple("Group").finish(),
            Self::GroupSeparator(_) => f.debug_tuple("GroupSeparator").finish(),
            Self::Separator(_) => f.debug_tuple("Separator").finish(),
            Self::Element(_) => f.debug_tuple("Element").finish(),
        }
    }
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

impl From<Input> for ButtonGroupItem {
    fn from(value: Input) -> Self {
        Self::Input(value)
    }
}

impl From<InputGroup> for ButtonGroupItem {
    fn from(value: InputGroup) -> Self {
        Self::InputGroup(value)
    }
}

impl From<Select> for ButtonGroupItem {
    fn from(value: Select) -> Self {
        Self::Select(value)
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

/// Upstream shadcn/ui `buttonGroupVariants(...)` compat surface.
///
/// Upstream returns a Tailwind/CVA class string. In Fret the button-group recipe applies the
/// important per-child border/corner merging directly, so this helper is primarily for copy/paste
/// parity in ports.
#[derive(Debug, Clone)]
pub struct ButtonGroupVariants {
    pub orientation: ButtonGroupOrientation,
    pub layout: LayoutRefinement,
    pub chrome: ChromeRefinement,
}

pub fn button_group_variants(orientation: ButtonGroupOrientation) -> ButtonGroupVariants {
    ButtonGroupVariants {
        orientation,
        // Upstream base includes `flex w-fit items-stretch`. Layout direction is owned by the
        // `ButtonGroup` element in Fret (not by this helper).
        layout: LayoutRefinement::default(),
        chrome: ChromeRefinement::default(),
    }
}

pub struct ButtonGroup {
    items: Vec<ButtonGroupItem>,
    orientation: ButtonGroupOrientation,
    layout: LayoutRefinement,
    radius_override: Option<Radius>,
    a11y_label: Option<Arc<str>>,
    labelled_by_element: Option<GlobalElementId>,
}

impl ButtonGroup {
    pub fn new(items: impl IntoIterator<Item = ButtonGroupItem>) -> Self {
        let items = items.into_iter().collect();
        Self {
            items,
            orientation: ButtonGroupOrientation::default(),
            layout: LayoutRefinement::default(),
            radius_override: None,
            a11y_label: None,
            labelled_by_element: None,
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

    /// Associates this group with an external label element (`aria-labelledby`-like outcome).
    pub fn labelled_by_element(mut self, label: GlobalElementId) -> Self {
        self.labelled_by_element = Some(label);
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

    /// Overrides the radius used when merging child borders/corners.
    ///
    /// This matches shadcn's `--radius` recipe patterns (e.g. fully-rounded nested groups).
    pub fn radius_override(mut self, radius: Radius) -> Self {
        self.radius_override = Some(radius);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let dir = crate::direction::use_direction(cx, None);

        let has_nested_group = self
            .items
            .iter()
            .any(|i| matches!(i, ButtonGroupItem::Group(_)));

        let gap = if has_nested_group { Px(8.0) } else { Px(0.0) };
        let direction = match self.orientation {
            ButtonGroupOrientation::Horizontal => Axis::Horizontal,
            ButtonGroupOrientation::Vertical => Axis::Vertical,
        };

        let mut outer_layout = decl_style::layout_style(&theme, self.layout);
        // Upstream shadcn/ui v4 `ButtonGroup` includes `w-fit` so groups do not stretch under parent
        // stacks using `items-stretch`. Approximate that by opting out of cross-axis stretch unless
        // a width is explicitly requested.
        if outer_layout.flex.align_self.is_none() && matches!(outer_layout.size.width, Length::Auto)
        {
            outer_layout.flex.align_self = Some(CrossAlign::Start);
        }
        let props = FlexProps {
            layout: outer_layout,
            direction,
            gap: gap.into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: fret_ui::element::MainAlign::Start,
            align: fret_ui::element::CrossAlign::Stretch,
            wrap: false,
        };

        let items = self.items;
        let orientation = self.orientation;
        let radius_override = self.radius_override;
        let a11y_label = self.a11y_label;
        let labelled_by_element = self.labelled_by_element;

        let group = cx.flex(props, move |cx| {
            let radius = {
                let theme = Theme::global(&*cx.app);
                radius_override
                    .map(|r| MetricRef::radius(r).resolve(theme))
                    .unwrap_or_else(|| theme.metric_token("metric.radius.md"))
            };
            let len = items.len();

            items
                .into_iter()
                .enumerate()
                .map(|(idx, item)| {
                    let visual = crate::rtl::horizontal_visual_item_position(dir, idx, len);
                    let (is_first, is_last, order) = match orientation {
                        ButtonGroupOrientation::Horizontal => {
                            (visual.is_visual_first, visual.is_visual_last, visual.order)
                        }
                        ButtonGroupOrientation::Vertical => (idx == 0, idx + 1 == len, None),
                    };

                    match item {
                        ButtonGroupItem::Button(button) => {
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
                            let button = if let Some(order) = order {
                                button.refine_layout(LayoutRefinement::default().order(order))
                            } else {
                                button
                            };

                            button.corner_radii_override(corners).into_element(cx)
                        }
                        ButtonGroupItem::Text(text) => {
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
                            let text = if let Some(order) = order {
                                text.refine_layout(LayoutRefinement::default().order(order))
                            } else {
                                text
                            };

                            text.corner_radii_override(corners).into_element(cx)
                        }
                        ButtonGroupItem::Input(input) => {
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

                            let input = match orientation {
                                ButtonGroupOrientation::Horizontal => {
                                    if is_first {
                                        input
                                    } else {
                                        input.border_left_width_override(Px(0.0))
                                    }
                                }
                                ButtonGroupOrientation::Vertical => {
                                    if is_first {
                                        input
                                    } else {
                                        input.border_top_width_override(Px(0.0))
                                    }
                                }
                            };
                            let mut input_layout = LayoutRefinement::default();
                            if let Some(order) = order {
                                input_layout = input_layout.order(order);
                            }
                            let input = match orientation {
                                ButtonGroupOrientation::Horizontal => {
                                    input.refine_layout(input_layout.flex_1())
                                }
                                ButtonGroupOrientation::Vertical => {
                                    if order.is_some() {
                                        input.refine_layout(input_layout)
                                    } else {
                                        input
                                    }
                                }
                            };

                            input.corner_radii_override(corners).into_element(cx)
                        }
                        ButtonGroupItem::InputGroup(group) => {
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
                            let mut group_layout = LayoutRefinement::default();
                            if let Some(order) = order {
                                group_layout = group_layout.order(order);
                            }
                            let group = match orientation {
                                ButtonGroupOrientation::Horizontal => {
                                    group.refine_layout(group_layout.flex_1())
                                }
                                ButtonGroupOrientation::Vertical => {
                                    if order.is_some() {
                                        group.refine_layout(group_layout)
                                    } else {
                                        group
                                    }
                                }
                            };

                            group.corner_radii_override(corners).into_element(cx)
                        }
                        ButtonGroupItem::Select(select) => {
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

                            let select = match orientation {
                                ButtonGroupOrientation::Horizontal => {
                                    if is_first {
                                        select
                                    } else {
                                        select.border_left_width_override(Px(0.0))
                                    }
                                }
                                ButtonGroupOrientation::Vertical => {
                                    if is_first {
                                        select
                                    } else {
                                        select.border_top_width_override(Px(0.0))
                                    }
                                }
                            };
                            let select = if let Some(order) = order {
                                select.refine_layout(LayoutRefinement::default().order(order))
                            } else {
                                select
                            };

                            select.corner_radii_override(corners).into_element(cx)
                        }
                        ButtonGroupItem::Group(group) => {
                            let group = *group;
                            let flex_grows = group
                                .layout
                                .flex_item
                                .as_ref()
                                .and_then(|flex| flex.grow)
                                .is_some_and(|grow| grow > 0.0);
                            let group = if let Some(radius) = radius_override {
                                group.radius_override(radius)
                            } else {
                                group
                            };
                            let group = if orientation == ButtonGroupOrientation::Horizontal
                                && flex_grows
                            {
                                group.refine_layout(LayoutRefinement::default().w_full().min_w_0())
                            } else {
                                group
                            };
                            let group = if let Some(order) = order {
                                group.refine_layout(LayoutRefinement::default().order(order))
                            } else {
                                group
                            };
                            group.into_element(cx)
                        }
                        ButtonGroupItem::GroupSeparator(separator) => {
                            let separator = if let Some(order) = order {
                                separator.refine_layout(LayoutRefinement::default().order(order))
                            } else {
                                separator
                            };
                            separator.into_element(cx)
                        }
                        ButtonGroupItem::Separator(separator) => {
                            let separator = separator.flex_stretch_cross_axis(true);
                            let separator = if let Some(order) = order {
                                separator.refine_layout(LayoutRefinement::default().order(order))
                            } else {
                                separator
                            };
                            separator.into_element(cx)
                        }
                        ButtonGroupItem::Element(element) => {
                            if let Some(order) = order {
                                let layout = decl_style::layout_style(
                                    Theme::global(&*cx.app),
                                    LayoutRefinement::default().order(order),
                                );
                                cx.container(
                                    fret_ui::element::ContainerProps {
                                        layout,
                                        ..Default::default()
                                    },
                                    move |_cx| vec![element],
                                )
                            } else {
                                element
                            }
                        }
                    }
                })
                .collect::<Vec<_>>()
        });

        let mut decoration = SemanticsDecoration::default().role(fret_core::SemanticsRole::Group);
        if let Some(a11y_label) = a11y_label {
            decoration = decoration.label(a11y_label);
        }
        if let Some(labelled_by_element) = labelled_by_element {
            decoration = decoration.labelled_by_element(labelled_by_element.0);
        }
        group.attach_semantics(decoration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::direction::LayoutDirection;
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{ElementKind, Length, MarginEdge};

    use crate::rtl;

    fn apply_theme(app: &mut App) {
        crate::shadcn_themes::apply_shadcn_new_york(
            app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );
    }

    fn bounds_320x240() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        )
    }

    fn render_group(app: &mut App, window: AppWindowId, group: ButtonGroup) -> AnyElement {
        fret_ui::elements::with_element_cx(app, window, bounds_320x240(), "test", |cx| {
            group.into_element(cx)
        })
    }

    fn render_group_rtl(app: &mut App, window: AppWindowId, group: ButtonGroup) -> AnyElement {
        fret_ui::elements::with_element_cx(app, window, bounds_320x240(), "test", |cx| {
            crate::direction::with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                group.into_element(cx)
            })
        })
    }

    fn basic_text_items() -> [ButtonGroupItem; 2] {
        [
            ButtonGroupText::new("A").into(),
            ButtonGroupText::new("B").into(),
        ]
    }

    #[test]
    fn button_group_stamps_role_without_layout_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();

        apply_theme(&mut app);

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds_320x240(), "test", |cx| {
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

    #[test]
    fn button_group_can_reference_a_label_element_for_a11y_association() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds_320x240(), "test", |cx| {
                let label = crate::label::Label::new("Formatting").into_element(cx);
                let label_id = label.id;

                cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                    vec![
                        label,
                        ButtonGroup::new([
                            ButtonGroupText::new("Bold").into(),
                            ButtonGroupText::new("Italic").into(),
                        ])
                        .labelled_by_element(label_id)
                        .into_element(cx),
                    ]
                })
            });

        let group = element.children.get(1).expect("button group child");
        let decoration = group
            .semantics_decoration
            .as_ref()
            .expect("expected semantics decoration on ButtonGroup root");
        assert_eq!(decoration.role, Some(fret_core::SemanticsRole::Group));
        assert_eq!(
            decoration.labelled_by_element,
            Some(element.children[0].id.0)
        );
    }

    #[test]
    fn button_group_defaults_to_w_fit_horizontal_stretch_and_no_gap() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let element = render_group(&mut app, window, ButtonGroup::new(basic_text_items()));
        let ElementKind::Flex(props) = &element.kind else {
            panic!("expected ButtonGroup to render as a flex element");
        };
        assert_eq!(props.layout.size.width, Length::Auto);
        assert_eq!(
            props.layout.flex.align_self,
            Some(fret_ui::element::CrossAlign::Start)
        );
        assert_eq!(props.direction, fret_core::Axis::Horizontal);
        assert_eq!(props.align, fret_ui::element::CrossAlign::Stretch);
        assert_eq!(props.gap, Px(0.0).into());
    }

    #[test]
    fn button_group_nested_flex_growing_group_fills_available_width() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let nested = ButtonGroup::new(basic_text_items())
            .refine_layout(LayoutRefinement::default().flex_1());
        let outer = render_group(
            &mut app,
            window,
            ButtonGroup::new([ButtonGroupItem::from(nested)]),
        );

        let ElementKind::Flex(inner) = &outer.kind else {
            panic!("expected outer ButtonGroup to render as a flex element");
        };
        assert_eq!(outer.children.len(), 1);

        let ElementKind::Flex(nested_outer) = &outer.children[0].kind else {
            panic!("expected nested ButtonGroup to render as a flex element");
        };
        assert_eq!(inner.direction, fret_core::Axis::Horizontal);
        assert_eq!(nested_outer.layout.size.width, Length::Fill);
        assert_eq!(
            nested_outer.layout.size.min_width,
            Some(Length::Px(Px(0.0)))
        );
    }

    #[test]
    fn button_group_horizontal_merges_borders_and_corners_for_children() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let outer = render_group(&mut app, window, ButtonGroup::new(basic_text_items()));
        assert_eq!(outer.children.len(), 2);

        let left = &outer.children[0];
        let right = &outer.children[1];

        let ElementKind::Container(left_props) = &left.kind else {
            panic!("expected left child to be a container (ButtonGroupText)");
        };
        let ElementKind::Container(right_props) = &right.kind else {
            panic!("expected right child to be a container (ButtonGroupText)");
        };

        // Merged borders: remove the inner border to avoid double-width seams.
        assert_eq!(right_props.border.left, Px(0.0));

        // Merged corners: middle edges lose radius.
        assert_eq!(left_props.corner_radii.top_right, Px(0.0));
        assert_eq!(left_props.corner_radii.bottom_right, Px(0.0));
        assert_eq!(right_props.corner_radii.top_left, Px(0.0));
        assert_eq!(right_props.corner_radii.bottom_left, Px(0.0));
    }

    #[test]
    fn button_group_horizontal_uses_logical_visual_order_in_rtl() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let outer = render_group_rtl(
            &mut app,
            window,
            ButtonGroup::new([
                ButtonGroupText::new("A").test_id("button-group-a").into(),
                ButtonGroupText::new("B").test_id("button-group-b").into(),
            ]),
        );
        assert_eq!(outer.children.len(), 2);

        let a = &outer.children[0];
        let b = &outer.children[1];

        let ElementKind::Container(a_props) = &a.kind else {
            panic!("expected first source child to be a container (ButtonGroupText)");
        };
        let ElementKind::Container(b_props) = &b.kind else {
            panic!("expected second source child to be a container (ButtonGroupText)");
        };

        let (order_inline_start, order_inline_end) =
            rtl::inline_start_end_pair(LayoutDirection::Rtl, 0, 1);
        assert_eq!(a_props.layout.flex.order, order_inline_start);
        assert_eq!(b_props.layout.flex.order, order_inline_end);

        assert_eq!(a_props.border.left, Px(0.0));
        assert_eq!(a_props.corner_radii.top_left, Px(0.0));
        assert_eq!(a_props.corner_radii.bottom_left, Px(0.0));
        assert_eq!(b_props.corner_radii.top_right, Px(0.0));
        assert_eq!(b_props.corner_radii.bottom_right, Px(0.0));
    }

    #[test]
    fn button_group_vertical_merges_borders_and_corners_for_children() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let outer = render_group(
            &mut app,
            window,
            ButtonGroup::new(basic_text_items()).orientation(ButtonGroupOrientation::Vertical),
        );
        let ElementKind::Flex(props) = &outer.kind else {
            panic!("expected ButtonGroup to render as a flex element");
        };
        assert_eq!(props.direction, fret_core::Axis::Vertical);
        assert_eq!(props.align, fret_ui::element::CrossAlign::Stretch);

        assert_eq!(outer.children.len(), 2);
        let top = &outer.children[0];
        let bottom = &outer.children[1];

        let ElementKind::Container(top_props) = &top.kind else {
            panic!("expected top child to be a container (ButtonGroupText)");
        };
        let ElementKind::Container(bottom_props) = &bottom.kind else {
            panic!("expected bottom child to be a container (ButtonGroupText)");
        };

        assert_eq!(bottom_props.border.top, Px(0.0));
        assert_eq!(top_props.corner_radii.bottom_left, Px(0.0));
        assert_eq!(top_props.corner_radii.bottom_right, Px(0.0));
        assert_eq!(bottom_props.corner_radii.top_left, Px(0.0));
        assert_eq!(bottom_props.corner_radii.top_right, Px(0.0));
    }

    #[test]
    fn button_group_nested_group_inserts_gap_8px() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let nested = ButtonGroup::new([ButtonGroupText::new("N").into()]);
        let outer = render_group(
            &mut app,
            window,
            ButtonGroup::new([
                ButtonGroupItem::from(nested),
                ButtonGroupText::new("A").into(),
            ]),
        );
        let ElementKind::Flex(props) = &outer.kind else {
            panic!("expected ButtonGroup to render as a flex element");
        };
        assert_eq!(props.gap, Px(8.0).into());
    }

    #[test]
    fn button_group_text_new_children_preserves_inline_custom_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let text =
            fret_ui::elements::with_element_cx(&mut app, window, bounds_320x240(), "test", |cx| {
                ButtonGroupText::new_children([cx.text("prefix"), cx.text("suffix")])
                    .into_element(cx)
            });

        let ElementKind::Container(_) = &text.kind else {
            panic!("expected ButtonGroupText to render a container");
        };
        assert_eq!(text.children.len(), 1);

        let ElementKind::Flex(_) = &text.children[0].kind else {
            panic!("expected ButtonGroupText content row to render as a flex element");
        };
        assert_eq!(text.children[0].children.len(), 2);
    }

    #[test]
    fn button_group_separator_defaults_to_zero_margins() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let vertical =
            fret_ui::elements::with_element_cx(&mut app, window, bounds_320x240(), "test", |cx| {
                ButtonGroupSeparator::new().into_element(cx)
            });
        let ElementKind::Container(props) = &vertical.kind else {
            panic!("expected ButtonGroupSeparator to render a container");
        };
        assert_eq!(props.layout.margin.top, MarginEdge::Px(Px(0.0)));
        assert_eq!(props.layout.margin.bottom, MarginEdge::Px(Px(0.0)));

        let horizontal =
            fret_ui::elements::with_element_cx(&mut app, window, bounds_320x240(), "test", |cx| {
                ButtonGroupSeparator::new()
                    .orientation(SeparatorOrientation::Horizontal)
                    .into_element(cx)
            });
        let ElementKind::Container(props) = &horizontal.kind else {
            panic!("expected ButtonGroupSeparator(horizontal) to render a container");
        };
        assert_eq!(props.layout.margin.left, MarginEdge::Px(Px(0.0)));
        assert_eq!(props.layout.margin.right, MarginEdge::Px(Px(0.0)));
    }
}
