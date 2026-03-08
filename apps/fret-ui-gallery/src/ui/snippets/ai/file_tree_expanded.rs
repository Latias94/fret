pub const SOURCE: &str = include_str!("file_tree_expanded.rs");

// region: example
use fret_core::Px;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_ai as ui_ai;
use fret_ui_kit::{LayoutRefinement, Space, ui};

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui_ai::FileTree::new([ui_ai::FileTreeFolder::new("src", "src")
        .children([
            ui_ai::FileTreeFolder::new("src/components", "components")
                .children([
                    ui_ai::FileTreeFile::new("src/components/button.tsx", "button.tsx").into(),
                    ui_ai::FileTreeFile::new("src/components/input.tsx", "input.tsx").into(),
                ])
                .into(),
            ui_ai::FileTreeFile::new("src/index.ts", "index.ts").into(),
        ])
        .into()])
    .default_expanded_paths(["src", "src/components"])
    .test_id_root("ui-ai-file-tree-expanded-root")
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .max_h(Px(320.0)),
    )
    .into_element(cx)
}

pub fn preview<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(move |cx| vec![render(cx)])
        .gap(Space::N0)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}
// endregion: example
