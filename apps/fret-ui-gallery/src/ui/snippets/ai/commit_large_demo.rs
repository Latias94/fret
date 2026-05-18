pub const SOURCE: &str = include_str!("commit_large_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui::element::{AnyElement, Length, SemanticsProps, SpacerProps};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement {
    cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Generic,
            test_id: Some(Arc::<str>::from(test_id)),
            ..Default::default()
        },
        |cx| {
            vec![cx.spacer(SpacerProps {
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(fret_core::Px(0.0)),
                        height: Length::Px(fret_core::Px(0.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                min: fret_core::Px(0.0),
            })]
        },
    )
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let opened = cx.local_model_keyed("opened", || None::<Arc<str>>);

    let opened_now = cx.get_model_cloned(&opened, Invalidation::Paint).flatten();
    let opened_marker = opened_now
        .is_some()
        .then(|| state_marker(cx, "ui-ai-commit-large-opened-marker"));

    let hash: Arc<str> = Arc::from("d00df00d");
    let hash_for_title = hash.clone();
    let header = ui_ai::CommitHeader::new([
        ui::h_row(move |cx| {
            vec![
                ui_ai::CommitHash::new(hash_for_title.clone()).into_element(cx),
                ui_ai::CommitMessage::new("Large file list scroll").into_element(cx),
            ]
        })
        .layout(LayoutRefinement::default().min_w_0())
        .gap(Space::N3)
        .items_center()
        .into_element(cx),
        ui_ai::CommitActions::new([ui_ai::CommitCopyButton::new(hash.clone()).into_element(cx)])
            .into_element(cx),
    ])
    .test_id("ui-ai-commit-large-header");

    let mut file_rows: Vec<AnyElement> = Vec::new();
    for index in 0..200usize {
        let path: Arc<str> = Arc::from(format!("src/generated/file_{index:04}.rs"));
        let test_id: Arc<str> = Arc::from(format!("ui-ai-commit-large-file-{index:04}-path"));

        let on_click: ui_ai::OnCommitFilePathClick = Arc::new({
            let opened = opened.clone();
            move |host, _action_cx, next| {
                let _ = host.models_mut().update(&opened, |v| *v = Some(next));
            }
        });

        let row = ui_ai::CommitFile::new([
            ui_ai::CommitFileInfo::new([
                ui_ai::CommitFileStatus::new(ui_ai::CommitFileStatusKind::Modified)
                    .into_element(cx),
                ui_ai::CommitFileIcon::default().into_element(cx),
                ui_ai::CommitFilePath::new(path)
                    .on_click(on_click)
                    .test_id(test_id)
                    .into_element(cx),
            ])
            .into_element(cx),
            ui_ai::CommitFileChanges::new([
                ui_ai::CommitFileAdditions::new((index % 17) as u32).into_element(cx),
                ui_ai::CommitFileDeletions::new((index % 9) as u32).into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);
        file_rows.push(row);
    }

    let mut content_children = vec![
        ui_ai::CommitSeparator::new("Files").into_element(cx),
        ui_ai::CommitFiles::new(file_rows).into_element(cx),
    ];
    if let Some(marker) = opened_marker {
        content_children.push(marker);
    }

    let content = ui_ai::CommitContent::new(content_children).test_id("ui-ai-commit-large-content");

    let commit = ui_ai::Commit::new(header, content)
        .default_open(false)
        .into_element(cx)
        .test_id("ui-ai-commit-large-root");

    ui::v_flex(move |cx| {
        vec![
            decl_text::text_section_chrome_label(cx, "Commit (Large)"),
            decl_text::text_paragraph(
                cx,
                "Scroll-heavy surface for hit testing + viewport scrolling.",
            ),
            commit,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
