use super::super::super::super::super::*;
use fret::AppComponentCx;
use fret::UiChild;

use super::OverlayModels;

fn overlay_status_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    doc_layout::control_readout_text(cx, text)
}

// Typed status helper: the label still reads preview-local models and attaches test ids, but the
// landing stays explicit at the preview result-vector seam.
pub(super) fn last_action_status(
    cx: &mut AppComponentCx<'_>,
    models: &OverlayModels,
) -> impl UiChild + use<> {
    let last = cx
        .app
        .models()
        .get_cloned(&models.last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let text = format!("last action: {last}");
    overlay_status_text(cx, text).test_id("ui-gallery-overlay-last-action")
}

// Intentional raw boundary: these flags are appended directly onto the overlay preview's concrete
// result vector after model reads and conditional visibility have completed.
pub(super) fn status_flags(cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> Vec<AnyElement> {
    let popover_dismissed_flag = {
        let last = cx
            .get_model_cloned(&models.last_action, Invalidation::Layout)
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        if last.as_ref() == "popover:dismissed" {
            Some(
                overlay_status_text(cx, "Popover dismissed")
                    .test_id("ui-gallery-popover-dismissed"),
            )
        } else {
            None
        }
    };

    let dialog_open_flag = {
        let open = cx
            .get_model_copied(&models.dialog_open, Invalidation::Layout)
            .unwrap_or(false);
        if open {
            Some(overlay_status_text(cx, "Dialog open").test_id("ui-gallery-dialog-open"))
        } else {
            None
        }
    };

    let dialog_glass_open_flag = {
        let open = cx
            .get_model_copied(&models.dialog_glass_open, Invalidation::Layout)
            .unwrap_or(false);
        if open {
            Some(
                overlay_status_text(cx, "Dialog (Glass) open")
                    .test_id("ui-gallery-dialog-glass-open"),
            )
        } else {
            None
        }
    };

    let underlay_activated_flag = {
        let activated = cx
            .get_model_copied(&models.underlay_activated, Invalidation::Layout)
            .unwrap_or(false);
        if activated {
            Some(
                overlay_status_text(cx, "Underlay activated")
                    .test_id("ui-gallery-overlay-underlay-activated"),
            )
        } else {
            None
        }
    };

    let alert_dialog_open_flag = {
        let open = cx
            .get_model_copied(&models.alert_dialog_open, Invalidation::Layout)
            .unwrap_or(false);
        if open {
            Some(
                overlay_status_text(cx, "AlertDialog open").test_id("ui-gallery-alert-dialog-open"),
            )
        } else {
            None
        }
    };

    let mut out = Vec::new();
    if let Some(flag) = popover_dismissed_flag {
        out.push(flag);
    }
    if let Some(flag) = dialog_open_flag {
        out.push(flag);
    }
    if let Some(flag) = dialog_glass_open_flag {
        out.push(flag);
    }
    if let Some(flag) = underlay_activated_flag {
        out.push(flag);
    }
    if let Some(flag) = alert_dialog_open_flag {
        out.push(flag);
    }
    out
}
