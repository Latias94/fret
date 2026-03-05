pub const SOURCE: &str = include_str!("commit_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let hash: Arc<str> = Arc::from("a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0");
    let short_hash: Arc<str> = Arc::from(hash.chars().take(7).collect::<String>());
    let timestamp = SystemTime::now()
        .checked_sub(Duration::from_secs(60 * 60 * 2))
        .unwrap_or_else(SystemTime::now);

    let header = ui_ai::CommitHeader::new([
        ui_ai::CommitAuthor::new([ui_ai::CommitAuthorAvatar::new("HB").into_element(cx)])
            .into_element(cx),
        ui_ai::CommitInfo::new([
            ui_ai::CommitMessage::new("feat: Add user authentication flow").into_element(cx),
            ui_ai::CommitMetadata::new([
                ui_ai::CommitHash::new(short_hash).into_element(cx),
                ui_ai::CommitSeparator::default().into_element(cx),
                ui_ai::CommitTimestamp::new(timestamp).into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx),
        ui_ai::CommitActions::new([ui_ai::CommitCopyButton::new(hash.clone())
            .on_copy(Arc::new(|host, action_cx| {
                host.notify(action_cx);
            }))
            .test_id("ui-ai-commit-copy")
            .copied_marker_test_id("ui-ai-commit-copied-marker")
            .into_element(cx)])
        .into_element(cx),
    ])
    .test_id("ui-ai-commit-header");

    let files = [
        (
            150u32,
            0u32,
            "src/auth/login.tsx",
            ui_ai::CommitFileStatusKind::Added,
        ),
        (
            45u32,
            0u32,
            "src/auth/logout.tsx",
            ui_ai::CommitFileStatusKind::Added,
        ),
        (
            23u32,
            8u32,
            "src/lib/session.ts",
            ui_ai::CommitFileStatusKind::Modified,
        ),
    ];

    let content = ui_ai::CommitContent::new([ui_ai::CommitFiles::new(files.into_iter().map(
        |(add, del, path, status)| {
            ui_ai::CommitFile::new([
                ui_ai::CommitFileInfo::new([
                    ui_ai::CommitFileStatus::new(status).into_element(cx),
                    ui_ai::CommitFileIcon::default().into_element(cx),
                    ui_ai::CommitFilePath::new(path).into_element(cx),
                ])
                .into_element(cx),
                ui_ai::CommitFileChanges::new([
                    ui_ai::CommitFileAdditions::new(add).into_element(cx),
                    ui_ai::CommitFileDeletions::new(del).into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        },
    ))
    .into_element(cx)])
    .test_id("ui-ai-commit-content");

    let commit = ui_ai::Commit::new(header, content)
        .default_open(false)
        .into_element(cx)
        .test_id("ui-ai-commit-root");

    commit
}
// endregion: example
