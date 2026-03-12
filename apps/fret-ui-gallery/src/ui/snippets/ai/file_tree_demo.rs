pub const SOURCE: &str = include_str!("file_tree_demo.rs");

// region: example
use fret_ui::action::ActionCx;
use fret_ui::element::{AnyElement, Length, SemanticsDecoration, SizeStyle, TextProps};
use fret_ui::{ElementContext, UiHost};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use std::sync::Arc;

fn invisible_marker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: &'static str,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: fret_ui::element::LayoutStyle {
            size: SizeStyle {
                width: Length::Px(fret_core::Px(0.0)),
                height: Length::Px(fret_core::Px(0.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        text: Arc::<str>::from(""),
        style: None,
        color: None,
        wrap: fret_core::TextWrap::None,
        overflow: fret_core::TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: Default::default(),
    })
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id(Arc::<str>::from(test_id)),
    )
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let selected = cx.local_model_keyed("selected", || None::<Arc<str>>);
    let selected_value = cx.watch_model(&selected).layout().cloned().flatten();

    let copied = cx.local_model_keyed("copied", || false);
    let copied_value = cx.watch_model(&copied).layout().copied().unwrap_or(false);

    let tree = ui_ai::FileTree::new([
        ui_ai::FileTreeFolder::new("src", "src")
            .test_id("ui-ai-file-tree-folder-src")
            .action(
                ui_ai::FileTreeAction::new(
                    fret_icons::ids::ui::COPY,
                    "Copy path",
                    Arc::new({
                        let copied = copied.clone();
                        move |host, _action_cx: ActionCx, _path| {
                            let _ = host.models_mut().update(&copied, |v| *v = true);
                        }
                    }),
                )
                .test_id("ui-ai-file-tree-folder-src-action-copy"),
            )
            .children([
                ui_ai::FileTreeFolder::new("src/components", "components")
                    .children([
                        ui_ai::FileTreeFile::new("src/components/button.tsx", "button.tsx").into(),
                        ui_ai::FileTreeFile::new("src/components/input.tsx", "input.tsx").into(),
                        ui_ai::FileTreeFile::new("src/components/modal.tsx", "modal.tsx").into(),
                    ])
                    .into(),
                ui_ai::FileTreeFolder::new("src/hooks", "hooks")
                    .children([
                        ui_ai::FileTreeFile::new("src/hooks/use-auth.ts", "use-auth.ts").into(),
                        ui_ai::FileTreeFile::new("src/hooks/use-theme.ts", "use-theme.ts").into(),
                    ])
                    .into(),
                ui_ai::FileTreeFolder::new("src/lib", "lib")
                    .children([ui_ai::FileTreeFile::new("src/lib/utils.ts", "utils.ts").into()])
                    .into(),
                ui_ai::FileTreeFile::new("src/app.tsx", "app.tsx").into(),
                ui_ai::FileTreeFile::new("src/main.tsx", "main.tsx")
                    .test_id("ui-ai-file-tree-file-main")
                    .into(),
                ui_ai::FileTreeFile::new("src/lib.rs", "lib.rs")
                    .test_id("ui-ai-file-tree-file-lib")
                    .into(),
            ])
            .into(),
        ui_ai::FileTreeFile::new("package.json", "package.json").into(),
        ui_ai::FileTreeFile::new("tsconfig.json", "tsconfig.json").into(),
        ui_ai::FileTreeFile::new("README.md", "README.md").into(),
    ])
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

    let mut out = vec![tree];
    if selected_value.as_deref() == Some("src/lib.rs") {
        out.push(invisible_marker(cx, "ui-ai-file-tree-selected-marker"));
    }
    if copied_value {
        out.push(invisible_marker(cx, "ui-ai-file-tree-action-marker"));
    }

    ui::v_flex(move |_cx| out)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N0)
        .into_element(cx)
}
// endregion: example
