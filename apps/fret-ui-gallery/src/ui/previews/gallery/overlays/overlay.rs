use super::super::super::super::*;
use fret::AppComponentCx;

mod flags;
mod layout;
mod widgets;

#[derive(Clone)]
struct OverlayModels {
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    dialog_glass_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
    underlay_activated: Model<bool>,
}

pub(in crate::ui) fn preview_overlay(
    cx: &mut AppComponentCx<'_>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    dialog_glass_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    preview_overlay_with_row_wrap(
        cx,
        popover_open,
        dialog_open,
        dialog_glass_open,
        alert_dialog_open,
        sheet_open,
        portal_geometry_popover_open,
        dropdown_open,
        context_menu_open,
        context_menu_edge_open,
        last_action,
        true,
    )
}

pub(in crate::ui) fn preview_overlay_fixed_rows(
    cx: &mut AppComponentCx<'_>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    dialog_glass_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    preview_overlay_with_row_wrap(
        cx,
        popover_open,
        dialog_open,
        dialog_glass_open,
        alert_dialog_open,
        sheet_open,
        portal_geometry_popover_open,
        dropdown_open,
        context_menu_open,
        context_menu_edge_open,
        last_action,
        false,
    )
}

fn preview_overlay_with_row_wrap(
    cx: &mut AppComponentCx<'_>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    dialog_glass_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
    wrap_rows: bool,
) -> Vec<AnyElement> {
    let underlay_activated = cx.local_model_keyed("overlay_underlay_activated", || false);

    // Intentional raw boundary: this internal preview still assembles cached overlay roots plus
    // status indicators as a concrete result vector after typed helpers land at the cache/vector
    // seam.
    let models = OverlayModels {
        popover_open,
        dialog_open,
        dialog_glass_open,
        alert_dialog_open,
        sheet_open,
        portal_geometry_popover_open,
        dropdown_open,
        context_menu_open,
        context_menu_edge_open,
        last_action,
        underlay_activated,
    };

    let last_action_status = flags::last_action_status(cx, &models).into_element(cx);

    let overlays = cx.cached_subtree_with(
        CachedSubtreeProps::default().contain_layout_when_bounds_known(true),
        {
            let models = models.clone();
            move |cx| {
                if wrap_rows {
                    vec![layout::compose_body(cx, models.clone()).into_element(cx)]
                } else {
                    vec![layout::compose_body_fixed_rows(cx, models.clone()).into_element(cx)]
                }
            }
        },
    );

    let mut out: Vec<AnyElement> = vec![overlays, last_action_status];
    out.extend(flags::status_flags(cx, &models));
    out
}
