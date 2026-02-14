//! XyFlow-style input exemption helpers for editor canvases.
//!
//! In XyFlow/ReactFlow, elements can opt-out of canvas pan/zoom behavior using CSS classes like
//! `.nowheel` and `.nopan`. In Fret, the recommended mechanism is explicit pointer regions layered
//! above the pan/zoom surface that consume events before they bubble to the canvas substrate.

use std::sync::Arc;

use fret_core::MouseButton;
use fret_ui::action::{OnPinchGesture, OnPointerDown, OnWheel};
use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone)]
pub struct CanvasInputExemptRegionProps {
    pub pointer_region: PointerRegionProps,

    /// Consume wheel events within this region (`.nowheel` equivalent).
    pub block_wheel: bool,
    /// Consume pinch gesture events within this region.
    pub block_pinch: bool,
    /// Consume pan start (pointer down) for the given button (`.nopan` equivalent for MMB-drag pans).
    pub block_pan_button: Option<MouseButton>,
}

impl Default for CanvasInputExemptRegionProps {
    fn default() -> Self {
        Self {
            pointer_region: PointerRegionProps::default(),
            block_wheel: true,
            block_pinch: true,
            block_pan_button: Some(MouseButton::Middle),
        }
    }
}

impl CanvasInputExemptRegionProps {
    pub fn nowheel(mut self, block: bool) -> Self {
        self.block_wheel = block;
        self
    }

    pub fn nopan_middle(mut self, block: bool) -> Self {
        self.block_pan_button = block.then_some(MouseButton::Middle);
        self
    }
}

/// A pointer region that blocks pan/zoom input bubbling from the canvas substrate.
///
/// Intended usage:
/// - mount above a pan/zoom canvas surface (e.g. overlay chrome panels),
/// - set `block_wheel` to prevent wheel-driven pan/zoom while hovering the region,
/// - set `block_pan_button` to prevent middle-drag pan capture within the region.
#[track_caller]
pub fn canvas_input_exempt_region<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: CanvasInputExemptRegionProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let block_wheel = props.block_wheel;
    let block_pinch = props.block_pinch;
    let block_pan_button = props.block_pan_button;

    cx.pointer_region(props.pointer_region, move |cx| {
        if block_wheel {
            let on_wheel: OnWheel = Arc::new(move |_host, _cx, _wheel| true);
            cx.pointer_region_on_wheel(on_wheel);
        }

        if block_pinch {
            let on_pinch: OnPinchGesture = Arc::new(move |_host, _cx, _pinch| true);
            cx.pointer_region_on_pinch_gesture(on_pinch);
        }

        if let Some(btn) = block_pan_button {
            let on_down: OnPointerDown = Arc::new(move |_host, _cx, down| down.button == btn);
            cx.pointer_region_add_on_pointer_down(on_down);
        }

        f(cx)
    })
}
