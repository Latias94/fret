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

- [x] AUE-chrome-010 Add debug-only builder helpers (debug border) consistent with shadcn token names.
  - Evidence: `ecosystem/fret-ui-kit/src/style/chrome.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-chrome-011 Add a kit-level “focused border/ring” preset usable by multiple shadcn components.
  - Evidence: `ecosystem/fret-ui-kit/src/style/chrome.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-chrome-012 Add per-corner radius refinement support (`corner_radii(...)` or `rounded_tl/...`).
  - Evidence: `ecosystem/fret-ui-kit/src/corners4.rs`, `ecosystem/fret-ui-kit/src/style/chrome.rs`
  - Builder: `ecosystem/fret-ui-kit/src/ui_builder.rs`
  - Resolution: `ecosystem/fret-ui-kit/src/declarative/style.rs`
- [x] AUE-chrome-013 Add shadow shorthands to the `ui()` chain (e.g. `shadow_sm/md/lg`).
  - Evidence: `ecosystem/fret-ui-kit/src/style/chrome.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
  - Resolution: `ecosystem/fret-ui-kit/src/declarative/style.rs`

## C. Layout Constructors (Reduce Props Noise)

- [x] AUE-layout-020 Add `ui::h_flex(...)` / `ui::v_flex(...)` constructors in `fret-ui-kit` that return a patchable builder.
  - Evidence: `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-layout-021 Add a minimal `ui::stack(...)` constructor (overlay composition helper; optional).
  - Evidence: `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-layout-022 Add “gap” and alignment shorthands on the layout constructor path (not only on components).
  - Evidence: `ecosystem/fret-ui-kit/src/ui_builder.rs`

## D. Surface Consolidation

- [x] AUE-surface-030 Decide whether `ecosystem/fret-ui-kit/src/styled.rs` should be:
  - deprecated in favor of `ui()`, or
  - expanded to be a thin alias over `UiBuilder`, or
  - kept intentionally tiny (and documented as such).
  - Decision: keep intentionally tiny + chrome-only; do not expand.
  - Evidence: `docs/workstreams/authoring-ergonomics-fluent-builder.md`
- [x] AUE-surface-031 Ensure `fret-ui-shadcn` prelude re-exports the single recommended authoring chain.
  - Evidence: `ecosystem/fret-ui-shadcn/src/lib.rs`

## E. Documentation / Adoption

- [x] AUE-docs-040 Add an “Authoring Golden Path” section with before/after examples in `docs/shadcn-declarative-progress.md`.
  - Evidence: `docs/shadcn-declarative-progress.md`
- [x] AUE-docs-041 Add a short cookbook for layout-only authoring (flex/grid/stack) using the new constructors.
  - Evidence: `docs/shadcn-declarative-progress.md`

## F. (Future) Proc-macro / Derive

- [ ] AUE-macro-050 Re-audit `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md` and decide the minimal derive set:
  - `IntoElement` for props structs
  - `RenderOnce` boilerplate reduction
  - ergonomics for children slots

## G. Parity / Presets (High ROI)

- [x] AUE-parity-060 Add a shadcn-level `popover_style` preset that can be applied from the `ui()` chain (policy layer).
  - Evidence: `ecosystem/fret-ui-shadcn/src/ui_builder_ext/surfaces.rs`, `ecosystem/fret-ui-shadcn/src/popover.rs`
- [x] AUE-parity-061 Add a shadcn-level `dialog_style` preset that can be applied from the `ui()` chain (policy layer).
  - Evidence: `ecosystem/fret-ui-shadcn/src/ui_builder_ext/surfaces.rs`, `ecosystem/fret-ui-shadcn/src/dialog.rs`
- [x] AUE-parity-062 Add shadcn-level `menu_style` / `menu_sub_style` presets that can be applied from the `ui()` chain (policy layer).
  - Evidence: `ecosystem/fret-ui-shadcn/src/ui_builder_ext/surfaces.rs`, `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- [x] AUE-layout-023 Add a patchable `ui::container(...)` constructor as the default “box” layout node.
  - Intent: reduce raw `cx.container(...)` usage in cookbook/recipes.
  - Evidence: `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [ ] AUE-text-070 Add a minimal patchable `ui::text(...)` / `ui::label(...)` authoring constructor with a small typed refinement surface.
  - Scope: size/weight/color + a shadcn-aligned default line-height.
  - Evidence target: `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/declarative/text.rs`
