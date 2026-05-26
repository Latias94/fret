# shadcn/ui v4 Audit - Item

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Item` against the upstream shadcn/ui v4 docs,
new-york-v4 examples, and the existing item layout gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/item.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/item.tsx`
- Example compositions:
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/item-demo.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/item-variant.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/item-size.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/item-icon.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/item-avatar.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/item-image.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/item-group.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/item-header.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/item-link.tsx`, and
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/item-dropdown.tsx`.
- Existing layout gates: `goldens/shadcn-web/v4/new-york-v4/item-demo.json`,
  `goldens/shadcn-web/v4/new-york-v4/item-variant.json`,
  `goldens/shadcn-web/v4/new-york-v4/item-size.json`,
  `goldens/shadcn-web/v4/new-york-v4/item-icon.json`,
  `goldens/shadcn-web/v4/new-york-v4/item-avatar.json`,
  `goldens/shadcn-web/v4/new-york-v4/item-image.json`,
  `goldens/shadcn-web/v4/new-york-v4/item-group.json`,
  `goldens/shadcn-web/v4/new-york-v4/item-header.json`,
  `goldens/shadcn-web/v4/new-york-v4/item-link.json`,
  `goldens/shadcn-web/v4/new-york-v4/item-dropdown.json`, and
  `goldens/shadcn-web/v4/new-york-v4/item-dropdown.open.json`.

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

- Pass: the gallery now mirrors the upstream Item docs path first: `Demo`, `Usage`,
  `Item vs Field`, `Variant`, `Size`, explicit `Examples`, the example set through `Dropdown`,
  and `API Reference`; the RTL example remains an explicit Fret follow-up because the current
  upstream Item docs do not include an RTL example.
- Pass: the page now keeps the upstream `Examples` grouping explicit before splitting `Icon`, `Avatar`, `Image`, `Group`, `Header`, `Link`, and `Dropdown` into separately previewable sections.
- Pass: `RTL`, `Gallery`, and `Link (render)` remain explicit Fret follow-ups after the upstream
  path because they document extra deterministic coverage and gallery-focused authoring shapes.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `python -m json.tool tools/diag-scripts/suites/ui-gallery-item-demo-action-state/suite.json | Out-Null`
- `python -m json.tool tools/diag-scripts/suites/ui-gallery-item-link-action-state/suite.json | Out-Null`
- `Get-ChildItem -Path tools/diag-scripts/ui-gallery/item -Filter *.json | ForEach-Object { python -m json.tool $_.FullName | Out-Null }`
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail item`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail item`
- `cargo nextest run -p fret-ui-gallery --test item_docs_surface --status-level fail`
- `cargo nextest run -p fret-ui-gallery --lib --status-level fail gallery_item`
