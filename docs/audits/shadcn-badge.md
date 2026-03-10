# shadcn/ui v4 Audit - Badge

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Badge` against the upstream shadcn/ui v4 base docs,
base examples, and the existing badge chrome/layout gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/badge.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/badge.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/badge-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/badge-variants.tsx`, `repo-ref/ui/apps/v4/examples/base/badge-icon.tsx`, `repo-ref/ui/apps/v4/examples/base/badge-spinner.tsx`, `repo-ref/ui/apps/v4/examples/base/badge-link.tsx`, `repo-ref/ui/apps/v4/examples/base/badge-colors.tsx`, `repo-ref/ui/apps/v4/examples/base/badge-rtl.tsx`
- Existing gates: `goldens/shadcn-web/v4/new-york-v4/badge-demo.json`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/badge.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/badge.rs`

## Audit checklist

### Authoring surface

- Pass: `Badge::new(label)` plus `variant(...)` covers the documented badge surface, including `default`, `secondary`, `destructive`, `outline`, `ghost`, and `link`.
- Pass: `BadgeRender::Link` is the right Fret equivalent of the upstream `render` / `asChild` outcome and keeps link semantics on the badge-owned render surface.
- Pass: icons, spinners, and custom color overrides remain recipe-owned additions on the badge surface; no extra generic `compose()` API is needed here.

### Layout & default-style ownership

- Pass: padding, radius, font weight, shrink behavior, and intrinsic chrome remain recipe-owned because the upstream badge source defines them on the component itself.
- Pass: surrounding width negotiation and row placement remain caller-owned.
- Pass: badge height/chrome and the `font-medium` / `shrink-0` defaults remain covered by the existing web/layout and unit-test gates.

### Gallery / docs parity

- Pass: the gallery mirrors the upstream base Badge docs path first: `Demo`, `Usage`, `Variants`, `With Icon`, `With Spinner`, `Link`, `Custom Colors`, `RTL`, and `API Reference`.
- Pass: `Counts (Fret)` remains an explicit follow-up after the upstream path so compact numeric badge diagnostics stay stable.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- Existing chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`web_vs_fret_badge_demo_chrome_matches`)
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/badge.rs` (`web_vs_fret_layout_badge_demo_heights`)
- Existing unit test: `ecosystem/fret-ui-shadcn/src/badge.rs` (`badge_defaults_to_font_medium_and_shrink_0`)
