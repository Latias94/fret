pub const SOURCE: &str = include_str!("file_tree_basic.rs");

// region: example
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_ai as ui_ai;
use fret_ui_kit::{IntoUiElement, LayoutRefinement, Space, ui};

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
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

pub fn preview<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
) -> impl IntoUiElement<H> + use<H> {
    let tree = render(cx);

    ui::v_flex(move |_cx| vec![tree])
        .gap(Space::N0)
        .layout(LayoutRefinement::default().w_full().min_w_0())
}
// endregion: example
