# shadcn/ui v4 Audit - Item

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Item` against the upstream shadcn/ui v4 base docs,
base examples, and the existing item layout gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/item.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/item.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/item-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/item-variant.tsx`, `repo-ref/ui/apps/v4/examples/base/item-size.tsx`, `repo-ref/ui/apps/v4/examples/base/item-icon.tsx`, `repo-ref/ui/apps/v4/examples/base/item-avatar.tsx`, `repo-ref/ui/apps/v4/examples/base/item-image.tsx`, `repo-ref/ui/apps/v4/examples/base/item-group.tsx`, `repo-ref/ui/apps/v4/examples/base/item-header.tsx`, `repo-ref/ui/apps/v4/examples/base/item-link.tsx`, `repo-ref/ui/apps/v4/examples/base/item-dropdown.tsx`, `repo-ref/ui/apps/v4/examples/base/item-rtl.tsx`
- Existing layout gates: `goldens/shadcn-web/v4/new-york-v4/item-demo.json`, `goldens/shadcn-web/v4/new-york-v4/item-size.json`, `goldens/shadcn-web/v4/new-york-v4/item-avatar.json`, `goldens/shadcn-web/v4/new-york-v4/item-link.json`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/item.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/item.rs`

## Audit checklist

### Authoring surface

- Pass: `Item::new([...])` plus `ItemMedia`, `ItemContent`, `ItemTitle`, `ItemDescription`, `ItemActions`, `ItemGroup`, and `ItemHeader` matches the upstream slot model directly.
- Pass: `ItemRender::Link` is the right Fret equivalent of the upstream `render={<a ... />}` pattern and keeps link semantics on the pressable root.
- Pass: `ItemSize::Xs` is already supported in Fret and is now surfaced explicitly by the gallery size example.
- Pass: no extra generic `asChild` / `compose()` API is needed here.

### Layout & default-style ownership

- Pass: intrinsic item chrome, slot spacing, media sizing, and size presets remain recipe-owned because the upstream component source defines those defaults on the item itself.
- Pass: surrounding width caps, page columns, grid placement, and mixed-list layouts remain caller-owned composition.
- Pass: existing item web layout gates continue to cover representative geometry for `item-demo`, `item-size`, `item-avatar`, and `item-link`.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream base Item docs path first: `Demo`, `Usage`, `Item vs Field`, `Variant`, `Size`, the example set through `RTL`, and `API Reference`.
- Pass: `Gallery` and `Link (render)` remain explicit Fret follow-ups after the upstream path because they document extra deterministic coverage and gallery-focused authoring shapes.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- Existing layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/item.rs` (`web_vs_fret_layout_item_avatar_geometry`, `web_vs_fret_layout_item_demo_item_rects_match_web`, `web_vs_fret_layout_item_size_item_rects_match_web`, `web_vs_fret_layout_item_link_item_rects_match_web`)
