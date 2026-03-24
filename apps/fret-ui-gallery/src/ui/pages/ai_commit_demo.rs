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
                "root() | children(header/content) | new(header, content) | default_open",
                "Collapsible root surface; docs-style children composition is supported, while the eager `new(header, content)` lane remains available.",
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
                "new(text) | children(...)",
                "Text leaves for the message and short hash; children overrides keep the surface closer to upstream JSX composition.",
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
                "new(hash) | children(...) | on_copy(...) | timeout(...)",
                "Matches upstream copied-state suppression, keeps the icon slot overridable, and exposes an app-owned success hook; `onError` is still not surfaced because clipboard writes are fire-and-forget today.",
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
                "new(path) | children(...) | on_click(...) | test_id(...)",
                "Upstream is presentational; Fret adds an explicit file-open seam for apps while preserving a children-based content override.",
            ],
            [
                "CommitFileChanges / CommitFileAdditions / CommitFileDeletions",
                "children | new(count)",
                "Monospace change counters aligned with the official preview; additions/deletions also support children overrides like upstream.",
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
        "The remaining work here is public-surface parity, not runtime mechanism: `Commit` now exposes the upstream-style compound root plus the documented custom-content slots on the common leaf surfaces used by the Gallery snippets.",
        "Rust now supports the docs-style `Commit::root().children([header, content])` lane; `Commit::new(header, content)` stays as the eager convenience builder.",
        "The main intentional API gap versus the web source is `CommitCopyButton.onError`: Fret clipboard writes are currently fire-and-forget effects, so the component does not yet receive a structured failure callback.",
        "`CommitFilePath::on_click(...)` remains an intentional Fret-only seam so apps can own file-open effects without pushing policy into `fret-ui`.",
        "This detail page is gated behind `gallery-dev`, which is also required for the wider `fret-ui-ai` surfaces in UI Gallery.",
    ]);
    let file_status = file_status_table(cx);
    let props = parts_props_table(cx);
    let overview_section = DocSection::build(cx, "Example", demo)
        .description("Rust/Fret analogue of the official AI Elements commit example, using the docs-style compound root.")
        .test_id_prefix("ui-gallery-ai-commit-demo")
        .code_rust_from_file_region(snippets::commit_demo::SOURCE, "example");
    let features_section = DocSection::build(cx, "Features", features).no_shell();
    let file_status_section = DocSection::build(cx, "File Status", file_status).no_shell();
    let custom_children_section = DocSection::build(cx, "Custom Children", custom_children)
        .description(
            "Covers the official custom-content slots plus leaf overrides that keep first-party Rust examples close to the upstream JSX composition model.",
        )
        .test_id_prefix("ui-gallery-ai-commit-custom-children")
        .code_rust_from_file_region(snippets::commit_custom_children::SOURCE, "example");
    let props_section = DocSection::build(cx, "Props", props).no_shell();
    let notes_section = DocSection::build(cx, "Notes", findings)
        .description("Layering + parity findings for Commit.");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "The Commit component displays commit details including hash, message, author, timestamp, and changed files.",
        ),
        vec![
            overview_section,
            features_section,
            file_status_section,
            custom_children_section,
            props_section,
            notes_section,
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
