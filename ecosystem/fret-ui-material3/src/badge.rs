//! Material 3 badge (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.badge.*` (Material Web v30).
//! - Supports dot and value (large) variants.
//! - Provides a small anchoring helper for navigation icons.

use std::sync::Arc;

use fret_core::{Edges, LayoutDirection, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ContainerProps, FlexProps, InsetStyle, Length, PositionStyle};
use fret_ui::elements::ElementContext;
use fret_ui::{Theme, UiHost};

use crate::foundation::context::{resolved_layout_direction, theme_default_layout_direction};
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

    pub fn navigation_anchor_size(mut self, size: Px) -> Self {
        self.anchor_size = Some(size);
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        anchor: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let layout_direction =
                resolved_layout_direction(cx, theme_default_layout_direction(&theme));

            let mut wrapper = ContainerProps::default();
            wrapper.layout.position = PositionStyle::Relative;
            wrapper.layout.overflow = fret_ui::element::Overflow::Visible;

            let anchor_children: Vec<AnyElement> = anchor(cx).into_iter().collect();
            let badge = badge_element(
                cx,
                &theme,
                layout_direction,
                self.value.clone(),
                self.placement,
                self.anchor_size,
                self.a11y_label.clone(),
                self.test_id.clone(),
            );

            cx.container(wrapper, move |_cx| {
                let mut out = anchor_children;
                out.push(badge);
                out
            })
        })
    }
}

fn badge_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    layout_direction: LayoutDirection,
    value: BadgeValue,
    placement: BadgePlacement,
    anchor_size: Option<Px>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let (is_large, inset_start_px, inset_top_px) = match value {
        BadgeValue::Dot => (false, Px(6.0), Px(4.0)),
        BadgeValue::Text(_) => (true, Px(2.0), Px(1.0)),
    };

    let mut inset = InsetStyle::default();
    match placement {
        BadgePlacement::TopRight => {
            inset.top = Some(Px(0.0));
            inset.right = Some(Px(0.0));
        }
        BadgePlacement::NavigationIcon => {
            let anchor = anchor_size.unwrap_or(Px(24.0));
            let start = Px(anchor.0 * 0.5 + inset_start_px.0);
            inset.top = Some(inset_top_px);
            match layout_direction {
                LayoutDirection::Ltr => inset.left = Some(start),
                LayoutDirection::Rtl => inset.right = Some(start),
            }
        }
    }

    let (height, width, min_width, background, corner_radii) = if is_large {
        let size = badge_tokens::large_size(theme);
        (
            size,
            Length::Auto,
            Some(size),
            badge_tokens::large_color(theme),
            badge_tokens::large_shape(theme),
        )
    } else {
        let size = badge_tokens::dot_size(theme);
        (
            size,
            Length::Px(size),
            Some(size),
            badge_tokens::dot_color(theme),
            badge_tokens::shape(theme),
        )
    };

    let mut container = ContainerProps::default();
    container.layout.position = PositionStyle::Absolute;
    container.layout.inset = inset;
    container.layout.size.height = Length::Px(height);
    container.layout.size.width = width;
    container.layout.size.min_width = min_width;
    container.background = Some(background);
    container.corner_radii = corner_radii;
    if is_large {
        container.padding = Edges {
            left: Px(4.0),
            right: Px(4.0),
            top: Px(0.0),
            bottom: Px(0.0),
        };
    }

    let content = match value {
        BadgeValue::Dot => cx.container(container, move |_cx| Vec::<AnyElement>::new()),
        BadgeValue::Text(text) => {
            let style = theme
                .text_style_by_key("md.comp.badge.large.label-text")
                .or_else(|| theme.text_style_by_key("md.sys.typescale.label-small"))
                .unwrap_or_else(fret_core::TextStyle::default);

            let mut props = fret_ui::element::TextProps::new(text.clone());
            props.style = Some(style);
            props.color = Some(badge_tokens::large_label_color(theme));
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

    cx.semantics(
        fret_ui::element::SemanticsProps {
            role: SemanticsRole::Generic,
            label: a11y_label,
            test_id,
            ..Default::default()
        },
        move |_cx| vec![content],
    )
}
