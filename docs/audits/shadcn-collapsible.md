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
- Base implementation: `repo-ref/ui/apps/v4/registry/bases/base/ui/collapsible.tsx`
- Current default visual/chrome implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/collapsible.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/collapsible-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/collapsible-basic.tsx`, `repo-ref/ui/apps/v4/examples/base/collapsible-settings.tsx`, `repo-ref/ui/apps/v4/examples/base/collapsible-file-tree.tsx`, `repo-ref/ui/apps/v4/examples/base/collapsible-rtl.tsx`
- Underlying primitive: Base UI `@base-ui/react/collapsible`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/collapsible.rs`
- Primitive/motion helpers: `ecosystem/fret-ui-kit/src/primitives/collapsible.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/collapsible.rs`

## Audit checklist

### Composition surface

- Pass: Provides `Collapsible`, `CollapsibleTrigger`, and `CollapsibleContent` wrappers.
- Pass: The curated facade now also exposes `CollapsibleRoot`, `CollapsibleTriggerPart`, and `CollapsibleContentPart` aliases so the copyable gallery usage lane can stay on `use fret_ui_shadcn::{facade as shadcn, prelude::*};`.
- Pass: A source-aligned children surface is available via
  `fret_ui_shadcn::raw::collapsible::primitives`.
- Pass: The legacy flat module path `fret_ui_shadcn::collapsible_primitives` remains available for compatibility.
- Pass: No extra generic `compose()` API is needed here because the parts/primitives surfaces already cover the free-form shadcn/Base UI composition model, while the top-level wrapper stays a compact Fret-first builder.
- Pass: Supports a controlled open state (`Model<bool>`).
- Pass: Supports uncontrolled `default_open` via `Collapsible::uncontrolled(...)`.
- Pass: `CollapsibleContentPart` keeps `gap=0` and start-aligned children by default, so width/flex negotiation stays caller-owned like the upstream unstyled panel surface instead of introducing an implicit stretch policy.

### A11y and motion behavior

- Pass: Trigger exposes the expected expanded/collapsed outcome.
- Pass: Trigger/content wiring can model the equivalent of `aria-controls` through the registered controls relationship.
- Pass: Measured open/close motion remains implementation-owned in the primitive/recipe layer; no mechanism-layer drift was identified in this pass.

### Gallery / docs parity

- Pass: The gallery now mirrors the upstream base Collapsible docs path first: `Demo`, `Usage`, `Controlled State`, `Basic`, `Settings Panel`, `File Tree`, `RTL`, and `API Reference`.
- Pass: The `Demo` section now matches the official shadcn repository-list example rather than the older order-details card.
- Pass: The `RTL` section now mirrors the current upstream repository-list composition as an Arabic RTL disclosure layout instead of the earlier simplified example.
- Pass: The `Basic` section now matches the upstream `data-open:bg-muted` surface, `Learn More` CTA copy, and `xs` button size while still teaching the compact top-level wrapper lane.
- Pass: The `Settings Panel` section now keeps the extra inputs nested inside the left field column, uses the right-aligned outline icon trigger shape, and restores the upstream `0` default values for all four inputs.
- Pass: The `File Tree` section now mirrors the upstream explorer shell with `Explorer` / `Outline` tabs, the current shadcn docs data set, default-closed folders, and non-link-colored file rows.
- Pass: The `Usage` section keeps the source-aligned children/parts composition model explicit on the curated facade through `CollapsibleRoot` / `CollapsibleTriggerPart` / `CollapsibleContentPart`, while `fret_ui_shadcn::raw::collapsible::primitives` remains the explicit escape hatch.
- Pass: The page now restores a dedicated `Notes` section that records the source axes, the current children-API decision, and the “docs/public-surface alignment rather than a mechanism bug” conclusion used by diagnostics and review.
- Pass: A promoted docs smoke script now checks the docs-path sections plus `Notes`, so the existing notes-focused diagnostics no longer drift away from the page structure.
- Pass: The raw `Collapsible` root now forwards caller-owned width into its internal stack, so `children` compositions that use `.w_full()` behave like the upstream flex-column root instead of collapsing to intrinsic content width.
- Pass: This work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-collapsible cargo check -p fret-ui-gallery --message-format short`
- `CARGO_TARGET_DIR=target-codex-collapsible cargo nextest run -p fret-ui-gallery --test collapsible_docs_surface`
- `CARGO_TARGET_DIR=target-codex-collapsible cargo nextest run -p fret-ui-shadcn --lib -- collapsible`
- `CARGO_TARGET_DIR=target-codex-collapsible cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery-collapsible-docs-smoke.json --dir target/diag-collapsible-docs-codex --session-auto --pack --ai-packet --launch -- env CARGO_TARGET_DIR=target-codex-collapsible cargo run -p fret-ui-gallery`
- `CARGO_TARGET_DIR=target-codex-collapsible cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery-collapsible-basic-double-click-close.json --dir target/diag-collapsible-codex --session-auto --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery-collapsible-demo-repository-list-shows-items.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
- `CARGO_TARGET_DIR=target-codex-collapsible cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery-collapsible-settings-open-shows-inputs.json --dir target/diag-collapsible-codex --session-auto --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
- `CARGO_TARGET_DIR=target-codex-collapsible cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery-collapsible-file-tree-open-components-ui-shows-button.json --dir target/diag-collapsible-codex --session-auto --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery-collapsible-rtl-open-scrolls-to-content.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
