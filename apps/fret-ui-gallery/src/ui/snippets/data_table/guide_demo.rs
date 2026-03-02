// region: example
use fret_app::App;
use fret_ui_headless::table::TableState;
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut ElementContext<'_, App>, state: Model<TableState>) -> AnyElement {
    let legacy_content = crate::ui::preview_data_table_legacy(cx, state);
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| legacy_content,
    )
    .test_id("ui-gallery-data-table-guide-demo")
}
// endregion: example
