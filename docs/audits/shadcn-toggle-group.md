# shadcn/ui v4 Audit — Toggle Group

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `ToggleGroup` against the upstream shadcn/ui v4 base docs,
base examples, and the current gallery/docs surface.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/toggle-group.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/toggle-group.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/toggle-group-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-group-outline.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-group-sizes.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-group-spacing.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-group-vertical.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-group-disabled.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-group-font-weight-selector.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-group-rtl.tsx`
- Underlying primitives: Base UI `@base-ui/react/toggle-group` + `@base-ui/react/toggle`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/toggle_group.rs`
- Related surfaces:
  - Toggle tokens: `ecosystem/fret-ui-shadcn/src/toggle.rs`
  - Roving focus policy: `ecosystem/fret-ui-kit/src/primitives/roving_focus_group.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/toggle_group.rs`

## Audit checklist

### Composition surface

- Pass: Supports `single` (`Model<Option<Arc<str>>>`) and `multiple` (`Model<Vec<Arc<str>>>`) modes.
- Pass: Supports uncontrolled default selection for both modes.
- Pass: Supports `orientation`, `loop_navigation`, `variant`, `size`, and `spacing(...)`.
- Pass: The default docs-path root surface remains `ToggleGroup::{single,multiple}*` plus `.items([...])`.
- Pass: The builder-preserving helper family `toggle_group_single(...)`, `toggle_group_single_uncontrolled(...)`, `toggle_group_multiple(...)`, and `toggle_group_multiple_uncontrolled(...)` now serves as the explicit composable-children lane on the Fret surface.
- Pass: `ToggleGroupItem::new(..., children)`, `child(...)`, and `children(...)` are sufficient for source-aligned item content composition; no extra root `children([...])` or generic `compose()` API is needed here.
- Pass: `ToggleGroupItem::refine_layout(...)` and `refine_style(...)` now cover upstream custom item-root sizing and rounding for card-like toggle items.
- Pass: `control_id(...)` and `test_id_prefix(...)` remain focused Fret follow-up surfaces rather than upstream docs-path requirements.

### Selection behavior

- Pass: Single mode deactivates when clicking the selected item (Base UI / shadcn single-toggle outcome).
- Pass: Multiple mode toggles membership per item value.
- Pass: Existing roving-focus behavior and test-id derivation remain covered by in-crate tests.

### Ownership and docs parity

- Pass: Selection semantics, roving focus, segmented borders, and pressed-state chrome remain recipe-owned.
- Pass: Item-root custom layout/chrome (`w/h`, radius) and surrounding width/flex negotiation remain caller-owned.
- Pass: The gallery now mirrors the upstream base Toggle Group docs path first with source-aligned defaults and content: `Demo`, `Usage`, `Outline`, `Size`, `Spacing`, `Vertical`, `Disabled`, `Custom`, `RTL`, and `API Reference`.
- Pass: The docs-path snippets no longer drift on the demo selection state, outline labels, size rows, spacing content, or disabled styling.
- Pass: `Children (Fret)`, `Single`, `Small`, `Large`, `Label Association`, `Full Width Items`, and `Flex-1 Items` remain explicit Fret follow-ups after the upstream path.
- Pass: `Children (Fret)` now teaches the helper-based composable-children lane without displacing the simpler docs-path `.items([...])` story.
- Pass: This work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `cargo nextest run -p fret-ui-gallery toggle_group_`
- `cargo nextest run -p fret-ui-shadcn toggle_group`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/toggle/ui-gallery-toggle-group-docs-smoke.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
- Existing chrome/layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`web_vs_fret_toggle_group_demo_chrome_matches`) and `ecosystem/fret-ui-shadcn/tests/web_vs_fret_toggle.rs` (`toggle-group-*` height cases)
