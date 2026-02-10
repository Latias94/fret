# ADR 0203: Code Editor Display Fragments and DisplayMap Composition v1

- Status: Proposed
- Date: 2026-02-08
- Related:
  - ADR 0200 (Code editor ecosystem v1)
  - ADR 0044 (Text editing state and commands)
  - ADR 0045 / ADR 0046 (Text geometry queries)
  - ADR 0012 / ADR 0071 (IME and composition range semantics)
  - ADR 0190 (Prepaint-windowed virtual surfaces)
  - ADR 0192 (Retained windowed surface hosts)

## Context

The code editor ecosystem already has a view-layer `DisplayMap` that supports:

- logical lines (`\n`) and display rows (optional wrap-after-N-columns),
- fold placeholders (replace a buffer range with placeholder text),
- inlays (inserted display text that does not mutate the underlying buffer).

However, inline IME preedit is currently modeled as a **paint-time injection** at the UI surface
layer. As a result, when preedit is active we intentionally suppress fold placeholders and inlays
to keep buffer↔display mapping stable.

This is acceptable for v1, but it fragments the long-term “display mapping” contract across:

- view mapping (`fret-code-editor-view`),
- UI paint (`fret-code-editor`),
- a11y export (`SemanticsRole::TextField` value/selection/composition ranges),
- hit-testing and caret mapping.

## Problem

We need a stable contract for composing multiple “display text sources” into a single mapping
surface so that:

1) fold placeholders, inlays, and inline preedit can coexist without disabling each other,
2) caret/selection/hit-test/a11y range mapping remains deterministic,
3) future editor-grade features can reuse the same seam (e.g. inline widgets, diagnostics markers),
4) we avoid a late rewrite of the view layer once downstream apps start depending on these behaviors.

## Decision

### 1) Introduce a view-layer “display fragments” composition contract

`ecosystem/fret-code-editor-view` defines a composition model for **display fragments**:

- A display fragment is a contiguous span of UTF-8 text that is present in the rendered/semantics
  value but does not necessarily correspond 1:1 to buffer bytes.
- Each fragment MUST map to a deterministic “buffer anchor” byte offset (`maps_to`) for clamping.
- Fragments MUST be treated as **atomic units** for wrapping and mapping (no partial selection
  inside a fragment at the view layer).

This contract is view-owned (ecosystem), not runtime-owned (`crates/fret-ui`), to keep the “policy
surface” out of the core mechanism layer (ADR 0066).

### 2) Fold placeholders, inlays, and preedit are modeled uniformly as fragments

The view layer treats these as fragment sources:

- **Fold placeholder**: replaces a buffer byte range with placeholder fragment text.
- **Inlay**: inserts an inlay fragment at a buffer byte insertion point (no buffer mutation).
- **Inline preedit** (future): inserts a preedit fragment at a buffer anchor (typically caret /
  selection start), and is reported as the IME composition range within the composed display value.

### 3) Composition order is deterministic and conflict rules are explicit

The composed fragment stream is produced with a deterministic order (within a single logical line):

1) Start from the buffer line text.
2) Apply folds (range replacement) first.
3) Apply inlays (insertion) next.
4) Apply inline preedit last (insertion) (future).

Conflict and clamping rules (v1 contract; applies even before preedit composition is implemented):

- Inlays whose insertion point lands inside a folded range are ignored.
- Preedit anchors that land inside a folded range are clamped to the fold start (before the
  placeholder) (future; consistent with fold placeholder atomicity).
- If two fragment insertions share the same `maps_to` anchor, the order MUST be deterministic
  (tie-break by fragment kind, then stable id).

### 4) Mapping behavior for positions inside a fragment is clamped, not policy-driven

Because fragments are atomic at the view layer, any display offset that falls “inside” a fragment
maps to the fragment’s `maps_to` anchor when converted back to buffer bytes. This keeps:

- caret clamping deterministic,
- a11y selection/composition mapping safe (ADR 0071),
- hit-testing stable under windowing (ADR 0190).

UI policy can choose how to present/activate fragments (e.g. clicking an inlay could jump the caret
before it), but the view-layer mapping must remain stable and non-surprising.

### 5) A11y “value/selection/composition” export uses the composed display value

When fragment composition is enabled for inline preedit, the editor’s semantics export MUST:

- produce the composed display value string for the exported window,
- express selection and composition ranges as UTF-8 byte offsets into that composed value (ADR 0071),
- keep range invariants valid even when fragments are present (clamping where necessary).

## Non-goals

- Pixel-accurate wrapping and caret stops (this remains renderer-driven; the view layer is column
  based in v1).
- Rich inline widgets inside text layout (separate follow-up; may reuse this fragment contract).
- Multi-cursor / multi-selection (separate follow-up).

## Migration plan (recommended)

1) Keep v1 behavior: while preedit is active, suppress folds/inlays (already gated by diagnostics).
2) Extend `fret-code-editor-view` to accept a preedit fragment source and produce a composed
   `DisplayMap` under the contract above.
3) Update the editor surface to stop paint-time preedit injection and instead rely on the composed
   view mapping for both paint and semantics export.
4) Add a dedicated diag script that validates “preedit + folds + inlays” coexistence without mapping
   drift.

