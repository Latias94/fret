use super::super::super::super::super::*;
use fret::AppComponentCx;
use fret::UiChild;

use super::OverlayModels;

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
    cx.text(text).test_id("ui-gallery-overlay-last-action")
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
                cx.text("Popover dismissed")
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
            Some(cx.text("Dialog open").test_id("ui-gallery-dialog-open"))
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
                cx.text("Dialog (Glass) open")
                    .test_id("ui-gallery-dialog-glass-open"),
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
                cx.text("AlertDialog open")
                    .test_id("ui-gallery-alert-dialog-open"),
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
    if let Some(flag) = alert_dialog_open_flag {
        out.push(flag);
    }
    out
}
