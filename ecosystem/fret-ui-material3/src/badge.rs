//! Material 3 badge (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.badge.*` (Material Web v30).
//! - Supports dot and value (large) variants.
//! - Provides a small anchoring helper for navigation icons.

use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, LayoutDirection, Px, SemanticsRole, TextOverflow, TextWrap,
};
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, InsetStyle, Length, PositionStyle, SemanticsDecoration,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Theme, UiHost};

use crate::foundation::context::material_layout_direction_in_scope;
use crate::foundation::test_id::part_test_id;
use crate::tokens::badge as badge_tokens;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BadgeValue {
    Dot,
    Text(Arc<str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgePlacement {
    /// Align with Material Web labs navigation tab badge placement:
    /// - Start edge at `50%` of the anchor width + a small px offset.
    #[default]
    NavigationIcon,
    /// Pin to the top-right of the anchor container.
    TopRight,
}

#[derive(Debug, Clone)]
pub struct Badge {
    value: BadgeValue,
    placement: BadgePlacement,
    anchor_size: Option<Px>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl Badge {
    pub fn dot() -> Self {
        Self {
            value: BadgeValue::Dot,
            placement: BadgePlacement::NavigationIcon,
            anchor_size: None,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn text(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: BadgeValue::Text(value.into()),
            placement: BadgePlacement::NavigationIcon,
            anchor_size: None,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn placement(mut self, placement: BadgePlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Sets the badge anchor box size used for deterministic placement.
    ///
    /// This is required when the anchor content is not enough to infer a stable relative box,
    /// especially for `BadgePlacement::TopRight`.
    pub fn anchor_size(mut self, size: Px) -> Self {
        self.anchor_size = Some(size);
        self
    }

    pub fn navigation_anchor_size(self, size: Px) -> Self {
        self.anchor_size(size)
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        anchor: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        cx.scope(|cx| {
            let anchor_children: Vec<AnyElement> = anchor(cx).into_iter().collect();
            let part_test_ids = self.test_id.clone().map(BadgePartTestIds::new);

            let (layout_direction, resolved) = {
                let theme = Theme::global(&*cx.app);
                let layout_direction = material_layout_direction_in_scope(&*cx);
                let resolved = BadgeResolvedTokens::resolve(theme);
                (layout_direction, resolved)
            };

            let mut wrapper = ContainerProps::default();
            wrapper.layout.position = PositionStyle::Relative;
            wrapper.layout.overflow = fret_ui::element::Overflow::Visible;
            if let Some(anchor_size) = self.anchor_size {
                wrapper.layout.size.width = Length::Px(anchor_size);
                wrapper.layout.size.height = Length::Px(anchor_size);
            }

            let mut anchor_wrapper = ContainerProps::default();
            if let Some(anchor_size) = self.anchor_size {
                anchor_wrapper.layout.size.width = Length::Px(anchor_size);
                anchor_wrapper.layout.size.height = Length::Px(anchor_size);
            }
            let mut anchor = cx.container(anchor_wrapper, move |_cx| anchor_children);
            if let Some(test_id) = part_test_ids.as_ref().map(|ids| ids.anchor.clone()) {
                anchor = anchor.test_id(test_id);
            }

            let badge = badge_element(
                cx,
                layout_direction,
                resolved,
                self.value.clone(),
                self.placement,
                self.anchor_size,
                self.a11y_label.clone(),
                part_test_ids.as_ref().map(|ids| ids.badge.clone()),
            );

            let mut root = cx.container(wrapper, move |_cx| vec![anchor, badge]);
            if let Some(test_id) = part_test_ids.map(|ids| ids.root) {
                root = root.a11y(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id(test_id),
                );
            }
            root
        })
    }
}

#[derive(Debug, Clone)]
struct BadgePartTestIds {
    root: Arc<str>,
    anchor: Arc<str>,
    badge: Arc<str>,
}

impl BadgePartTestIds {
    fn new(root: Arc<str>) -> Self {
        Self {
            anchor: part_test_id(&root, "anchor"),
            badge: part_test_id(&root, "badge"),
            root,
        }
    }
}

#[derive(Debug, Clone)]
struct BadgeResolvedTokens {
    dot_size: Px,
    dot_color: Color,
    dot_shape: Corners,
    large_size: Px,
    large_color: Color,
    large_shape: Corners,
    large_label_style: fret_core::TextStyle,
    large_label_color: Color,
}

impl BadgeResolvedTokens {
    fn resolve(theme: &Theme) -> Self {
        Self {
            dot_size: badge_tokens::dot_size(theme),
            dot_color: badge_tokens::dot_color(theme),
            dot_shape: badge_tokens::shape(theme),
            large_size: badge_tokens::large_size(theme),
            large_color: badge_tokens::large_color(theme),
            large_shape: badge_tokens::large_shape(theme),
            large_label_style: badge_tokens::large_label_text_style(theme),
            large_label_color: badge_tokens::large_label_color(theme),
        }
    }
}

fn badge_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout_direction: LayoutDirection,
    resolved: BadgeResolvedTokens,
    value: BadgeValue,
    placement: BadgePlacement,
    anchor_size: Option<Px>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let semantics_value = match &value {
        BadgeValue::Dot => None,
        BadgeValue::Text(text) => Some(text.clone()),
    };

    let (is_large, inset_start_px, inset_top_px) = match value {
        BadgeValue::Dot => (false, Px(6.0), Px(4.0)),
        BadgeValue::Text(_) => (true, Px(2.0), Px(1.0)),
    };

    let mut inset = InsetStyle::default();
    match placement {
        BadgePlacement::TopRight => {
            inset.top = Some(Px(0.0)).into();
            if let Some(anchor) = anchor_size {
                let badge_width = if is_large {
                    resolved.large_size
                } else {
                    resolved.dot_size
                };
                inset.left = Some(Px((anchor.0 - badge_width.0).max(0.0))).into();
            } else {
                inset.right = Some(Px(0.0)).into();
            }
        }
        BadgePlacement::NavigationIcon => {
            let anchor = anchor_size.unwrap_or(Px(24.0));
            let start = Px(anchor.0 * 0.5 + inset_start_px.0);
            inset.top = Some(inset_top_px).into();
            match layout_direction {
                LayoutDirection::Ltr => inset.left = Some(start).into(),
                LayoutDirection::Rtl => inset.right = Some(start).into(),
            }
        }
    }

    let (height, width, min_width, background, corner_radii) = if is_large {
        (
            resolved.large_size,
            Length::Auto,
            Some(resolved.large_size),
            resolved.large_color,
            resolved.large_shape,
        )
    } else {
        (
            resolved.dot_size,
            Length::Px(resolved.dot_size),
            Some(resolved.dot_size),
            resolved.dot_color,
            resolved.dot_shape,
        )
    };

    let mut container = ContainerProps::default();
    container.layout.position = PositionStyle::Absolute;
    container.layout.inset = inset;
    container.layout.size.height = Length::Px(height);
    container.layout.size.width = width;
    container.layout.size.min_width = min_width.map(Length::Px);
    container.background = Some(background);
    container.corner_radii = corner_radii;
    if is_large {
        container.padding = Edges {
            left: Px(4.0),
            right: Px(4.0),
            top: Px(0.0),
            bottom: Px(0.0),
        }
        .into();
    }

    let content = match value {
        BadgeValue::Dot => cx.container(container, move |_cx| Vec::<AnyElement>::new()),
        BadgeValue::Text(text) => {
            let mut props = fret_ui::element::TextProps::new(text.clone());
            props.style = Some(resolved.large_label_style);
            props.color = Some(resolved.large_label_color);
            props.wrap = TextWrap::None;
            props.overflow = TextOverflow::Clip;

            let mut flex = FlexProps::default();
            flex.direction = fret_core::Axis::Horizontal;
            flex.justify = fret_ui::element::MainAlign::Center;
            flex.align = fret_ui::element::CrossAlign::Center;
            flex.wrap = false;

            cx.container(container, move |cx| {
                vec![cx.flex(flex, move |cx| vec![cx.text_props(props)])]
            })
        }
    };

    let mut decoration = SemanticsDecoration::default().role(SemanticsRole::Generic);
    if let Some(label) = a11y_label {
        decoration = decoration.label(label);
    }
    if let Some(value) = semantics_value {
        decoration = decoration.value(value);
    }
    if let Some(test_id) = test_id {
        decoration = decoration.test_id(test_id);
    }

    content.a11y(decoration)
}
