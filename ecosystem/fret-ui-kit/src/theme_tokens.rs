pub mod metric {
    pub const COMPONENT_TEXT_XS_PX: &str = "component.text.xs_px";
    pub const COMPONENT_TEXT_XS_LINE_HEIGHT: &str = "component.text.xs_line_height";
    pub const COMPONENT_TEXT_SM_PX: &str = "component.text.sm_px";
    pub const COMPONENT_TEXT_SM_LINE_HEIGHT: &str = "component.text.sm_line_height";
    pub const COMPONENT_TEXT_BASE_PX: &str = "component.text.base_px";
    pub const COMPONENT_TEXT_BASE_LINE_HEIGHT: &str = "component.text.base_line_height";

    // Tailwind Typography (`prose`) defaults for body copy.
    pub const COMPONENT_TEXT_PROSE_PX: &str = "component.text.prose_px";
    pub const COMPONENT_TEXT_PROSE_LINE_HEIGHT: &str = "component.text.prose_line_height";

    /// Default spacing between consecutive `imui` items along the X axis (ImGui-style
    /// `ImGuiStyle::ItemSpacing.x`).
    pub const COMPONENT_IMUI_ITEM_SPACING_X_PX: &str = "component.imui.item_spacing_x_px";

    /// Default spacing between consecutive `imui` items along the Y axis (ImGui-style
    /// `ImGuiStyle::ItemSpacing.y`).
    pub const COMPONENT_IMUI_ITEM_SPACING_Y_PX: &str = "component.imui.item_spacing_y_px";

    /// Pointer drag threshold used by the `imui` facade before a drag session transitions to
    /// `dragging=true`.
    ///
    /// This aligns with Dear ImGui's `ImGuiIO::MouseDragThreshold` default (`6.0f`), but Fret's
    /// facade keeps its own fallback default unless the token is configured.
    pub const COMPONENT_IMUI_DRAG_THRESHOLD_PX: &str = "component.imui.drag_threshold_px";
}

pub mod number {
    /// Additional alpha multiplier applied by the `imui` facade's scoped disable helper.
    ///
    /// This aligns with Dear ImGui's `ImGuiStyle::DisabledAlpha` default (`0.60f`).
    pub const COMPONENT_IMUI_DISABLED_ALPHA: &str = "component.imui.disabled_alpha";
}
