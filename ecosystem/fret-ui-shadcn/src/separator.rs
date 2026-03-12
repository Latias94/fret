//! shadcn/ui `Separator` (v4).
//!
//! Upstream reference: `repo-ref/ui/apps/v4/registry/bases/radix/ui/separator.tsx`

use fret_core::Px;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{IntoUiElement, LayoutRefinement};

use fret_ui_kit::primitives::separator as prim;

pub use prim::SeparatorOrientation;

/// shadcn/ui `Separator` (v4).
///
/// This is a thin wrapper over the headless separator primitive that applies shadcn-default
/// layout constraints:
/// - `shrink-0`
/// - horizontal: `h-px w-full`
/// - vertical: `w-px` + best-effort `self-stretch` equivalent via `flex_stretch_cross_axis=true`
#[derive(Debug, Clone)]
pub struct Separator {
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
            inner: prim::Separator::new()
                .refine_layout(LayoutRefinement::default().flex_shrink_0()),
        }
    }

    pub fn orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.inner = self.inner.orientation(orientation);
        if orientation == SeparatorOrientation::Vertical {
            // Upstream applies `self-stretch` for vertical separators. Fret currently has no
            // per-item `align-self` refinement, so we approximate the common flex-row use case
            // by opting into "stretch via auto cross-axis size".
            self.inner = self.inner.flex_stretch_cross_axis(true);
        }
        self
    }

    pub fn thickness(mut self, thickness: Px) -> Self {
        self.inner = self.inner.thickness(thickness);
        self
    }

    /// When `true`, a vertical separator uses `height: auto` so it can stretch when placed inside
    /// a flex row with `items-stretch` (best-effort `self-stretch` parity).
    pub fn flex_stretch_cross_axis(mut self, stretch: bool) -> Self {
        self.inner = self.inner.flex_stretch_cross_axis(stretch);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.inner = self.inner.refine_layout(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.inner.into_element(cx)
    }
}

pub fn separator<H: UiHost>() -> impl IntoUiElement<H> + use<H> {
    Separator::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
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

        let ElementKind::Container(props) = el.kind else {
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
    }

    #[test]
    fn vertical_separator_approximates_self_stretch() {
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

        let ElementKind::Container(props) = el.kind else {
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
            "expected vertical Separator to opt into cross-axis stretch via auto height"
        );
    }
}
