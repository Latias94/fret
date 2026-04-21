use std::cell::Cell;
use std::rc::Rc;

use fret_core::Modifiers;
use fret_ui::{ElementContext, UiHost};

pub(in super::super) struct DisabledScopeGuard {
    depth: Rc<Cell<u32>>,
    active: bool,
}

impl DisabledScopeGuard {
    pub(in super::super) fn push(depth: Rc<Cell<u32>>) -> Self {
        depth.set(depth.get().saturating_add(1));
        Self {
            depth,
            active: true,
        }
    }
}

impl Drop for DisabledScopeGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let v = self.depth.get();
        self.depth.set(v.saturating_sub(1));
    }
}

pub(in super::super) fn imui_is_disabled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> bool {
    super::disabled_scope_depth_for(cx).get() > 0
}

pub(in super::super) fn disabled_alpha_for<H: UiHost>(cx: &ElementContext<'_, H>) -> f32 {
    let theme = fret_ui::Theme::global(&*cx.app);
    let v = theme
        .number_by_key(crate::theme_tokens::number::COMPONENT_IMUI_DISABLED_ALPHA)
        .unwrap_or(super::super::DEFAULT_DISABLED_ALPHA);
    v.clamp(0.0, 1.0)
}

pub(in super::super) fn sanitize_response_for_enabled(
    enabled: bool,
    response: &mut super::super::ResponseExt,
) {
    response.enabled = enabled;
    if enabled {
        return;
    }
    response.activated = false;
    response.deactivated = false;
    response.edited = false;
    response.deactivated_after_edit = false;
    response.core.hovered = false;
    response.core.pressed = false;
    response.core.focused = false;
    response.core.clicked = false;
    response.core.changed = false;
    response.nav_highlighted = false;
    response.secondary_clicked = false;
    response.double_clicked = false;
    response.long_pressed = false;
    response.press_holding = false;
    response.context_menu_requested = false;
    response.context_menu_anchor = None;
    response.pointer_clicked = false;
    response.pointer_click_modifiers = Modifiers::default();
    response.drag = super::super::DragResponse::default();
}
