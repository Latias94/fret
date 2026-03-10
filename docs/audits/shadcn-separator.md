# shadcn/ui v4 Audit - Separator

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Separator` against the upstream shadcn/ui v4 base docs,
base examples, and the existing separator geometry/chrome gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/separator.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/separator.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/separator-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-vertical.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-menu.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-list.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-rtl.tsx`

## Fret implementation

- Primitive: `ecosystem/fret-ui-kit/src/primitives/separator.rs`
- shadcn re-export: `ecosystem/fret-ui-shadcn/src/separator.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/separator.rs`

## Audit checklist

### Authoring surface

- Pass: `Separator::new()` plus `orientation(...)` covers the documented separator surface.
- Pass: `Separator` remains a minimal leaf primitive; no extra generic `compose()` / `asChild` API is needed here.

### Layout & default-style ownership

- Pass: the 1px rule chrome remains recipe-owned on the separator primitive.
- Pass: surrounding width/height negotiation and row/column stretch behavior remain caller-owned composition.
- Pass: horizontal and vertical separator geometry remain covered by the existing web geometry/chrome gates.

### Gallery / docs parity

- Pass: the gallery mirrors the upstream base Separator docs path first: `Demo`, `Usage`, `Vertical`, `Menu`, `List`, `RTL`, and `API Reference`.
- Pass: the previous mixed notes are folded into `API Reference`, leaving the page source-comparable against the upstream docs headings.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- Existing chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`web_vs_fret_separator_demo_geometry_matches`)
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/separator.rs` (`web_vs_fret_layout_separator_demo_geometry`)
