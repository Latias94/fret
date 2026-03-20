# shadcn/ui v4 Audit - Input Group

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `InputGroup` against the upstream shadcn/ui v4 base docs
and example implementations in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/input-group.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/input-group.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/input-group-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-inline-start.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-inline-end.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-block-start.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-block-end.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-icon.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-text.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-button.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-kbd.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-dropdown.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-spinner.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-textarea.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-custom.tsx`, `repo-ref/ui/apps/v4/examples/base/input-group-rtl.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/input_group.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/input_group.rs`

## Audit checklist

### Authoring surface

- Pass: the part-based API matches the upstream docs: `InputGroup`, `InputGroupAddon`, `InputGroupButton`, `InputGroupInput`, `InputGroupTextarea`, and `InputGroupText`.
- Pass: Fret also keeps the compact `InputGroup::new(model)` shorthand with `leading`, `trailing`, `block_start`, and `block_end` slots for ergonomic app-side authoring.
- Pass: `InputGroupAddon::align(...)` covers the documented addon placement surface without widening the mechanism layer.
- Pass: prebuilt/third-party controls now have a narrow recipe-layer seam via `custom_input(...)` / `custom_textarea(...)`, so the upstream `Custom Input` docs path is no longer a docs-only approximation.
- Pass: first-party gallery snippets now keep the compact shorthand as the default ergonomic lane
  (including the dropdown example), while the explicit addon/control parts remain the docs-parity
  lane rather than an advanced-only escape hatch.
- Pass: no generic root `children(...)` API is warranted; the component still benefits from typed
  structure (`InputGroupInput` / `InputGroupTextarea` / `InputGroupAddon`) plus the narrow
  caller-owned custom-control seam.

### Layout & default-style ownership

- Pass: root `w-full min-w-0` remains recipe-owned because the upstream source defines width negotiation on the component root.
- Pass: caller-owned refinements remain explicit for max width, surrounding grid/flex placement, and any page-level centering.
- Pass: button/text/kbd/dropdown/spinner/textarea/custom-input examples stay recipe-comparable without baking single-page constraints into the recipe defaults.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream docs path first: `Demo`, `Usage`, `Align`, the example set through `Custom Input`, `RTL`, and `API Reference`.
- Pass: the `Usage` section is now a real snippet-backed compact-shorthand example instead of a page-local hand-written Rust string, so the default first-party lane is copyable and compiled.
- Pass: `Tooltip`, `Label Association`, and `Button Group` remain explicit Fret follow-ups after the upstream path.
- Pass: the remaining gap was public-surface parity in `fret-ui-shadcn`, not a mechanism bug in `crates/fret-ui`; the custom-control seam now lands in the recipe layer.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- Existing shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`input-group-dropdown`, `input-group-custom`, and related `input-group-*` cases)
- Source-policy gate: `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` (`input_group_snippets_prefer_ui_cx_on_the_default_app_surface`, `selected_input_group_snippets_prefer_compact_slot_shorthand`)
- Docs smoke gate: `tools/diag-scripts/ui-gallery/input/ui-gallery-input-group-docs-smoke.json`
