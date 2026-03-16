# shadcn/ui v4 Audit — Collapsible

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Collapsible` against the upstream shadcn/ui v4 base docs,
base examples, and the current gallery/docs surface.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/collapsible.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/collapsible.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/collapsible-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/collapsible-basic.tsx`, `repo-ref/ui/apps/v4/examples/base/collapsible-settings.tsx`, `repo-ref/ui/apps/v4/examples/base/collapsible-file-tree.tsx`, `repo-ref/ui/apps/v4/examples/base/collapsible-rtl.tsx`
- Underlying primitive: Base UI `@base-ui/react/collapsible`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/collapsible.rs`
- Primitive/motion helpers: `ecosystem/fret-ui-kit/src/primitives/collapsible.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/collapsible.rs`

## Audit checklist

### Composition surface

- Pass: Provides `Collapsible`, `CollapsibleTrigger`, and `CollapsibleContent` wrappers.
- Pass: A source-aligned children surface is available via
  `fret_ui_shadcn::raw::collapsible::primitives`.
- Pass: The legacy flat module path `fret_ui_shadcn::collapsible_primitives` remains available for compatibility.
- Pass: No extra generic `compose()` API is needed here because the primitives surface already covers the free-form shadcn/Base UI composition model, while the top-level wrapper stays a compact Fret-first builder.
- Pass: Supports a controlled open state (`Model<bool>`).
- Pass: Supports uncontrolled `default_open` via `Collapsible::uncontrolled(...)`.

### A11y and motion behavior

- Pass: Trigger exposes the expected expanded/collapsed outcome.
- Pass: Trigger/content wiring can model the equivalent of `aria-controls` through the registered controls relationship.
- Pass: Measured open/close motion remains implementation-owned in the primitive/recipe layer; no mechanism-layer drift was identified in this pass.

### Gallery / docs parity

- Pass: The gallery now mirrors the upstream base Collapsible docs path first: `Demo`, `Usage`, `Controlled State`, `Basic`, `Settings Panel`, `File Tree`, `RTL`, and `API Reference`.
- Pass: The `Demo` section now matches the current shadcn/base docs order-details card rather than the older repository-list example.
- Pass: The `RTL` section now mirrors the current upstream order-details composition as an Arabic RTL card instead of a simplified English-only disclosure layout.
- Pass: The `Basic` section now matches the upstream `data-open:bg-muted` surface, `Learn More` CTA copy, and `xs` button size while still teaching the compact top-level wrapper lane.
- Pass: The `Settings Panel` section now keeps the extra inputs nested inside the left field column, uses the right-aligned outline icon trigger shape, and restores the upstream `0` default values for all four inputs.
- Pass: The `File Tree` section now mirrors the upstream explorer shell with `Explorer` / `Outline` tabs, the current shadcn docs data set, default-closed folders, and non-link-colored file rows.
- Pass: The `Usage` section keeps the source-aligned children/parts composition model explicit through `fret_ui_shadcn::raw::collapsible::primitives`, so no extra top-level `compose()` API is required.
- Pass: The raw `Collapsible` root now forwards caller-owned width into its internal stack, so `children` compositions that use `.w_full()` behave like the upstream flex-column root instead of collapsing to intrinsic content width.
- Pass: This work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-collapsible cargo check -p fret-ui-gallery --message-format short`
- `CARGO_TARGET_DIR=target-codex-collapsible cargo nextest run -p fret-ui-shadcn --lib -- collapsible`
- `CARGO_TARGET_DIR=target-codex-collapsible cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-collapsible-basic-double-click-close.json --dir target/diag-collapsible-codex --session-auto --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-collapsible-demo-order-details-shows-sections.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
- `CARGO_TARGET_DIR=target-codex-collapsible cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-collapsible-settings-open-shows-inputs.json --dir target/diag-collapsible-codex --session-auto --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
- `CARGO_TARGET_DIR=target-codex-collapsible cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-collapsible-file-tree-open-components-ui-shows-button.json --dir target/diag-collapsible-codex --session-auto --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-collapsible-rtl-open-scrolls-to-content.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
