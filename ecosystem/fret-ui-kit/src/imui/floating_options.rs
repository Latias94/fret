//! Public floating-surface facade types owned separately from the root `imui.rs` hub.

use std::sync::Arc;

use fret_core::{Point, Px, Size};
use fret_ui::GlobalElementId;

#[derive(Debug, Clone, Copy)]
pub struct FloatingWindowResizeOptions {
    pub min_size: Size,
    pub max_size: Option<Size>,
}

impl Default for FloatingWindowResizeOptions {
    fn default() -> Self {
        Self {
            min_size: Size::new(Px(120.0), Px(72.0)),
            max_size: None,
        }
    }
}

/// Behavior flags for in-window floating windows.
///
/// This is an ecosystem-level facade surface (not a mechanism-layer contract). The goal is to
/// provide ImGui-like control over common floating window behavior without introducing a parallel
/// runtime or duplicating canonical policy.
#[derive(Debug, Clone, Copy)]
pub struct FloatingWindowOptions {
    /// When true, the window can be moved by dragging the title bar.
    pub movable: bool,
    /// When true, resize handles are active when the window is rendered with an initial size.
    pub resizable: bool,
    /// When true, title-bar double click toggles collapse/expand.
    pub collapsible: bool,
    /// When true and an `open` model is provided, the close button and `Escape`-to-close are enabled.
    pub closable: bool,
    /// When true, pointer down inside the window requests focus for the surface (even if
    /// activation is disabled).
    ///
    /// This is useful to model ImGui's `NoBringToFrontOnFocus` behavior: you may want a window to
    /// take focus when clicked without also being activated for z-order.
    pub focus_on_click: bool,
    /// When true, pointer down anywhere in the window activates it for z-order (when nested under
    /// `floating_layer(...)`).
    pub activate_on_click: bool,
    /// When false, the window is rendered but pointer interactions are blocked (no activation,
    /// drag, resize, or child clicks).
    pub inputs_enabled: bool,
    /// When true, the window is rendered but is inert for pointer and keyboard navigation:
    /// it does not participate in pointer hit-testing and is skipped by focus traversal.
    ///
    /// This is intended to model Dear ImGui's `NoInputs` window flag, which implies mouse
    /// pass-through and disables nav/focus participation.
    ///
    /// Note: `no_inputs=true` is different from `inputs_enabled=false`:
    /// - `inputs_enabled=false` blocks pointer hits (not click-through) but still participates
    ///   in focus traversal.
    /// - `no_inputs=true` is click-through and is skipped by focus traversal.
    pub no_inputs: bool,
    /// When true, the floating window is hit-test transparent (pointer events pass through to
    /// underlay content).
    ///
    /// This is intended to model Dear ImGui's "mouse pass-through" style behavior (`NoMouseInputs`
    /// for in-window floating surfaces. In Fret's current facade, this is pointer pass-through
    /// only: the subtree remains present for focus traversal / keyboard navigation.
    ///
    /// Note: `inputs_enabled=false` is *not* click-through; it is "non-interactive but blocks
    /// pointer hits". Use `pointer_passthrough=true` when you explicitly want click-through.
    pub pointer_passthrough: bool,
}

impl Default for FloatingWindowOptions {
    fn default() -> Self {
        Self {
            movable: true,
            resizable: true,
            collapsible: true,
            closable: true,
            focus_on_click: true,
            activate_on_click: true,
            inputs_enabled: true,
            no_inputs: false,
            pointer_passthrough: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WindowOptions {
    /// Optional `open` model controlling whether the window is rendered.
    ///
    /// When present, close actions update the model to `false`.
    pub open: Option<fret_runtime::Model<bool>>,
    /// Optional fixed initial size for the floating window.
    ///
    /// When absent, the window uses content-driven sizing and `resize` is ignored.
    pub size: Option<Size>,
    /// Optional resize policy for sized windows.
    ///
    /// This only takes effect when `size` is also set.
    pub resize: Option<FloatingWindowResizeOptions>,
    /// Behavior flags for the floating window surface.
    pub behavior: FloatingWindowOptions,
}

impl WindowOptions {
    pub fn with_open(mut self, open: &fret_runtime::Model<bool>) -> Self {
        self.open = Some(open.clone());
        self
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_resize(mut self, resize: FloatingWindowResizeOptions) -> Self {
        self.resize = Some(resize);
        self
    }

    pub fn with_behavior(mut self, behavior: FloatingWindowOptions) -> Self {
        self.behavior = behavior;
        self
    }
}

#[derive(Debug, Clone)]
pub struct FloatingAreaOptions {
    /// A stable semantics test-id prefix used when `test_id` is not provided.
    ///
    /// The final test id is `{test_id_prefix}{id}`.
    pub test_id_prefix: &'static str,
    /// Explicitly overrides the semantics test-id for the floating area root element.
    pub test_id: Option<Arc<str>>,
    /// When true, the floating area root is hit-test transparent (pointer events pass through).
    ///
    /// This is a facade-level policy knob intended for click-through / pass-through floating
    /// surfaces. It wraps the area in a `HitTestGate` so the subtree does not intercept pointer
    /// input while still allowing focus traversal.
    pub hit_test_passthrough: bool,
    /// When true, the floating area is rendered but is inert for pointer and focus traversal:
    /// it is click-through and skipped by focus traversal.
    ///
    /// This wraps the area in an `InteractivityGate(present=true, interactive=false)` to model
    /// ImGui-style `NoInputs` behavior.
    ///
    /// Precedence: when `no_inputs == true`, `hit_test_passthrough` is ignored.
    pub no_inputs: bool,
}

impl Default for FloatingAreaOptions {
    fn default() -> Self {
        Self {
            test_id_prefix: "imui.float_area.area:",
            test_id: None,
            hit_test_passthrough: false,
            no_inputs: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FloatingAreaContext {
    pub id: GlobalElementId,
    pub position: Point,
    pub drag_kind: fret_runtime::DragKindId,
}
