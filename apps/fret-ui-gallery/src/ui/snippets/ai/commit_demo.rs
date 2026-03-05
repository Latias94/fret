pub const SOURCE: &str = include_str!("commit_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let hash: Arc<str> = Arc::from("b4a1c0ffee");
    let hash_for_title = hash.clone();

    let header = ui_ai::CommitHeader::new([
        ui::h_row(move |cx| {
            vec![
                ui_ai::CommitHash::new(hash_for_title.clone()).into_element(cx),
                ui_ai::CommitMessage::new("Align mic selector popover width").into_element(cx),
            ]
        })
        .layout(LayoutRefinement::default().min_w_0())
        .gap(Space::N3)
        .items_center()
        .into_element(cx),
        ui_ai::CommitActions::new([ui_ai::CommitCopyButton::new(hash.clone())
            .test_id("ui-ai-commit-copy")
            .copied_marker_test_id("ui-ai-commit-copied-marker")
            .into_element(cx)])
        .into_element(cx),
    ])
    .test_id("ui-ai-commit-header");

    let content = ui_ai::CommitContent::new([
        ui_ai::CommitSeparator::new("Files").into_element(cx),
        ui_ai::CommitFiles::new([ui_ai::CommitFile::new([
            ui_ai::CommitFileInfo::new([
                ui_ai::CommitFileStatus::new(ui_ai::CommitFileStatusKind::Modified)
                    .into_element(cx),
                ui_ai::CommitFileIcon::default().into_element(cx),
                ui_ai::CommitFilePath::new("apps/fret-ui-gallery/src/ui/nav.rs").into_element(cx),
            ])
            .into_element(cx),
            ui_ai::CommitFileChanges::new([
                ui_ai::CommitFileAdditions::new(12).into_element(cx),
                ui_ai::CommitFileDeletions::new(3).into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx)])
        .into_element(cx),
    ])
    .test_id("ui-ai-commit-content");

    let commit = ui_ai::Commit::new(header, content)
        .default_open(false)
        .into_element(cx)
        .test_id("ui-ai-commit-root");

    ui::v_flex(move |cx| {
        vec![
            cx.text("Commit (AI Elements)"),
            cx.text("Disclosure surface: copy is independent of open state."),
            commit,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
