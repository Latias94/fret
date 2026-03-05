# Docking TabBar Fearless Refactor v1 (TODO)

## Goals

- Make docking TabBar drop behavior deterministic and regression-gated.
- Reduce coupling between docking model ops and TabBar rendering details.
- Provide a clear path to editor-grade semantics (pinned/preview/dirty) without leaking policy into `fret-ui`.

## Work items

### Contracts

- Write down the docking TabBar “resolved drop” contract surface:
  - `ResolvedZone` vocabulary (tab, tab-gap, end-space, header-space, content split zones)
  - `insert_index` definition (canonical list order; pinned/preview considerations)
- Decide whether explicit end-drop surfaces should be:
  - declarative `test_id` anchors (where possible), and/or
  - diagnostics predicates only (for self-drawn widgets)

### Kernel extraction (unit-testable)

- Extract a `tab_bar_kernel` module:
  - Inputs: tabs geometry + header geometry + pointer
  - Outputs: resolved zone + insert index (+ split placement)
- Status: partially extracted (drop-surface + insert-index resolution)
  - Evidence: `ecosystem/fret-docking/src/dock/tab_bar_kernel.rs` (`resolve_tab_bar_drop`)
  - Consumer: `ecosystem/fret-docking/src/dock/tab_bar_drop_target.rs` (thin wrapper)
- Add unit tests for edge cases:
  - empty stack
  - one tab
  - drop at end (must produce `insert_index == tab_count`)
  - overflow visible subset (insert index stable in canonical list)
- Harden overflow control surfaces:
  - overflow button is not treated as a drop surface
  - reserved header space between strip and overflow control resolves to an explicit end-drop (`insert_index == tab_count`)

### Modularization (fearless refactor hygiene)

- [x] Extract `pressed_tab_close` pointer-up handling into a helper to keep `DockSpace` input
  arbitration auditable.

### Overflow

- Refactor the existing overflow button/menu into a pipeline:
  - compute visible tabs
  - compute overflow tabs
  - ensure drop resolution works across both
- Add overflow affordances:
  - dropdown list
  - scroll buttons or track-scroll behavior

### Diagnostics gates (scripted)

- Keep the “drop-end resolves insert_index” scripts as the baseline gate.
- Add a “drag reorder inside stack” script with insert-index assertions.
- Add a “drag-to-split from tab bar” script (if we expose stable split preview semantics).

### Editor semantics (later, policy layer)

- Pinned tabs region:
  - pinned never closes by default
  - pinned reorder rules (within pinned region vs across boundary)
- Preview tabs:
  - click to “commit”
  - replaced by next preview unless committed
- Dirty indicator / close confirmation policy (workspace-level)

## Open questions (tracked in `OPEN_QUESTIONS.md`)

- How much of the overflow/drop resolution should be shared with workspace tab strip?
- Should docking use explicit geometry surfaces everywhere, or rely on diagnostics for self-drawn cases?
