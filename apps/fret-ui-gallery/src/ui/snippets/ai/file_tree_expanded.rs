pub const SOURCE: &str = include_str!("file_tree_expanded.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_ai as ui_ai;
use fret_ui_kit::{LayoutRefinement, Space, ui};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
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

pub fn preview(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let tree = render(cx);

    ui::v_flex(move |_cx| vec![tree])
        .gap(Space::N0)
        .layout(LayoutRefinement::default().w_full().min_w_0())
}
// endregion: example
