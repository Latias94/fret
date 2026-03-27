pub const SOURCE: &str = include_str!("groups.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);

    let combo = shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox groups")
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-groups")
        .groups([
            shadcn::ComboboxGroup::new()
                .label(shadcn::ComboboxLabel::new("Americas"))
                .items([
                    shadcn::ComboboxItem::new("americas-ny", "(GMT-5) New York"),
                    shadcn::ComboboxItem::new("americas-la", "(GMT-8) Los Angeles"),
                    shadcn::ComboboxItem::new("americas-chi", "(GMT-6) Chicago"),
                ]),
            shadcn::ComboboxGroup::new()
                .label(shadcn::ComboboxLabel::new("Europe"))
                .items([
                    shadcn::ComboboxItem::new("europe-lon", "(GMT+0) London"),
                    shadcn::ComboboxItem::new("europe-paris", "(GMT+1) Paris"),
                    shadcn::ComboboxItem::new("europe-berlin", "(GMT+1) Berlin"),
                ]),
            shadcn::ComboboxGroup::new()
                .label(shadcn::ComboboxLabel::new("Asia/Pacific"))
                .items([
                    shadcn::ComboboxItem::new("asia-tokyo", "(GMT+9) Tokyo"),
                    shadcn::ComboboxItem::new("asia-shanghai", "(GMT+8) Shanghai"),
                    shadcn::ComboboxItem::new("asia-singapore", "(GMT+8) Singapore"),
                ]),
        ])
        .trigger(shadcn::ComboboxTrigger::new().width_px(Px(300.0)))
        .input(shadcn::ComboboxInput::new().placeholder("Select a timezone"))
        .into_element(cx);

    combo
}
// endregion: example