## Evidence anchors (current state)

- View mapping and fragments: `ecosystem/fret-code-editor-view/src/lib.rs` (`DisplayMap`, `DisplayRowFragment`).
- Fold mapping atomicity: `ecosystem/fret-code-editor-view/src/folds.rs` (`folded_*` mapping helpers).
- Inlay validation and insertion: `ecosystem/fret-code-editor-view/src/inlays.rs`.
- Current v1 suppression policy: `ecosystem/fret-code-editor/src/editor/mod.rs` (`refresh_display_map`).
- Regression gates (v1 policy): `tools/diag-scripts/ui-gallery-code-editor-torture-*-inline-preedit-baseline.json`,
  `apps/fretboard/src/diag/stats.rs` (fold/inlay absent under inline preedit).

## Implementation gap checklist (as of 2026-02-09)

This section tracks concrete gaps between the contract in this ADR and the current implementation.
It is intentionally code-oriented so migration work is easy to scope and review.

### Already aligned (v1)

- View-layer fold placeholders and inlays participate in wrapped row-breaking and buffer↔display mapping:
  - `ecosystem/fret-code-editor-view/src/lib.rs` (`DisplayRowFragment`, `compute_wrapped_row_start_cols`,
    `decorated_byte_to_col`, `decorated_col_to_byte`).
- Fold placeholder atomicity and clamping rules are enforced at the view layer:
  - `ecosystem/fret-code-editor-view/src/folds.rs` (`folded_*` mapping helpers and tests).
- Inlays that land inside folded ranges are ignored (v1 contract):
  - `ecosystem/fret-code-editor-view/src/lib.rs` (inlay cursor advancement around folds).
- v1 policy and staging are locked by diagnostics:
  - v1 suppress gates (decorations absent under inline preedit):
    - `tools/diag-scripts/ui-gallery-code-editor-torture-*-soft-wrap-inline-preedit-baseline.json`
    - `crates/fret-diag/src/stats.rs` (`*_absent_under_inline_preedit`).
  - ADR 0203 staging opt-ins (decorations present under inline preedit in specific baselines):
    - unwrapped: `tools/diag-scripts/ui-gallery-code-editor-torture-*-inline-preedit-baseline.json`
    - wrapped: `tools/diag-scripts/ui-gallery-code-editor-torture-*-with-decorations-baseline.json`
    - gates: `crates/fret-diag/src/stats.rs` (`*_present_under_inline_preedit_*`).
  - opt-in surface (editor policy): `ecosystem/fret-code-editor/src/editor/mod.rs`
    (`allow_decorations_under_inline_preedit`).

### Remaining gaps (v2+; ADR 0203)

- [x] Add a preedit fragment representation to `fret-code-editor-view` (data model + composition order).
  - Implemented: `ecosystem/fret-code-editor-view/src/lib.rs` (`InlinePreedit`, `DisplayRowFragment::Preedit`).
- [x] Extend view-layer mapping and wrapping helpers to account for preedit insertion.
  - Implemented: `ecosystem/fret-code-editor-view/src/lib.rs`
    (`DisplayMap::new_with_decorations_and_preedit`, preedit-aware row breaking and mapping helpers).
  - Rule enforced: any display offset “inside” the preedit fragment maps back to the fragment’s anchor.
- [ ] Promote inline IME preedit from paint-time string splicing to a view-layer fragment source (fully).
  - Status: **partially implemented behind an ecosystem policy toggle**.
  - Current baseline still supports paint-time injection:
    `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`RowPreeditMapping`, `materialize_preedit_rich_text`).
  - New composed path: `ecosystem/fret-code-editor/src/editor/mod.rs` (`compose_inline_preedit`) and
    `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (preedit composed into cached row text).
- [x] Provide a view-owned way to materialize composed display text for windowed export ranges.
  - Used by: paint row text (shaping), semantics `TextField.value`, and debug snapshots.
  - Constraint: windowed-only outputs (ADR 0190); avoid full-document composed strings.
  - Evidence:
    - `ecosystem/fret-code-editor-view/src/lib.rs` (`DisplayMap::materialize_display_row_text` + tests).
    - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`cached_row_text_with_range` uses the view-owned materialization).
- [ ] Migrate editor paint + hit-test + caret/selection mapping to consume the composed view mapping.
  - Goal: remove the paint-time preedit injection path and avoid “dual mapping” seams.
- [ ] Update a11y export to consume the composed display value (and composition range) from the view layer.
  - Current a11y preedit handling lives in: `ecosystem/fret-code-editor/src/editor/a11y/mod.rs`
    (`a11y_composed_text_window`, `map_a11y_offset_to_buffer_with_preedit`).
  - Goal: keep ADR 0071 invariants while relying on one mapping surface.
- [ ] Add a dedicated diag baseline + gate for “soft wrap + folds + inlays + inline preedit” coexistence.
  - Baseline (new): `tools/diag-scripts/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-baseline.json`.
  - Gate (new): asserts at least one snapshot where folds and inlays are observed while preedit is active,
    plus a minimal mapping sanity check (e.g. caret does not jump during controlled toggles).
  - Keep the existing v1 suppress + staging opt-in gates until this composed path is stable.
