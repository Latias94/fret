use super::*;

mod alert;
mod alert_dialog;
mod aspect_ratio;
mod toggle;
mod toggle_group;
mod tooltip;
mod typography;

pub(super) fn preview_alert(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    alert::preview_alert(cx)
}

pub(super) fn preview_alert_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    alert_dialog::preview_alert_dialog(cx, open)
}

pub(super) fn preview_aspect_ratio(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    aspect_ratio::preview_aspect_ratio(cx)
}

pub(super) fn preview_toggle(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    toggle::preview_toggle(cx)
}

pub(super) fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    toggle_group::preview_toggle_group(cx)
}

pub(super) fn preview_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    tooltip::preview_tooltip(cx)
}

pub(super) fn preview_typography(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    typography::preview_typography(cx)
}

pub(super) fn render_component_page_tabs(
    cx: &mut ElementContext<'_, App>,
    test_id_prefix: &'static str,
    component_panel: AnyElement,
    code_panel: AnyElement,
    notes_panel: AnyElement,
) -> Vec<AnyElement> {
    let tabs = shadcn::Tabs::uncontrolled(Some("component"))
        .refine_layout(LayoutRefinement::default().w_full())
        .items([
            shadcn::TabsItem::new("component", "Component", [component_panel]),
            shadcn::TabsItem::new("code", "Code", [code_panel]),
            shadcn::TabsItem::new("notes", "Notes", [notes_panel]),
        ])
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id(format!("{test_id_prefix}-tabs")));

    vec![tabs]
}
