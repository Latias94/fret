# Authoring Ergonomics — Fluent Builder TODOs (v1)

Status: Active

This tracker focuses on authoring ergonomics improvements that stay within Fret’s layering rules.

Related:

- Design note: `docs/workstreams/authoring-ergonomics-fluent-builder.md`
- Shadcn progress: `docs/shadcn-declarative-progress.md`

## Tracking Format

- ID: `AUE-{area}-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

## A. Fluent “Edges” Helpers

- [x] AUE-helpers-001 Add `UiBuilder::paddings(...)` that accepts a single token-aware 4-edge value.
  - Evidence: `ecosystem/fret-ui-kit/src/edges4.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-helpers-002 Add `UiBuilder::margins(...)` that accepts a single token-aware 4-edge value (supports `auto`).
  - Evidence: `ecosystem/fret-ui-kit/src/edges4.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-helpers-003 Add `UiBuilder::insets(...)` (positioning) that accepts a token-aware 4-edge value and supports
  signed/negative values via `SignedMetricRef`.
  - Evidence: `ecosystem/fret-ui-kit/src/edges4.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`

## B. Chrome Presets (Discoverable Recipes)

- [ ] AUE-chrome-010 Add debug-only builder helpers (`debug_border_red/blue/...`) consistent with shadcn token names.
- [ ] AUE-chrome-011 Add a kit-level “focused border/ring” preset usable by multiple shadcn components.
- [ ] AUE-chrome-012 Add per-corner radius refinement support (`corner_radii(...)` or `rounded_tl/...`).
- [ ] AUE-chrome-013 Add shadow shorthands to the `ui()` chain (e.g. `shadow_sm/md/lg`).

## C. Layout Constructors (Reduce Props Noise)

- [ ] AUE-layout-020 Add `ui::h_flex(...)` / `ui::v_flex(...)` constructors in `fret-ui-kit` that return a patchable builder.
- [ ] AUE-layout-021 Add a minimal `ui::stack(...)` constructor (overlay composition helper; optional).
- [ ] AUE-layout-022 Add “gap” and alignment shorthands on the layout constructor path (not only on components).

## D. Surface Consolidation

- [ ] AUE-surface-030 Decide whether `ecosystem/fret-ui-kit/src/styled.rs` should be:
  - deprecated in favor of `ui()`, or
  - expanded to be a thin alias over `UiBuilder`, or
  - kept intentionally tiny (and documented as such).
- [ ] AUE-surface-031 Ensure `fret-ui-shadcn` prelude re-exports the single recommended authoring chain.

## E. Documentation / Adoption

- [ ] AUE-docs-040 Add an “Authoring Golden Path” section with before/after examples in `docs/shadcn-declarative-progress.md`.
- [ ] AUE-docs-041 Add a short cookbook for layout-only authoring (flex/grid/stack) using the new constructors.

## F. (Future) Proc-macro / Derive

- [ ] AUE-macro-050 Re-audit `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md` and decide the minimal derive set:
  - `IntoElement` for props structs
  - `RenderOnce` boilerplate reduction
  - ergonomics for children slots
