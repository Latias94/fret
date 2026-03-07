use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
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
                shadcn::TableCell::new(cx.text("Green")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("modified")).into_element(cx),
                shadcn::TableCell::new(cx.text("M")).into_element(cx),
                shadcn::TableCell::new(cx.text("Yellow")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("deleted")).into_element(cx),
                shadcn::TableCell::new(cx.text("D")).into_element(cx),
                shadcn::TableCell::new(cx.text("Red")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("renamed")).into_element(cx),
                shadcn::TableCell::new(cx.text("R")).into_element(cx),
                shadcn::TableCell::new(cx.text("Blue")).into_element(cx),
            ],
        )
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Table::new([header, body]).into_element(cx)
}

fn parts_props_table(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let header = shadcn::TableHeader::new([shadcn::TableRow::new(
        3,
        [
            shadcn::TableHead::new("Part").into_element(cx),
            shadcn::TableHead::new("Key inputs").into_element(cx),
            shadcn::TableHead::new("Notes").into_element(cx),
        ],
    )
    .into_element(cx)])
    .into_element(cx);

    let body = shadcn::TableBody::new([
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("Commit")).into_element(cx),
                shadcn::TableCell::new(cx.text("header, content, default_open")).into_element(cx),
                shadcn::TableCell::new(cx.text(
                    "Collapsible root surface; mechanism stays in shadcn/fret-ui primitives.",
                ))
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitHeader")).into_element(cx),
                shadcn::TableCell::new(cx.text("children, test_id")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Trigger row; nested actions should not toggle the disclosure."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitAuthor / CommitInfo")).into_element(cx),
                shadcn::TableCell::new(cx.text("children")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Layout helpers for the left avatar slot and the main info column."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitAuthorAvatar")).into_element(cx),
                shadcn::TableCell::new(cx.text("new(initials)")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Avatar fallback initials, matching the official example."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitMessage / CommitHash")).into_element(cx),
                shadcn::TableCell::new(cx.text("new(text)")).into_element(cx),
                shadcn::TableCell::new(cx.text("Text leaves for the message and short hash."))
                    .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitMetadata")).into_element(cx),
                shadcn::TableCell::new(cx.text("children")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Inline metadata row for hash, separator, and timestamp."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitSeparator")).into_element(cx),
                shadcn::TableCell::new(cx.text("default() | new(text) | children(...)"))
                    .into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Docs-aligned custom separator slot; defaults to `•`."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitTimestamp")).into_element(cx),
                shadcn::TableCell::new(cx.text("new(date) | children(...)")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Relative time by default; custom children mirror the official API."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitActions")).into_element(cx),
                shadcn::TableCell::new(cx.text("children")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Trailing action cluster; button activation stays app-owned."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitCopyButton")).into_element(cx),
                shadcn::TableCell::new(cx.text("new(hash), on_copy(...), timeout(...)"))
                    .into_element(cx),
                shadcn::TableCell::new(cx.text(
                    "Matches upstream copied-state suppression and exposes a hook for app effects.",
                ))
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(
                    cx.text("CommitContent / CommitFiles / CommitFile / CommitFileInfo"),
                )
                .into_element(cx),
                shadcn::TableCell::new(cx.text("children")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Composable wrappers for the disclosure body and file rows."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitFileStatus")).into_element(cx),
                shadcn::TableCell::new(cx.text("new(status) | children(...)")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Docs-aligned custom status slot; defaults to A/M/D/R labels."),
                )
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitFileIcon")).into_element(cx),
                shadcn::TableCell::new(cx.text("default()")).into_element(cx),
                shadcn::TableCell::new(cx.text("Muted file glyph matching the official chrome."))
                    .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(cx.text("CommitFilePath")).into_element(cx),
                shadcn::TableCell::new(cx.text("new(path), on_click(...), test_id(...)"))
                    .into_element(cx),
                shadcn::TableCell::new(cx.text(
                    "Upstream is presentational; Fret adds an explicit file-open seam for apps.",
                ))
                .into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            [
                shadcn::TableCell::new(
                    cx.text("CommitFileChanges / CommitFileAdditions / CommitFileDeletions"),
                )
                .into_element(cx),
                shadcn::TableCell::new(cx.text("children | new(count)")).into_element(cx),
                shadcn::TableCell::new(
                    cx.text("Monospace change counters aligned with the official preview."),
                )
                .into_element(cx),
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
    let custom_children = snippets::commit_custom_children::render(cx);
    let features = doc_layout::notes(
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
    let findings = doc_layout::notes(
        cx,
        [
            "Mechanism/lifecycle looks healthy here: existing copy + large-list diag gates already cover toggle, copy feedback, and scroll seams.",
            "The main drift was component-layer parity: the Gallery page was less docs-aligned than other AI Elements surfaces, and `Commit` was missing the three upstream custom-children slots.",
            "`CommitFilePath::on_click(...)` remains an intentional Fret-only seam so apps can own file-open effects without pushing policy into `fret-ui`.",
        ],
    );
    let file_status = file_status_table(cx);
    let props = parts_props_table(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "The Commit component displays commit details including hash, message, author, timestamp, and changed files.",
        ),
        vec![
            DocSection::new("Overview", demo)
                .description("Rust/Fret analogue of the official AI Elements preview.")
                .test_id_prefix("ui-gallery-ai-commit-demo")
                .code_rust_from_file_region(snippets::commit_demo::SOURCE, "example"),
            DocSection::new("Features", features).no_shell(),
            DocSection::new("File Status", file_status).no_shell(),
            DocSection::new("Custom Children", custom_children)
                .description(
                    "Covers the three upstream custom-content slots: `CommitSeparator`, `CommitTimestamp`, and `CommitFileStatus`.",
                )
                .test_id_prefix("ui-gallery-ai-commit-custom-children")
                .code_rust_from_file_region(snippets::commit_custom_children::SOURCE, "example"),
            DocSection::new("Parts & Props", props).no_shell(),
            DocSection::new("Notes", findings)
                .description("Layering + parity findings for Commit."),
        ],
    );

    vec![body]
}
