use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

fn file_status_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let header = shadcn::TableHeader::new([shadcn::TableRow::new(
        3,
        [
            shadcn::TableHead::new("Status").into_element(cx),
            shadcn::TableHead::new("Label").into_element(cx),
            shadcn::TableHead::new("Color").into_element(cx),
        ],
    )
    .into_element(cx)])
    .into_element(cx);

    let body = shadcn::TableBody::new([
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("added")).into_element(cx),
                shadcn::TableCell::new(cx.text("A")).into_element(cx),
                shadcn::TableCell::new(cx.text("green")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("modified")).into_element(cx),
                shadcn::TableCell::new(cx.text("M")).into_element(cx),
                shadcn::TableCell::new(cx.text("yellow")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("deleted")).into_element(cx),
                shadcn::TableCell::new(cx.text("D")).into_element(cx),
                shadcn::TableCell::new(cx.text("red")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("renamed")).into_element(cx),
                shadcn::TableCell::new(cx.text("R")).into_element(cx),
                shadcn::TableCell::new(cx.text("blue")).into_element(cx),
            ],
        )
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Table::new([header, body]).into_element(cx)
}

fn props_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let header = shadcn::TableHeader::new([shadcn::TableRow::new(
        2,
        [
            shadcn::TableHead::new("Component").into_element(cx),
            shadcn::TableHead::new("Notes").into_element(cx),
        ],
    )
    .into_element(cx)])
    .into_element(cx);

    let body = shadcn::TableBody::new([
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("Commit")).into_element(cx),
                shadcn::TableCell::new(cx.text("Spread to the disclosure root.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitHeader")).into_element(cx),
                shadcn::TableCell::new(cx.text("Spread to the trigger row (toggle)."))
                    .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitAuthor")).into_element(cx),
                shadcn::TableCell::new(cx.text("Author row container.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitAuthorAvatar")).into_element(cx),
                shadcn::TableCell::new(cx.text("Initials badge (avatar).")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitInfo")).into_element(cx),
                shadcn::TableCell::new(cx.text("Message + metadata column.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitMessage")).into_element(cx),
                shadcn::TableCell::new(cx.text("Commit message label.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitMetadata")).into_element(cx),
                shadcn::TableCell::new(cx.text("Hash + separator + timestamp row."))
                    .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitHash")).into_element(cx),
                shadcn::TableCell::new(cx.text("Monospace short hash label.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitSeparator")).into_element(cx),
                shadcn::TableCell::new(cx.text("Separator token (default: \"•\")."))
                    .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitTimestamp")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Relative timestamp label (defaults to relative days)."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitActions")).into_element(cx),
                shadcn::TableCell::new(cx.text("Right-side actions row.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitCopyButton")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Copies the full hash; supports timeout + (optional) on_copy."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitContent")).into_element(cx),
                shadcn::TableCell::new(cx.text("Disclosure content wrapper.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitFiles")).into_element(cx),
                shadcn::TableCell::new(cx.text("File list wrapper.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitFile")).into_element(cx),
                shadcn::TableCell::new(cx.text("File row wrapper.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitFileInfo")).into_element(cx),
                shadcn::TableCell::new(cx.text("Status + icon + path group.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitFileStatus")).into_element(cx),
                shadcn::TableCell::new(cx.text("Status label: A/M/D/R.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitFileIcon")).into_element(cx),
                shadcn::TableCell::new(cx.text("File icon.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitFilePath")).into_element(cx),
                shadcn::TableCell::new(cx.text("Path label (optionally pressable via on_click)."))
                    .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitFileChanges")).into_element(cx),
                shadcn::TableCell::new(cx.text("Right-side changes group.")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitFileAdditions")).into_element(cx),
                shadcn::TableCell::new(cx.text("Additions count (+N).")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            2,
            [
                shadcn::TableCell::new(cx.text("CommitFileDeletions")).into_element(cx),
                shadcn::TableCell::new(cx.text("Deletions count (-N).")).into_element(cx),
            ],
        )
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Table::new([header, body]).into_element(cx)
}

pub(super) fn preview_ai_commit_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::commit_demo::render(cx);
    let features = crate::ui::doc_layout::notes(
        cx,
        [
            "Commit hash display with copy button",
            "Author avatar with initials",
            "Relative timestamp formatting",
            "Collapsible file changes list",
            "Color-coded file status (added/modified/deleted/renamed)",
            "Line additions/deletions count",
        ],
    );
    let file_status = file_status_table(cx);
    let props = props_table(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![
            DocSection::new("Commit", demo)
                .descriptions([
                    "Disclosure surface built on top of Collapsible primitives.",
                    "Copy action should not toggle the disclosure.",
                ])
                .test_id_prefix("ui-gallery-ai-commit-demo")
                .code_rust_from_file_region(snippets::commit_demo::SOURCE, "example"),
            DocSection::new("Features", features).no_shell(),
            DocSection::new("File status", file_status).no_shell(),
            DocSection::new("Parts & props", props).no_shell(),
        ],
    );

    vec![body]
}
