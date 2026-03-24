pub const SOURCE: &str = include_str!("commit_custom_children.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let hash: Arc<str> = Arc::from("9f8e7d6c5b4a3210feedbeefcafe123456789abc");
    let short_hash: Arc<str> = Arc::from(hash.chars().take(7).collect::<String>());
    let timestamp = SystemTime::now()
        .checked_sub(Duration::from_secs(60 * 60 * 2))
        .unwrap_or_else(SystemTime::now);

    let header = ui_ai::CommitHeader::new([
        ui_ai::CommitAuthor::new([ui_ai::CommitAuthorAvatar::new("FX").into_element(cx)])
            .into_element(cx),
        ui_ai::CommitInfo::new([
            ui_ai::CommitMessage::new("docs: align commit compound-part API")
                .children([
                    shadcn::raw::typography::muted("docs: align commit compound-part API")
                        .into_element(cx),
                ])
                .into_element(cx),
            ui_ai::CommitMetadata::new([
                ui_ai::CommitHash::new(short_hash.clone())
                    .children([shadcn::raw::typography::muted(short_hash).into_element(cx)])
                    .into_element(cx),
                ui_ai::CommitSeparator::default()
                    .children([shadcn::raw::typography::muted("/").into_element(cx)])
                    .into_element(cx),
                ui_ai::CommitTimestamp::new(timestamp)
                    .children([shadcn::raw::typography::muted("2 hours ago").into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx),
        ui_ai::CommitActions::new([ui_ai::CommitCopyButton::new(hash.clone()).into_element(cx)])
            .into_element(cx),
    ]);

    let files = [
        (
            12u32,
            0u32,
            "docs/workstreams/ai-elements-port/ai-elements-port.md",
            ui_ai::CommitFileStatusKind::Added,
            Some(
                shadcn::Badge::new("NEW")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
            ),
        ),
        (
            8u32,
            3u32,
            "apps/fret-ui-gallery/src/ui/pages/ai_commit_demo.rs",
            ui_ai::CommitFileStatusKind::Modified,
            Some(
                shadcn::Badge::new("DOCS")
                    .variant(shadcn::BadgeVariant::Outline)
                    .into_element(cx),
            ),
        ),
    ];

    let content = ui_ai::CommitContent::new([ui_ai::CommitFiles::new(files.into_iter().map(
        |(add, del, path, status, custom_status)| {
            let status = match custom_status {
                Some(custom_status) => ui_ai::CommitFileStatus::new(status)
                    .children([custom_status])
                    .into_element(cx),
                None => ui_ai::CommitFileStatus::new(status).into_element(cx),
            };

            ui_ai::CommitFile::new([
                ui_ai::CommitFileInfo::new([
                    status,
                    ui_ai::CommitFileIcon::default().into_element(cx),
                    ui_ai::CommitFilePath::new(path)
                        .children([shadcn::raw::typography::muted(path).into_element(cx)])
                        .into_element(cx),
                ])
                .into_element(cx),
                ui_ai::CommitFileChanges::new([
                    ui_ai::CommitFileAdditions::new(add)
                        .children([
                            shadcn::raw::typography::muted(format!("+{add}")).into_element(cx)
                        ])
                        .into_element(cx),
                    ui_ai::CommitFileDeletions::new(del)
                        .children([
                            shadcn::raw::typography::muted(format!("-{del}")).into_element(cx)
                        ])
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        },
    ))
    .into_element(cx)]);

    ui_ai::Commit::new(header, content)
        .default_open(true)
        .into_element(cx)
}
// endregion: example
