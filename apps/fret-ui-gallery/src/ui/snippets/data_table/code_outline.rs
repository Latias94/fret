pub const SOURCE: &str = include_str!("code_outline.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![shadcn::raw::typography::muted(
            "Reference outline only: the default recipe lives above, and these snippets map the denser advanced guide-aligned surface.",
        ).into_element(cx)]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}

// Basic Table (guide outline)
//
// let table = shadcn::DataTable::new()
//     .row_height(Px(40.0))
//     .header_height(Px(40.0))
//     .refine_layout(LayoutRefinement::default().w_full().h_px(Px(280.0)))
//     .into_element(cx, data, 1, state, columns, row_key, col_key, render_cell, render_header);
//
// // State + Sorting
// let selected_count = models.read(&state, |st| st.row_selection.len())?;
// let sorting = models.read(&state, |st| st.sorting.first().cloned())?;
//
// // show selection/sorting summaries in a deterministic status row
//
// // Docs Gap Markers
// section_card("Filtering", Alert::new([...]))
// section_card("Visibility", Alert::new([...]))
// // keep unsupported guide sections explicit so parity work is traceable
// endregion: example
