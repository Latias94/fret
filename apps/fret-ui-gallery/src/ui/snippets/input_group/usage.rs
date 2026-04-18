pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let query = cx.local_model_keyed("query", String::new);

    shadcn::InputGroup::new(query)
        .a11y_label("Search")
        .placeholder("Search...")
        .trailing([icon::icon(cx, IconId::new_static("lucide.search"))])
        .into_element(cx)
        .test_id("ui-gallery-input-group-usage")
}
// endregion: example
