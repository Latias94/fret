pub const SOURCE: &str = include_str!("responsive.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn status_items() -> Vec<shadcn::ComboboxItem> {
    vec![
        shadcn::ComboboxItem::new("backlog", "Backlog"),
        shadcn::ComboboxItem::new("todo", "Todo"),
        shadcn::ComboboxItem::new("in progress", "In Progress"),
        shadcn::ComboboxItem::new("done", "Done"),
        shadcn::ComboboxItem::new("canceled", "Canceled"),
    ]
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("responsive-value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("responsive-open", || false);
    let query = cx.local_model_keyed("responsive-query", String::new);

    shadcn::Combobox::new(value, open)
        .a11y_label("Status")
        .device_shell_responsive(true)
        .query_model(query)
        .test_id_prefix("ui-gallery-combobox-responsive")
        .trigger(
            shadcn::ComboboxTrigger::new()
                .variant(shadcn::ComboboxTriggerVariant::Button)
                .width_px(Px(150.0)),
        )
        .input(shadcn::ComboboxInput::new().placeholder("+ Set status"))
        .content(
            shadcn::ComboboxContent::new([
                shadcn::ComboboxContentPart::input(
                    shadcn::ComboboxInput::new().placeholder("Filter status..."),
                ),
                shadcn::ComboboxContentPart::empty(shadcn::ComboboxEmpty::new("No results found.")),
                shadcn::ComboboxContentPart::list(
                    shadcn::ComboboxList::new().items(status_items()),
                ),
            ])
            .width_px(Px(200.0))
            .test_id("ui-gallery-combobox-responsive-content"),
        )
        .into_element(cx)
}
// endregion: example
