use super::*;

mod alert;
mod alert_dialog;
mod aspect_ratio;
mod breadcrumb;
mod checkbox;
mod carousel;
mod chart;
mod collapsible;
mod combobox;
mod command;
mod context_menu;
mod data_table;
mod date_picker;
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

pub(super) fn preview_breadcrumb(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    breadcrumb::preview_breadcrumb(cx, last_action)
}

pub(super) fn preview_checkbox(cx: &mut ElementContext<'_, App>, model: Model<bool>) -> Vec<AnyElement> {
    checkbox::preview_checkbox(cx, model)
}

pub(super) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    carousel::preview_carousel(cx)
}

pub(super) fn preview_chart(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    chart::preview_chart(cx)
}

pub(super) fn preview_collapsible(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    collapsible::preview_collapsible(cx)
}

pub(super) fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    combobox::preview_combobox(cx, value, open, query)
}

pub(super) fn preview_command_palette(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    query: Model<String>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    command::preview_command_palette(cx, open, query, last_action)
}

pub(super) fn preview_context_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    context_menu::preview_context_menu(cx, open, last_action)
}

pub(super) fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    data_table::preview_data_table(cx, state)
}
pub(super) fn preview_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    date_picker::preview_date_picker(cx, open, month, selected)
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
