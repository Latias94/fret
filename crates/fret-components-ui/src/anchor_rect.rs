use fret_core::{AppWindowId, Rect};
use fret_ui::{GlobalElementId, UiHost, elements::bounds_for_element};

/// An overlay anchor that can be resolved from either a concrete rect or a declarative element id.
///
/// This is intentionally a small contract between component-layer overlay policies and the
/// declarative runtime (ADR 0066): the runtime owns the element-to-geometry mapping, and overlay
/// services can request a stable anchor rect without depending on retained widgets.
#[derive(Debug, Clone, Copy)]
pub struct AnchorRect {
    element: Option<GlobalElementId>,
    fallback: Rect,
}

impl AnchorRect {
    pub fn from_rect(rect: Rect) -> Self {
        Self {
            element: None,
            fallback: rect,
        }
    }

    pub fn from_element(element: GlobalElementId, fallback: Rect) -> Self {
        Self {
            element: Some(element),
            fallback,
        }
    }

    pub fn resolve<H: UiHost>(&self, app: &mut H, window: AppWindowId) -> Rect {
        self.element
            .and_then(|id| bounds_for_element(app, window, id))
            .unwrap_or(self.fallback)
    }
}

