use std::sync::Arc;

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
