use fret_ui::elements::GlobalElementId;

pub fn popover_root_name(id: GlobalElementId) -> String {
    format!("window-overlays.popover.{:x}", id.0)
}

pub fn modal_root_name(id: GlobalElementId) -> String {
    format!("window-overlays.modal.{:x}", id.0)
}

pub fn tooltip_root_name(id: GlobalElementId) -> String {
    format!("window-overlays.tooltip.{:x}", id.0)
}

pub fn hover_overlay_root_name(id: GlobalElementId) -> String {
    format!("window-overlays.hover-overlay.{:x}", id.0)
}

pub fn toast_layer_root_name(id: GlobalElementId) -> String {
    format!("window-overlays.toast-layer.{:x}", id.0)
}
