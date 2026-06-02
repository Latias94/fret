use fret_core::{Color, Px, Rect, Size};

pub(in crate::imui::popup_overlay::modal) struct PopupModalPalette {
    pub(in crate::imui::popup_overlay::modal) popover: Color,
    pub(in crate::imui::popup_overlay::modal) border: Color,
    pub(in crate::imui::popup_overlay::modal) dim: Color,
}

#[derive(Clone, Copy)]
pub(in crate::imui::popup_overlay::modal) struct PopupModalPanelLayout {
    pub(in crate::imui::popup_overlay::modal) left: Px,
    pub(in crate::imui::popup_overlay::modal) top: Px,
    pub(in crate::imui::popup_overlay::modal) size: Size,
}

pub(in crate::imui::popup_overlay::modal) fn popup_modal_palette(
    theme: &fret_ui::Theme,
) -> PopupModalPalette {
    PopupModalPalette {
        popover: theme.color_token("popover"),
        border: theme.color_token("border"),
        dim: Color {
            a: 0.4,
            ..Color::from_srgb_hex_rgb(0x00_00_00)
        },
    }
}

pub(in crate::imui::popup_overlay::modal) fn centered_panel_layout(
    bounds: Rect,
    size: Size,
) -> PopupModalPanelLayout {
    PopupModalPanelLayout {
        left: Px(bounds.origin.x.0 + (bounds.size.width.0 - size.width.0).max(0.0) * 0.5),
        top: Px(bounds.origin.y.0 + (bounds.size.height.0 - size.height.0).max(0.0) * 0.5),
        size,
    }
}
