pub const SOURCE: &str = include_str!("loading.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::Invalidation;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let last_action = super::last_action_model(cx);
    let query = cx.local_model_keyed("query", String::new);
    let loading_enabled = cx.local_model_keyed("loading_enabled", || false);
    let on_select = super::on_select_for_last_action(last_action.clone());
    let loading_enabled_value = cx
        .get_model_copied(&loading_enabled, Invalidation::Layout)
        .unwrap_or(false);

    ui::v_flex(|cx| {
        let entries: Vec<shadcn::CommandEntry> = if loading_enabled_value {
            vec![
                shadcn::CommandLoading::new("Fetching commands…")
                    .test_id("ui-gallery-command-loading-row")
                    .into(),
            ]
        } else {
            vec![
                shadcn::CommandGroup::new([
                    shadcn::CommandItem::new("Calendar")
                        .on_select_action(on_select(Arc::from("command.loading.calendar"))),
                    shadcn::CommandItem::new("Search Emoji")
                        .on_select_action(on_select(Arc::from("command.loading.search-emoji"))),
                ])
                .heading("Loaded items")
                .into(),
            ]
        };

        let toggle_row = ui::h_row(|cx| {
            vec![
                shadcn::Checkbox::new(loading_enabled.clone())
                    .control_id("command-loading-enabled")
                    .a11y_label("Loading (demo-only)")
                    .test_id("ui-gallery-command-loading-enabled")
                    .into_element(cx),
                shadcn::FieldLabel::new("Loading (demo-only)")
                    .for_control("command-loading-enabled")
                    .into_element(cx),
            ]
        })
        .gap(Space::N3)
        .items_center()
        .into_element(cx);

        vec![
            toggle_row,
            shadcn::CommandPalette::new(query.clone(), Vec::new())
                .placeholder("Type a command or search...")
                .a11y_label("Command loading demo")
                .entries(entries)
                .test_id_input("ui-gallery-command-loading-input")
                .list_test_id("ui-gallery-command-loading-listbox")
                .test_id_item_prefix("ui-gallery-command-loading-item-")
                .into_element(cx)
                .test_id("ui-gallery-command-loading"),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
