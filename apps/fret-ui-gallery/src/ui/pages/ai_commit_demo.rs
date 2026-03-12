use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::UiCx;
use fret_ui_kit::ui::UiElementSinkExt as _;

fn file_status_table(cx: &mut UiCx<'_>) -> AnyElement {
    let row = |status: &'static str, label: &'static str, color: &'static str| {
        shadcn::TableRow::build(3, move |cx, out| {
            out.push_ui(cx, shadcn::TableCell::build(ui::text(status)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(label)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(color)));
        })
    };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(3, |cx, out| {
                        out.push(shadcn::TableHead::new("Status").into_element(cx));
                        out.push(shadcn::TableHead::new("Label").into_element(cx));
                        out.push(shadcn::TableHead::new("Color").into_element(cx));
                    })
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push_ui(cx, row("added", "A", "Green"));
                out.push_ui(cx, row("modified", "M", "Yellow"));
                out.push_ui(cx, row("deleted", "D", "Red"));
                out.push_ui(cx, row("renamed", "R", "Blue"));
            }),
        );
    })
    .into_element(cx)
}

fn parts_props_table(cx: &mut UiCx<'_>) -> AnyElement {
    let row = |part: &'static str, inputs: &'static str, notes: &'static str| {
        shadcn::TableRow::build(3, move |cx, out| {
            out.push_ui(cx, shadcn::TableCell::build(ui::text(part)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(inputs)));
            out.push_ui(cx, shadcn::TableCell::build(ui::text(notes)));
        })
    };

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(3, |cx, out| {
                        out.push(shadcn::TableHead::new("Part").into_element(cx));
                        out.push(shadcn::TableHead::new("Key inputs").into_element(cx));
                        out.push(shadcn::TableHead::new("Notes").into_element(cx));
                    })
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push_ui(cx, row("Commit", "header, content, default_open", "Collapsible root surface; mechanism stays in shadcn/fret-ui primitives."));
                out.push_ui(cx, row("CommitHeader", "children, test_id", "Trigger row; nested actions should not toggle the disclosure."));
                out.push_ui(cx, row("CommitAuthor / CommitInfo", "children", "Layout helpers for the left avatar slot and the main info column."));
                out.push_ui(cx, row("CommitAuthorAvatar", "new(initials)", "Avatar fallback initials, matching the official example."));
                out.push_ui(cx, row("CommitMessage / CommitHash", "new(text)", "Text leaves for the message and short hash."));
                out.push_ui(cx, row("CommitMetadata", "children", "Inline metadata row for hash, separator, and timestamp."));
                out.push_ui(cx, row("CommitSeparator", "default() | new(text) | children(...)", "Docs-aligned custom separator slot; defaults to `•`."));
                out.push_ui(cx, row("CommitTimestamp", "new(date) | children(...)", "Relative time by default; custom children mirror the official API."));
                out.push_ui(cx, row("CommitActions", "children", "Trailing action cluster; button activation stays app-owned."));
                out.push_ui(cx, row("CommitCopyButton", "new(hash), on_copy(...), timeout(...)", "Matches upstream copied-state suppression and exposes a hook for app effects."));
                out.push_ui(cx, row("CommitContent / CommitFiles / CommitFile / CommitFileInfo", "children", "Composable wrappers for the disclosure body and file rows."));
                out.push_ui(cx, row("CommitFileStatus", "new(status) | children(...)", "Docs-aligned custom status slot; defaults to A/M/D/R labels."));
                out.push_ui(cx, row("CommitFileIcon", "default()", "Muted file glyph matching the official chrome."));
                out.push_ui(cx, row("CommitFilePath", "new(path), on_click(...), test_id(...)", "Upstream is presentational; Fret adds an explicit file-open seam for apps."));
                out.push_ui(cx, row("CommitFileChanges / CommitFileAdditions / CommitFileDeletions", "children | new(count)", "Monospace change counters aligned with the official preview."));
            }),
        );
    })
    .into_element(cx)
}

pub(super) fn preview_ai_commit_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
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
