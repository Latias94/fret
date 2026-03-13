use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn file_status_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Status", "Label", "Color"],
        [
            ["added", "A", "Green"],
            ["modified", "M", "Yellow"],
            ["deleted", "D", "Red"],
            ["renamed", "R", "Blue"],
        ],
        false,
    )
}

fn parts_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Key inputs", "Notes"],
        [
            [
                "Commit",
                "header, content, default_open",
                "Collapsible root surface; mechanism stays in shadcn/fret-ui primitives.",
            ],
            [
                "CommitHeader",
                "children, test_id",
                "Trigger row; nested actions should not toggle the disclosure.",
            ],
            [
                "CommitAuthor / CommitInfo",
                "children",
                "Layout helpers for the left avatar slot and the main info column.",
            ],
            [
                "CommitAuthorAvatar",
                "new(initials)",
                "Avatar fallback initials, matching the official example.",
            ],
            [
                "CommitMessage / CommitHash",
                "new(text)",
                "Text leaves for the message and short hash.",
            ],
            [
                "CommitMetadata",
                "children",
                "Inline metadata row for hash, separator, and timestamp.",
            ],
            [
                "CommitSeparator",
                "default() | new(text) | children(...)",
                "Docs-aligned custom separator slot; defaults to `•`.",
            ],
            [
                "CommitTimestamp",
                "new(date) | children(...)",
                "Relative time by default; custom children mirror the official API.",
            ],
            [
                "CommitActions",
                "children",
                "Trailing action cluster; button activation stays app-owned.",
            ],
            [
                "CommitCopyButton",
                "new(hash), on_copy(...), timeout(...)",
                "Matches upstream copied-state suppression and exposes a hook for app effects.",
            ],
            [
                "CommitContent / CommitFiles / CommitFile / CommitFileInfo",
                "children",
                "Composable wrappers for the disclosure body and file rows.",
            ],
            [
                "CommitFileStatus",
                "new(status) | children(...)",
                "Docs-aligned custom status slot; defaults to A/M/D/R labels.",
            ],
            [
                "CommitFileIcon",
                "default()",
                "Muted file glyph matching the official chrome.",
            ],
            [
                "CommitFilePath",
                "new(path), on_click(...), test_id(...)",
                "Upstream is presentational; Fret adds an explicit file-open seam for apps.",
            ],
            [
                "CommitFileChanges / CommitFileAdditions / CommitFileDeletions",
                "children | new(count)",
                "Monospace change counters aligned with the official preview.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_commit_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::commit_demo::render(cx);
    let custom_children = snippets::commit_custom_children::render(cx);
    let features = doc_layout::notes_block([
        "Commit hash display with copy button",
        "Author avatar with initials",
        "Relative timestamp formatting",
        "Collapsible file changes list",
        "Color-coded file status (added/modified/deleted/renamed)",
        "Line additions/deletions count",
    ]);
    let findings = doc_layout::notes_block([
        "Mechanism/lifecycle looks healthy here: existing copy + large-list diag gates already cover toggle, copy feedback, and scroll seams.",
        "The main drift was component-layer parity: the Gallery page was less docs-aligned than other AI Elements surfaces, and `Commit` was missing the three upstream custom-children slots.",
        "`CommitFilePath::on_click(...)` remains an intentional Fret-only seam so apps can own file-open effects without pushing policy into `fret-ui`.",
    ]);
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
            DocSection::build(cx, "Features", features).no_shell(),
            DocSection::build(cx, "File Status", file_status).no_shell(),
            DocSection::new("Custom Children", custom_children)
                .description(
                    "Covers the three upstream custom-content slots: `CommitSeparator`, `CommitTimestamp`, and `CommitFileStatus`.",
                )
                .test_id_prefix("ui-gallery-ai-commit-custom-children")
                .code_rust_from_file_region(snippets::commit_custom_children::SOURCE, "example"),
            DocSection::build(cx, "Parts & Props", props).no_shell(),
            DocSection::build(cx, "Notes", findings)
                .description("Layering + parity findings for Commit."),
        ],
    );

    vec![body]
}
