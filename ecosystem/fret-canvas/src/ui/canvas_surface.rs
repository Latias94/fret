//! Canvas surface helpers.
//!
//! This module provides a small declarative wrapper that:
//! - wires `PointerRegion` input hooks around a leaf `Canvas`,
//! - keeps the "mechanism vs. policy" boundary explicit (ADR 0141).
//!
//! Policy-heavy behaviors like pan/zoom, tool modes, selection, and snapping should be layered
//! above this helper (e.g. in `crate::ui`), not embedded here.

use fret_ui::action::{OnPinchGesture, OnPointerDown, OnPointerMove, OnPointerUp, OnWheel};
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{AnyElement, CanvasProps, Length, PointerRegionProps};
use fret_ui::{ElementContext, UiHost};

/// Props for [`canvas_surface_panel`].
///
/// Note: this intentionally carries optional action hooks (which are not `Debug`) because this
/// wrapper is purely wiring-focused.
#[derive(Clone)]
pub struct CanvasSurfacePanelProps {
    pub pointer_region: PointerRegionProps,
    pub canvas: CanvasProps,
    pub on_pointer_down: Option<OnPointerDown>,
    pub on_pointer_move: Option<OnPointerMove>,
    pub on_pointer_up: Option<OnPointerUp>,
    pub on_wheel: Option<OnWheel>,
    pub on_pinch_gesture: Option<OnPinchGesture>,
}

impl Default for CanvasSurfacePanelProps {
    fn default() -> Self {
        let mut pointer_region = PointerRegionProps::default();
        pointer_region.layout.size.width = Length::Fill;
        pointer_region.layout.size.height = Length::Fill;

        Self {
            pointer_region,
            canvas: CanvasProps::default(),
            on_pointer_down: None,
            on_pointer_move: None,
            on_pointer_up: None,
            on_wheel: None,
            on_pinch_gesture: None,
        }
    }
}

#[track_caller]
pub fn canvas_surface_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: CanvasSurfacePanelProps,
    paint: impl for<'p> Fn(&mut CanvasPainter<'p>) + 'static,
) -> AnyElement {
    let CanvasSurfacePanelProps {
        pointer_region,
        canvas,
        on_pointer_down,
        on_pointer_move,
        on_pointer_up,
        on_wheel,
        on_pinch_gesture,
    } = props;

    cx.pointer_region(pointer_region, move |cx| {
        if let Some(on_pointer_down) = on_pointer_down {
            cx.pointer_region_on_pointer_down(on_pointer_down);
        }
        if let Some(on_pointer_move) = on_pointer_move {
            cx.pointer_region_on_pointer_move(on_pointer_move);
        }
        if let Some(on_pointer_up) = on_pointer_up {
            cx.pointer_region_on_pointer_up(on_pointer_up);
        }
        if let Some(on_wheel) = on_wheel {
            cx.pointer_region_on_wheel(on_wheel);
        }
        if let Some(on_pinch_gesture) = on_pinch_gesture {
            cx.pointer_region_on_pinch_gesture(on_pinch_gesture);
        }

        vec![cx.canvas(canvas, paint)]
    })
}
