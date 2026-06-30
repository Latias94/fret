use std::sync::Arc;

use fret_runtime::Model;

use crate::primitives::dismissable_layer::{DismissReason, DismissRequestCx, OnDismissRequest};

pub(super) fn modal_dismiss_request(
    open: Model<bool>,
    close_on_outside_press: bool,
) -> OnDismissRequest {
    Arc::new(
        move |host, acx, req: &mut DismissRequestCx| match req.reason {
            DismissReason::Escape => {
                let _ = host.models_mut().update(&open, |v| *v = false);
                host.notify(acx);
            }
            DismissReason::OutsidePress { .. } if close_on_outside_press => {
                let _ = host.models_mut().update(&open, |v| *v = false);
                host.notify(acx);
            }
            _ => {
                req.prevent_default();
            }
        },
    )
}
