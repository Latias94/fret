pub const SOURCE: &str = include_str!("file_tree_basic.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_kit::{LayoutRefinement, Space, ui};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui_ai::FileTree::new([
        ui_ai::FileTreeFolder::new("src", "src")
            .child(ui_ai::FileTreeFile::new("src/index.ts", "index.ts"))
            .into(),
        ui_ai::FileTreeFile::new("package.json", "package.json").into(),
    ])
    .test_id_root("ui-ai-file-tree-basic-root")
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}

pub fn preview(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let tree = render(cx);

    ui::v_flex(move |_cx| vec![tree])
        .gap(Space::N0)
        .layout(LayoutRefinement::default().w_full().min_w_0())
}
// endregion: example
