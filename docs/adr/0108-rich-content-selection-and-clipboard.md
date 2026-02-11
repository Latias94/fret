# ADR 0108: Rich Content Selection and Clipboard (Zed-Aligned, Markdown-First)

Status: Proposed

## Context

Fret targets editor-grade UI and already has most of the *text geometry* contract needed for
selection:

- `TextService::hit_test_point(blob, point) -> HitTestResult` (ADR 0046)
- `TextService::caret_rect(blob, index, affinity) -> Rect` (ADR 0046)
- `TextService::selection_rects(blob, range, out)` (ADR 0045/0046)

However, Fret does not yet define the *interaction* and *composition* contract for:

- mouse selection (drag to select),
- word/line/all selection via multi-click,
- selection that spans multiple rendered lines/blocks within a rich content surface (Markdown, chat transcripts, code blocks),
- selection painting across line wraps and block boundaries,
- clipboard extraction behavior (what text is copied).

This is a “hard-to-change” behavior because it affects user expectations, input routing, and future
editor work.

Zed’s Markdown view is a good reference: it supports selection over a rendered text stream, paints
selection backgrounds, updates during drag with autoscroll, and writes to the clipboard/primary
selection on platforms that support it.

Reference:
- `repo-ref/zed/crates/markdown/src/markdown.rs:447` (`Selection` + `SelectMode`)
- `repo-ref/zed/crates/markdown/src/markdown.rs:594` (`paint_selection`)
- `repo-ref/zed/crates/markdown/src/markdown.rs:686` (mouse handling, click-count modes, autoscroll)

## Goals

1) Provide **Markdown-first** selection suitable for LLM chat transcripts.
2) Keep the UI/layout layer independent of shaping (ADR 0006 / ADR 0029).
3) Define a reusable selection foundation that can later power:
   - read-only rich text views,
   - code viewer selection,
   - editor selection (large documents; virtualization).
4) Keep the contract compatible with streaming Markdown (ADR 0099):
   selection must remain stable across incremental block commits.

## Decision

### 1) Selection is defined in a document coordinate space (source-indexed)

We standardize a source-indexed selection coordinate for rich content surfaces:

- `source_index` (UTF-8 byte offset into the Markdown source string)
- `CaretAffinity` at line breaks (ADR 0046)

Selection state:

- `anchor: SourceCursor` (fixed)
- `head: SourceCursor` (moves with mouse/keyboard)
- `pending: bool` (drag in progress)
- `mode: SelectMode` (Character / Word / Line / All), aligned with common editor UX

Rationale:
- This matches Zed’s Markdown behavior: selection state is stored as byte offsets into the original
  Markdown source, and the renderer maintains mappings from source indices to rendered text indices.
- For streaming (ADR 0099), append-only updates preserve existing indices; selection remains stable.

### 2) Rich content surfaces expose a “selection segment” interface

A rich content surface (e.g. Markdown view) is composed of ordered **selection segments** (typically
blocks).

Each segment must be able to:

1) Hit-test: `point -> source_index`
2) Paint selection: `source_range -> Vec<Rect>` (local rects suitable for background highlight)
3) Extract clipboard text: `source_range -> String`

The concrete API surface is implementation-defined (crate placement TBD), but the behavior must
follow this contract.

### 3) Selection is surface-owned (Markdown-first)

For the initial implementation, selection ownership lives in `fret-markdown` (Markdown-first):

- The Markdown surface owns `SelectionState`.
- The surface captures pointer input on drag and updates `head` as the pointer moves.
- The surface is responsible for painting selection backgrounds and providing clipboard text.

This matches Zed’s shape: a “content view” owns selection state and uses a cached rendered-text
mapping for hit-testing, painting, and clipboard extraction.

Longer-term, this can be generalized into a `fret-ui`/`fret-ui-kit` primitive once the contract has
proven stable.

### 4) Embedded components are selection boundaries (v1 lock-in)

In v1, selection and clipboard extraction operate only over the **rendered text stream** produced
by the Markdown renderer.

Implications:

- Injected non-text components (images, custom callouts, buttons, webviews, etc.) are treated as
  **selection boundaries** and are not part of the selectable text stream.
- Overlay controls (e.g. code block copy buttons) do not participate in selection.
- Images that successfully render as an image component do not contribute their alt text to the
  selectable text stream.

This aligns with Zed’s Markdown behavior and avoids hard-to-maintain “mixed selection” across
arbitrary embedded widgets.

### 5) Interaction semantics (mouse)

Baseline behavior (desktop):

- Single click + drag: character selection.
- Double click + drag: word selection (expands to whole words while dragging).
- Triple click + drag: line selection (expands to whole visual lines while dragging).
- Quadruple click (optional): select all within the surface.

During drag:

- pointer is captured,
- autoscroll is requested when the head cursor approaches the viewport edge,
- selection updates continuously.

Link arbitration:

- If selection is pending, link activation is suppressed.
- If no selection occurred, click activates the link (host-provided handler; Markdown itself remains
  render-only by default per ADR 0099).

### 6) Clipboard extraction semantics

We standardize that clipboard extraction uses **rendered text**, not the raw Markdown source:

- Markdown syntax markers are not copied (e.g. `**`, backticks).
- Code blocks copy their code content.
- Inline images:
  - default: copy their `alt` text if present, else nothing,
  - optional future: copy URL as a separate line.

Copy concatenation is **rendered-line-based** (v1 lock-in):

- extraction iterates rendered lines intersecting the selection range,
- each rendered line contributes a substring of its rendered text,
- rendered lines are joined with a single `\n`,
- no additional paragraph-aware `\n\n` insertion is performed.

This matches Zed’s Markdown `text_for_range` behavior and keeps copying deterministic under
streaming/block recomposition.

Platform-specific:

- Linux/FreeBSD may also set the primary selection on mouse-up (Zed behavior).

### 7) Relationship to Rich Text Runs (ADR 0107)

Rich selection and rich text runs are coupled:

- For “highlight + soft wrap” and Markdown inline formatting (italic/underline/strikethrough),
  selection must operate over a unified shaped layout.
- ADR 0107 introduces runs so a single shaped layout can carry per-range styling. ADR 0108 defines
  how users select across that content.

In the short term, selection may be implemented with per-block text blobs and a segment-level
selection manager. In the long term, selection should directly leverage the run-aware text layout
pipeline.

## Consequences

- `fret-markdown` becomes a true “content surface” with selection state, hit-testing, and selection
  painting.
- We will need stable mappings from rendered geometry to document offsets:
  - segment bounds (for cross-block hit-testing),
  - per-line caret stops / per-line ranges (for fast mapping and painting).
- Once stabilized, the same selection contract can be reused by `fret-code-view` and future
  “read-only rich text” widgets.

## Open Questions

1) Do we need a global selection service (window-level) for consistent keyboard shortcuts and
   context menus, or keep it surface-owned initially?
2) Do we want selection to include “alt text” for images by default, or treat images strictly as
   boundaries (Zed-like default)?
