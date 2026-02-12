use super::super::super::super::super::*;

use super::OverlayModels;

pub(super) fn last_action_status(
    cx: &mut ElementContext<'_, App>,
    models: &OverlayModels,
) -> AnyElement {
    let last = cx
        .app
        .models()
        .get_cloned(&models.last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let text = format!("last action: {last}");
    cx.text(text).test_id("ui-gallery-overlay-last-action")
}

pub(super) fn status_flags(
    cx: &mut ElementContext<'_, App>,
    models: &OverlayModels,
) -> Vec<AnyElement> {
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
    if let Some(flag) = alert_dialog_open_flag {
        out.push(flag);
    }
    out
}
