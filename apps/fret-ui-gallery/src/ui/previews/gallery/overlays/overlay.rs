use super::super::super::super::*;

mod flags;
mod layout;
mod widgets;

#[derive(Clone)]
struct OverlayModels {
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
}

pub(in crate::ui) fn preview_overlay(
    cx: &mut ElementContext<'_, App>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let models = OverlayModels {
        popover_open,
        dialog_open,
        alert_dialog_open,
        sheet_open,
        portal_geometry_popover_open,
        dropdown_open,
        context_menu_open,
        context_menu_edge_open,
        last_action,
    };

    let last_action_status = flags::last_action_status(cx, &models);

    let overlays = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), {
        let models = models.clone();
        move |cx| {
            let widgets = widgets::build(cx, &models);
            vec![layout::compose_body(cx, widgets)]
        }
    });

    let mut out: Vec<AnyElement> = vec![overlays, last_action_status];
    out.extend(flags::status_flags(cx, &models));
    out
}
