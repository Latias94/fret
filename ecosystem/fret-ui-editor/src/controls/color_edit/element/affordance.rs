use super::super::{ColorEditOptions, ColorEditPopupOptions};

pub(super) struct ColorEditFrameAffordances {
    pub(super) popup_has_visible_content: bool,
    pub(super) swatch_enabled: bool,
    pub(super) swatch_focusable: bool,
    pub(super) copy_enabled: bool,
    pub(super) drag_drop_enabled: bool,
}

pub(super) fn color_edit_frame_affordances(
    options: &ColorEditOptions,
    popup_options_for_frame: ColorEditPopupOptions,
    has_palette: bool,
    has_history: bool,
    has_eyedropper: bool,
) -> ColorEditFrameAffordances {
    let popup_has_visible_content = popup_options_for_frame.has_visible_content_with_swatches(
        options.show_alpha,
        has_palette,
        has_history,
    ) || has_eyedropper;
    let drag_drop_enabled = options.enabled && options.drag_drop.enabled;
    let tooltip_enabled = options.enabled && options.tooltip.enabled;
    let copy_enabled = options.enabled && options.copy.enabled;
    let eyedropper_enabled = options.enabled && has_eyedropper;
    let swatch_enabled = options.enabled
        && (popup_has_visible_content
            || drag_drop_enabled
            || tooltip_enabled
            || copy_enabled
            || eyedropper_enabled);
    let swatch_focusable = options.focusable
        && (popup_has_visible_content || drag_drop_enabled || copy_enabled || eyedropper_enabled);

    ColorEditFrameAffordances {
        popup_has_visible_content,
        swatch_enabled,
        swatch_focusable,
        copy_enabled,
        drag_drop_enabled,
    }
}
