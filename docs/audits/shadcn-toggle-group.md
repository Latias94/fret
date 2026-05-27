# shadcn/ui v4 Audit — Toggle Group

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `ToggleGroup` against the current shadcn/ui v4
new-york-v4 docs path, recipe source, Radix/Base UI semantics references, and the current
gallery/docs surface.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/toggle-group.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/toggle-group.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-group-spacing.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-group-outline.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-group-single.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-group-sm.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-group-lg.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-group-disabled.tsx`
- Tracked upstream goldens: `goldens/shadcn-web/v4/new-york-v4/toggle-group-demo.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-group-outline.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-group-single.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-group-sm.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-group-lg.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-group-disabled.json`, and `goldens/shadcn-web/v4/new-york-v4/toggle-group-spacing.json`
- Underlying primitives: Radix `@radix-ui/react-toggle-group`, Base UI `@base-ui/react/toggle-group`, and Fret `fret_ui_kit::primitives::toggle_group`

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
- Pass: The gallery now mirrors the current shadcn Toggle Group docs path first with source-aligned defaults and content: `Spacing`, `Usage`, `Outline`, `Single`, `Small`, `Large`, `Disabled`, and `API Reference`.
- Pass: The docs-path snippets no longer drift on the top spacing preview, outline icon set, split single/small/large examples, or disabled multiple-group styling.
- Pass: `Demo (Fret)`, `Vertical (Base/Radix)`, `Custom (Fret)`, `RTL (Fret)`, `Children (Fret)`, `Label Association (Fret)`, `Disabled Item Action-State (Fret)`, `Full Width Items (Fret)`, and `Flex-1 Items (Fret)` remain explicit Fret/base-radix follow-ups after the upstream path.
- Pass: `Children (Fret)` now teaches the helper-based composable-children lane without displacing the simpler docs-path `.items([...])` story.
- Pass: This work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail toggle_group`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_toggle --status-level fail toggle_group`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail toggle_group`
- `cargo nextest run -p fret-ui-shadcn --test snapshots --status-level fail snapshot_toggle_group_pressed_semantics`
- `cargo nextest run -p fret-ui-shadcn --test radix_web_primitives_state --status-level fail toggle_group`
- `cargo nextest run -p fret-ui-gallery --test toggle_group_docs_surface --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail toggle_group`
- Diagnostic smoke anchor: `tools/diag-scripts/ui-gallery/toggle/ui-gallery-toggle-group-docs-smoke.json`
