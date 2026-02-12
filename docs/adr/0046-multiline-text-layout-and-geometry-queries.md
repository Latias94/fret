# ADR 0046: Multiline Text Layout and Geometry Queries

Status: Accepted

## Context

Fret’s near-term widgets are mostly single-line (search fields, property values, command palette).
However, editor-grade applications quickly require **multiline** text surfaces:

- inspectors with wrapped values,
- logs/output panes,
- markdown/rich text previews (future),
- and ultimately a code editor (large documents; virtualized).

We already locked:

- the text boundary (`TextBlobId` + `TextMetrics`) (ADR 0006),
- text editing state representation (byte indices) and command vocabulary (ADR 0044),
- single-line geometry queries (caret/hit-test) (ADR 0045).

The remaining “hard-to-change” decision is: **how multiline layout participates in geometry queries** without forcing
a redesign of `SceneOp::Text` or of IME cursor-area behavior later.

## Decision

### 1) `TextBlobId` represents a shaped layout, potentially multiline

`TextBlobId` continues to be the only text handle used by `SceneOp::Text`.

A prepared blob may represent:

- a single visual line (wrap disabled), or
- multiple visual lines (wrap enabled and/or explicit newlines).

### 2) Multiline geometry uses caret affinity at line breaks

Multiline introduces ambiguity at line boundaries (the caret can be “end of previous line” or “start of next line”).

We therefore reserve and standardize:

- `CaretAffinity::{Upstream,Downstream}`
- `TextService::caret_rect(blob, index, affinity)`
- `TextService::hit_test_point(blob, point) -> HitTestResult { index, affinity }`

Rules:

- `Upstream` means “prefer the position before a line break”.
- `Downstream` means “prefer the position after a line break”.
- For single-line blobs, implementations may ignore affinity.

### 3) Coordinate spaces for multiline are still local-to-text

All multiline geometry query results are returned in **logical pixels**, relative to the text origin:

- x=0 at the start of the first line
- y=0 at the top of the text box

Widgets remain responsible for:

- padding and baseline placement within a control,
- clipping/scissoring,
- converting to window coordinates for IME cursor-area effects.

### 4) Large-document editors are a separate layer (virtualization)

This ADR covers multiline for “moderate text surfaces” (wrapped labels, logs).

For code-editor-grade surfaces:

- layout must be virtualized (ADR 0042),
- the editor may use a higher-level text layout/cache object that reuses the same index and geometry semantics,
  but does not require a single monolithic `TextBlobId` for an entire document.

This separation avoids forcing `fret-core` to own a full text document model while still keeping the geometry semantics stable.

## Consequences

- IME cursor-area placement remains correct for multiline widgets (caret rect is unambiguous via affinity).
- We can implement wrapped labels/log panes without inventing new coordinate conventions.
- Future code editor work can build on the same caret/index semantics while using virtualization for scale.

## Implementation Notes (Current Workspace)

- Types and stubs:
  - `crates/fret-core/src/text.rs` (`CaretAffinity`, `HitTestResult`)
  - `crates/fret-core/src/text.rs` (`TextService::caret_rect`, `TextService::hit_test_point`)
- Single-line implementations exist in the renderer:
  - `crates/fret-render-wgpu/src/renderer/mod.rs`
  - `crates/fret-render-wgpu/src/text/mod.rs`
