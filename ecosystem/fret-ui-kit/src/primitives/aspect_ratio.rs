//! Aspect Ratio primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/aspect-ratio/src/aspect-ratio.tsx`
//!
//! Note: Fret enforces aspect ratio via the layout engine (ADR 0057). This primitive is a small
//! declarative helper that stamps the `aspect_ratio` constraint onto a container.

use fret_ui::element::{AnyElement, Overflow};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::style as decl_style;
use crate::{ChromeRefinement, LayoutRefinement};

/// Radix-style `AspectRatio` wrapper.
///
/// This applies `layout.aspect_ratio = Some(ratio)` and defaults to `Overflow::Clip`.
#[derive(Debug, Clone)]
pub struct AspectRatio {
    ratio: f32,
    child: AnyElement,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    overflow: Overflow,
}

impl AspectRatio {
    pub fn new(ratio: f32, child: AnyElement) -> Self {
        Self {
            ratio,
            child,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            overflow: Overflow::Clip,
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let base_layout = LayoutRefinement::default().w_full();
        let mut props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                self.chrome,
                base_layout.aspect_ratio(self.ratio).merge(self.layout),
            )
        };
        props.layout.overflow = self.overflow;

        let child = self.child;
        cx.container(props, move |_cx| vec![child])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::ElementKind;

    #[test]
    fn aspect_ratio_stamps_layout_ratio() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let child = cx.container(Default::default(), |_cx| Vec::new());
            let el = AspectRatio::new(16.0 / 9.0, child).into_element(cx);

            let ElementKind::Container(props) = &el.kind else {
                panic!("expected a container element");
            };
            assert_eq!(props.layout.aspect_ratio, Some(16.0 / 9.0));
            assert_eq!(props.layout.overflow, Overflow::Clip);
        });
    }
}
