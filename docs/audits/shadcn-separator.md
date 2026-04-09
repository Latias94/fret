# shadcn/ui v4 Audit - Separator

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Separator` against the current shadcn/ui base docs,
the base/radix registry recipes, the upstream Radix/Base UI primitives, and the existing separator
geometry/chrome gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/separator.mdx`
- Base recipe source: `repo-ref/ui/apps/v4/registry/bases/base/ui/separator.tsx`
- Radix recipe source: `repo-ref/ui/apps/v4/registry/bases/radix/ui/separator.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/separator-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-vertical.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-menu.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-list.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-rtl.tsx`
- Headless references: `repo-ref/primitives/packages/react/separator/src/separator.tsx`, `repo-ref/base-ui/packages/react/src/separator/Separator.tsx`

## Fret implementation

- Primitive: `ecosystem/fret-ui-kit/src/primitives/separator.rs`
- shadcn re-export: `ecosystem/fret-ui-shadcn/src/separator.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/separator.rs`

## Audit checklist

### Authoring surface

- Pass: `Separator::new()` plus `orientation(...)` and `decorative(...)` covers the Fret-facing
  shadcn surface.
- Pass: `Separator` remains a minimal leaf primitive; no extra generic `compose()` / `asChild` /
  children API is needed here because composition happens around the separator, not through it.

### Layout & default-style ownership

- Pass: the 1px rule chrome remains recipe-owned on the separator primitive.
- Fixed: the vertical shadcn recipe now maps upstream `data-vertical:self-stretch` to
  `align-self: stretch` plus auto cross-axis sizing instead of a fill-height default.
- Pass: surrounding width/height negotiation remains caller-owned composition.
- Pass: horizontal and vertical separator geometry remain covered by the existing web
  geometry/chrome gates.

### Gallery / docs parity

- Fixed: the gallery `Demo` snippet now mirrors the upstream top preview instead of bundling the
  vertical example into the same section.
- Fixed: the `Menu` snippet now follows the upstream responsive composition more closely and no
  longer hard-codes a vertical separator height.
- Fixed: `API Reference` now documents the source-axis split between shadcn base docs, Radix
  primitive semantics, and Base UI's headless surface so the `decorative(...)` knob is explained
  instead of looking accidental.
- Conclusion: this pass found a shadcn recipe/docs-surface mismatch, not a missing mechanism-layer
  substrate.

## Validation

- `cargo nextest run -p fret-ui-shadcn --lib separator_defaults_match_shadcn_constraints vertical_separator_matches_shadcn_self_stretch vertical_separator_fill_height_lane_is_explicit_opt_out semantic_vertical_separator_is_opt_in --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test separator_docs_surface --status-level fail`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/separator/ui-gallery-separator-docs-smoke.json --dir target/fret-diag-separator --session-auto --pack --ai-packet --timeout-ms 240000 --launch -- cargo run -p fret-ui-gallery --release`
- Existing chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`web_vs_fret_separator_demo_geometry_matches`)
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/separator.rs` (`web_vs_fret_layout_separator_demo_geometry`)
