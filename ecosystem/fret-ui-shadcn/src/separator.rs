//! shadcn/ui `Separator` (v4).
//!
//! Upstream reference: `repo-ref/ui/apps/v4/registry/bases/radix/ui/separator.tsx`

use fret_core::Px;
use fret_ui::element::{AnyElement, CrossAlign, ElementKind};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{IntoUiElement, LayoutRefinement};

use fret_ui_kit::primitives::separator as prim;

pub use prim::SeparatorOrientation;

/// shadcn/ui `Separator` (v4).
///
/// This is a thin wrapper over the headless separator primitive that applies shadcn-default
/// layout constraints:
/// - `shrink-0`
/// - decorative by default
/// - horizontal: `h-px w-full`
/// - vertical: `w-px self-stretch`
#[derive(Debug, Clone)]
pub struct Separator {
    orientation: SeparatorOrientation,
    align_self_stretch: bool,
    flex_stretch_cross_axis: bool,
    inner: prim::Separator,
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl Separator {
    pub fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            align_self_stretch: false,
            // Upstream `registry/bases/{base,radix}/ui/separator.tsx` uses
            // `data-vertical:self-stretch`, which maps to `height: auto`
            // plus `align-self: stretch` in Fret's flex model.
            flex_stretch_cross_axis: true,
            inner: prim::Separator::new()
                .decorative(true)
                .refine_layout(LayoutRefinement::default().flex_shrink_0()),
        }
    }

    pub fn orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.orientation = orientation;
        self.align_self_stretch = orientation == SeparatorOrientation::Vertical;
        self.inner = self
            .inner
            .orientation(orientation)
            .flex_stretch_cross_axis(self.flex_stretch_cross_axis);
        self
    }

    pub fn thickness(mut self, thickness: Px) -> Self {
        self.inner = self.inner.thickness(thickness);
        self
    }

    pub fn decorative(mut self, decorative: bool) -> Self {
        self.inner = self.inner.decorative(decorative);
        self
    }

    /// Controls whether vertical separators keep the upstream `self-stretch` lane (`true`,
    /// default) or use the rarer fill-height lane (`false`) for custom compositions that really
    /// want `height: 100%`.
    pub fn flex_stretch_cross_axis(mut self, stretch: bool) -> Self {
        self.flex_stretch_cross_axis = stretch;
        self.inner = self.inner.flex_stretch_cross_axis(stretch);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.inner = self.inner.refine_layout(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut element = self.inner.into_element(cx);

        if self.orientation == SeparatorOrientation::Vertical && self.align_self_stretch {
            if let ElementKind::Container(props) = &mut element.kind {
                props.layout.flex.align_self = Some(CrossAlign::Stretch);
            }
        }

        element
    }
}

pub fn separator<H: UiHost>() -> impl IntoUiElement<H> + use<H> {
    Separator::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, SemanticsOrientation, SemanticsRole, Size};
    use fret_ui::element::{ElementKind, Length};

    #[test]
    fn separator_defaults_match_shadcn_constraints() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Separator::new().into_element(cx)
        });

        let ElementKind::Container(props) = &el.kind else {
            panic!("expected Separator to be a Container element");
        };

        assert_eq!(
            props.layout.flex.shrink, 0.0,
            "expected shadcn Separator to default to shrink-0"
        );
        assert_eq!(
            props.layout.size.width,
            Length::Fill,
            "expected horizontal Separator to default to w-full"
        );
        assert_eq!(
            props.layout.size.height,
            Length::Px(Px(1.0)),
            "expected horizontal Separator to default to h-px"
        );
        let decoration = el
            .semantics_decoration
            .as_ref()
            .expect("expected shadcn separator semantics decoration");
        assert_eq!(decoration.hidden, Some(true));
        assert_eq!(decoration.role, None);
    }

    #[test]
    fn vertical_separator_matches_shadcn_self_stretch() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Separator::new()
                .orientation(SeparatorOrientation::Vertical)
                .into_element(cx)
        });

        let ElementKind::Container(props) = &el.kind else {
            panic!("expected Separator to be a Container element");
        };

        assert_eq!(
            props.layout.size.width,
            Length::Px(Px(1.0)),
            "expected vertical Separator to default to w-px"
        );
        assert_eq!(
            props.layout.size.height,
            Length::Auto,
            "expected vertical Separator to map upstream self-stretch to auto cross-axis sizing"
        );
        assert_eq!(
            props.layout.flex.align_self,
            Some(CrossAlign::Stretch),
            "expected vertical Separator to translate upstream self-stretch"
        );
    }

    #[test]
    fn vertical_separator_fill_height_lane_is_explicit_opt_out() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Separator::new()
                .orientation(SeparatorOrientation::Vertical)
                .flex_stretch_cross_axis(false)
                .into_element(cx)
        });

        let ElementKind::Container(props) = &el.kind else {
            panic!("expected Separator to be a Container element");
        };

        assert_eq!(
            props.layout.size.height,
            Length::Fill,
            "expected fill-height to remain available as an explicit escape hatch"
        );
        assert_eq!(props.layout.flex.align_self, Some(CrossAlign::Stretch));
    }

    #[test]
    fn semantic_vertical_separator_is_opt_in() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Separator::new()
                .orientation(SeparatorOrientation::Vertical)
                .decorative(false)
                .into_element(cx)
        });

        let decoration = el
            .semantics_decoration
            .as_ref()
            .expect("expected semantic separator decoration");
        assert_eq!(decoration.hidden, None);
        assert_eq!(decoration.role, Some(SemanticsRole::Separator));
        assert_eq!(decoration.orientation, Some(SemanticsOrientation::Vertical));
    }
}
