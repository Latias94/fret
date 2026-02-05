# Semantics Decorators Adoption (ADR 1161) — TODO Tracker

Status: Active (workstream tracker; keep updated as migrations land)

This document tracks the migration from layout-affecting `Semantics` wrappers (used only for
stamping `test_id` / a11y labels) to the layout-transparent `attach_semantics` decorators
introduced by ADR 1161.

- ADR: `docs/adr/1161-semantics-decorators-and-attach-semantics-v1.md`
- API: `crates/fret-ui/src/element.rs::AnyElement::attach_semantics`

Tracking format:

- ID: `SDC-{area}-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

---

## Why this workstream exists

`Semantics` is a real layout wrapper (it carries `SemanticsProps.layout`). Using it only to stamp
`test_id` is a recurring footgun:

- inserts a new layout node,
- breaks common Tailwind/shadcn flex sizing chains (`flex-1`, `basis-0`, `min-w-0`),
- causes subtle geometry regressions (e.g. slider range/fill collapsing, text truncation drift),
- makes tests brittle (heuristics-based node picking instead of `test_id` selectors).

ADR 1161 introduces `attach_semantics` to stamp semantics on an existing element without changing
layout. This workstream is about making that the default across ecosystem code.

---

## Migration rules (hard constraints)

- Use `attach_semantics` when the intent is stamping only:
  - `test_id`
  - `role` override
  - `label` override
  - `value` override
- Use `Semantics` only when you need a semantics node boundary or features not supported by v1
  decorators (focusable/disabled/selected/expanded/checked/relations like labelled-by).
- Prefer `SemanticFlex` when the desired semantics node is the flex container itself (structural
  grouping), to avoid `Semantics(Flex(...))` wrapper layering.

---

## Milestones

### M0 — Tooling + conventions

- [x] SDC-tool-010 Add a small, shared helper for `test_id` stamping (suffixing + optional prefix).
  - Target: `ecosystem/fret-ui-shadcn` (and optionally `fret-ui-kit`).
  - Outcome: call sites become a one-liner and all IDs follow the same naming scheme.
  - Evidence: `ecosystem/fret-ui-shadcn/src/test_id.rs`
- [ ] SDC-tool-020 Add an audit recipe (rg patterns) and “exceptions list” section below.
  - Outcome: quick repo-wide scans can be done without guesswork.

### M1 — shadcn: remove “test_id-only Semantics wrappers”

- [x] SDC-shadcn-100 Slider: migrate `*-track/*-range/*-thumb-*` stamping to `attach_semantics`.
  - Delete any layout workarounds that exist only because of wrapper semantics.
  - Evidence: `ecosystem/fret-ui-shadcn/src/slider.rs`
- [x] SDC-shadcn-101 ScrollArea: migrate viewport `test_id` stamping to `attach_semantics`.
  - Evidence: `ecosystem/fret-ui-shadcn/src/scroll_area.rs`
- [x] SDC-shadcn-102 Select: migrate scroll viewport `test_id` stamping to `attach_semantics`.
  - Evidence: `ecosystem/fret-ui-shadcn/src/select.rs`
- [x] SDC-shadcn-103 Card: migrate `CardAction` stamping to `attach_semantics` (avoid `Semantics` wrappers for slots).
  - Evidence: `ecosystem/fret-ui-shadcn/src/card.rs`
- [x] SDC-shadcn-110 DataGrid: migrate header/body cell stamps to decorators where possible.
  - Evidence: `ecosystem/fret-ui-shadcn/src/data_grid.rs`, `ecosystem/fret-ui-shadcn/tests/data_grid_layout.rs`
- [ ] SDC-shadcn-120 Form/Field: migrate any “label-only” wrappers used for targeting.

### M2 — Tests: converge on `test_id` selectors

- [ ] SDC-test-200 Replace geometry heuristics that implicitly depend on wrapper shapes.
  - Preferred: select nodes via semantics snapshot `test_id` rather than scanning `debug_node_bounds`.
- [ ] SDC-test-210 Add at least one regression test per migrated component that asserts bounds are
  non-zero for the stamped nodes (detects wrapper-induced collapse).

### M3 — UI Gallery: migrate authoring patterns

- [ ] SDC-gal-300 Update UI Gallery component previews to avoid `Semantics`-for-test-id patterns.
- [ ] SDC-gal-310 Add notes to the shadcn UI Gallery tracker pointing to this workstream.

---

## Audit recipes

Quick scans to find problematic patterns:

- `cx.semantics(` where `SemanticsProps` only sets `test_id` (and otherwise defaults)
- `SemanticsProps { test_id: ... }` wrapping a single child and not adding structure

Commands:

- `rg -n \"cx\\.semantics\\(\" ecosystem/fret-ui-shadcn ecosystem/fret-ui-kit apps`
- `rg -n \"SemanticsProps\\s*\\{\" ecosystem/fret-ui-shadcn ecosystem/fret-ui-kit apps`

---

## Exceptions (intentional wrappers)

Document any case where `Semantics` remains required (and why). Keep this list small and explicit.

- (none yet)
