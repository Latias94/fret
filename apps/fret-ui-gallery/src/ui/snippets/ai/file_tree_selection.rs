pub const SOURCE: &str = include_str!("file_tree_selection.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::action::ActionCx;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::{LayoutRefinement, Space, ui};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let selected = cx.local_model_keyed("selected", || None::<Arc<str>>);
    let selected_value = cx.watch_model(&selected).layout().cloned().flatten();

    ui_ai::FileTree::empty()
        .child(
            ui_ai::FileTreeFolder::new("src", "src")
                .child(ui_ai::FileTreeFile::new("src/app.tsx", "app.tsx"))
                .child(ui_ai::FileTreeFile::new("src/index.ts", "index.ts")),
        )
        .child(ui_ai::FileTreeFile::new("package.json", "package.json"))
        .selected_path(selected_value)
        .on_select(Arc::new({
            let selected = selected.clone();
            move |host, _action_cx: ActionCx, path| {
                let _ = host
                    .models_mut()
                    .update(&selected, |value| *value = Some(path));
            }
        }))
        .test_id_root("ui-ai-file-tree-selection-root")
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}

pub fn preview(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let tree = render(cx);

    ui::v_flex(move |_cx| vec![tree])
        .gap(Space::N0)
        .layout(LayoutRefinement::default().w_full().min_w_0())
}
// endregion: example
