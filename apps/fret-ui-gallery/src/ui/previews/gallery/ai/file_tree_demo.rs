use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_file_tree_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::collections::HashSet;
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::action::ActionCx;
    use fret_ui::element::SemanticsProps;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Default)]
    struct FileTreeModels {
        expanded: Option<Model<HashSet<Arc<str>>>>,
        selected: Option<Model<Option<Arc<str>>>>,
    }

    let expanded = cx.with_state(FileTreeModels::default, |st| st.expanded.clone());
    let expanded = match expanded {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(HashSet::<Arc<str>>::new());
            cx.with_state(FileTreeModels::default, |st| {
                st.expanded = Some(model.clone())
            });
            model
        }
    };

    let selected = cx.with_state(FileTreeModels::default, |st| st.selected.clone());
    let selected = match selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(FileTreeModels::default, |st| {
                st.selected = Some(model.clone())
            });
            model
        }
    };

    let selected_value = cx.watch_model(&selected).layout().cloned().flatten();

    let tree = ui_ai::FileTree::new([
        ui_ai::FileTreeFolder::new("src", "src")
            .test_id("ui-ai-file-tree-folder-src")
            .children([
                ui_ai::FileTreeFile::new("src/lib.rs", "lib.rs")
                    .test_id("ui-ai-file-tree-file-lib")
                    .into(),
                ui_ai::FileTreeFile::new("src/main.rs", "main.rs")
                    .test_id("ui-ai-file-tree-file-main")
                    .into(),
            ])
            .into(),
        ui_ai::FileTreeFile::new("Cargo.toml", "Cargo.toml")
            .test_id("ui-ai-file-tree-file-cargo-toml")
            .into(),
        ui_ai::FileTreeFolder::new("tests", "tests")
            .test_id("ui-ai-file-tree-folder-tests")
            .child(
                ui_ai::FileTreeFile::new("tests/file_tree.rs", "file_tree.rs")
                    .test_id("ui-ai-file-tree-file-tests-file-tree"),
            )
            .into(),
    ])
    .expanded_paths(expanded.clone())
    .selected_path(selected_value.clone())
    .on_select(Arc::new({
        let selected = selected.clone();
        move |host, _action_cx: ActionCx, path| {
            let _ = host.models_mut().update(&selected, |v| *v = Some(path));
        }
    }))
    .test_id_root("ui-ai-file-tree-root")
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let selected_label_text = selected_value
        .as_deref()
        .map(|s| format!("Selected: {s}"))
        .unwrap_or_else(|| "Selected: <none>".to_string());

    let selected_label = cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Text,
            test_id: Some(Arc::<str>::from("ui-ai-file-tree-selected-label")),
            ..Default::default()
        },
        move |cx| vec![cx.text(selected_label_text)],
    );

    let selected_marker = (selected_value.as_deref() == Some("src/lib.rs")).then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Generic,
                test_id: Some(Arc::<str>::from("ui-ai-file-tree-selected-marker")),
                ..Default::default()
            },
            move |_cx| vec![],
        )
    });

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3),
        move |cx| {
            vec![
                cx.text("FileTree (AI Elements)"),
                tree,
                selected_label,
                selected_marker.unwrap_or_else(|| cx.text("")),
            ]
        },
    )]
}
