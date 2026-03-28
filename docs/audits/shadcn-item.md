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
- Component implementation: `repo-ref/ui/apps/v4/registry/bases/base/ui/item.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/item-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/item-variant.tsx`, `repo-ref/ui/apps/v4/examples/base/item-size.tsx`, `repo-ref/ui/apps/v4/examples/base/item-icon.tsx`, `repo-ref/ui/apps/v4/examples/base/item-avatar.tsx`, `repo-ref/ui/apps/v4/examples/base/item-image.tsx`, `repo-ref/ui/apps/v4/examples/base/item-group.tsx`, `repo-ref/ui/apps/v4/examples/base/item-header.tsx`, `repo-ref/ui/apps/v4/examples/base/item-link.tsx`, `repo-ref/ui/apps/v4/examples/base/item-dropdown.tsx`, `repo-ref/ui/apps/v4/examples/base/item-rtl.tsx`
- Existing layout gates: `goldens/shadcn-web/v4/new-york-v4/item-demo.json`, `goldens/shadcn-web/v4/new-york-v4/item-size.json`, `goldens/shadcn-web/v4/new-york-v4/item-avatar.json`, `goldens/shadcn-web/v4/new-york-v4/item-link.json`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/item.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/item.rs`

## Audit checklist

### Authoring surface

- Pass: `Item::new([...])` plus `ItemMedia`, `ItemContent`, `ItemTitle`, `ItemDescription`, `ItemActions`, `ItemGroup`, and `ItemHeader` matches the upstream slot model directly.
- Pass: `ItemRender::Link` is the right Fret equivalent of the upstream `render={<a ... />}` pattern and keeps link semantics on the pressable root.
- Pass: `Item::new([...])` already serves as the composable heterogeneous-children root lane, so no extra generic root `children(...)` / `compose()` surface is warranted here.
- Pass: `ItemTitle::new_children([...])` and `ItemDescription::new_children([...])` now keep slot-local rich text / composed-child authoring available without widening the root `Item` API beyond the documented link-render lane.
- Pass: avatar rows stay slot-composed (`ItemMedia` + `Avatar`) rather than growing a separate `ItemMediaVariant::Avatar`; this follows the upstream registry source even though the docs prose loosely names an "avatar" lane.
- Pass: `ItemSize::Xs` is already supported in Fret and is now surfaced explicitly by the gallery size example.
- Pass: no extra generic `asChild` / `compose()` API is needed here.

### Layout & default-style ownership

- Pass: intrinsic item chrome, slot spacing, media sizing, and size presets remain recipe-owned because the upstream component source defines those defaults on the item itself.
- Pass: media parts now self-start with a small top offset when an `ItemDescription` is present, matching the upstream `group-has(...):self-start translate-y-0.5` outcome at the recipe layer.
- Pass: surrounding width caps, page columns, grid placement, and mixed-list layouts remain caller-owned composition.
- Pass: existing item web layout gates continue to cover representative geometry for `item-demo`, `item-size`, `item-avatar`, and `item-link`.
- Pass: `ItemGroup` continues to own only the list-container semantics; per-row `listitem` semantics remain caller-owned because item rows may also need stronger interactive roles such as `link`, and the upstream source does not define a stable default here.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream base Item docs path first: `Demo`, `Usage`, `Item vs Field`, `Variant`, `Size`, explicit `Examples`, the example set through `RTL`, and `API Reference`.
- Pass: the page now keeps the upstream `Examples` grouping explicit before splitting `Icon`, `Avatar`, `Image`, `Group`, `Header`, `Link`, and `Dropdown` into separately previewable sections.
- Pass: `Gallery` and `Link (render)` remain explicit Fret follow-ups after the upstream path because they document extra deterministic coverage and gallery-focused authoring shapes.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/item/ui-gallery-item-docs-smoke.json --dir target/fret-diag-item-docs --session-auto --launch -- cargo run -p fret-ui-gallery`
- Existing layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/item.rs` (`web_vs_fret_layout_item_avatar_geometry`, `web_vs_fret_layout_item_demo_item_rects_match_web`, `web_vs_fret_layout_item_size_item_rects_match_web`, `web_vs_fret_layout_item_link_item_rects_match_web`)
