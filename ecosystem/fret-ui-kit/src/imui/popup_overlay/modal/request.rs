use crate::primitives::dismissable_layer::OnDismissRequest;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::{OverlayController, OverlayPresence, OverlayRequest};

pub(super) struct PopupModalOverlayTarget {
    pub(super) overlay_id: GlobalElementId,
    pub(super) root_name: String,
}

pub(super) struct PopupModalOverlayRequestInput {
    pub(super) target: PopupModalOverlayTarget,
    pub(super) trigger: Option<GlobalElementId>,
    pub(super) open: Model<bool>,
    pub(super) on_dismiss_request: OnDismissRequest,
    pub(super) layer: AnyElement,
    pub(super) initial_focus: Option<GlobalElementId>,
}

pub(super) fn popup_modal_overlay_target<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
) -> PopupModalOverlayTarget {
    let overlay_key = format!("fret-ui-kit.imui.popup_modal.overlay.{id}");
    let overlay_id = cx.named(overlay_key.as_str(), |cx| cx.root_id());
    let root_name = OverlayController::modal_root_name(overlay_id);
    PopupModalOverlayTarget {
        overlay_id,
        root_name,
    }
}

pub(super) fn request_popup_modal_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: PopupModalOverlayRequestInput,
) {
    let PopupModalOverlayRequestInput {
        target,
        trigger,
        open,
        on_dismiss_request,
        layer,
        initial_focus,
    } = input;

    let mut req = OverlayRequest::modal(
        target.overlay_id,
        trigger,
        open,
        OverlayPresence::instant(true),
        vec![layer],
    );
    req.root_name = Some(target.root_name);
    req.dismissible_on_dismiss_request = Some(on_dismiss_request);
    req.initial_focus = initial_focus;
    OverlayController::request(cx, req);
}
