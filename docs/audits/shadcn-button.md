# shadcn/ui v4 Audit - Button

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- MUI Base UI: https://github.com/mui/base-ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Button` against the upstream shadcn/ui v4 docs and base
example implementations in `repo-ref/ui`, using Base UI as an additional headless reference for the
semantic-link and cursor caveats.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/button.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/button.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/button-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/button-size.tsx`, `repo-ref/ui/apps/v4/examples/base/button-default.tsx`, `repo-ref/ui/apps/v4/examples/base/button-outline.tsx`, `repo-ref/ui/apps/v4/examples/base/button-secondary.tsx`, `repo-ref/ui/apps/v4/examples/base/button-ghost.tsx`, `repo-ref/ui/apps/v4/examples/base/button-destructive.tsx`, `repo-ref/ui/apps/v4/examples/base/button-link.tsx`, `repo-ref/ui/apps/v4/examples/base/button-icon.tsx`, `repo-ref/ui/apps/v4/examples/base/button-with-icon.tsx`, `repo-ref/ui/apps/v4/examples/base/button-rounded.tsx`, `repo-ref/ui/apps/v4/examples/base/button-spinner.tsx`, `repo-ref/ui/apps/v4/examples/base/button-render.tsx`, `repo-ref/ui/apps/v4/examples/base/button-rtl.tsx`
- Base UI references: `repo-ref/base-ui/packages/react/src/button`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/button.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/button.rs`

## Audit checklist

### Authoring surface

- Pass: `Button::new(label)` plus `variant(...)` covers the documented `default`, `outline`, `secondary`, `ghost`, `destructive`, and `link` recipe surface.
- Pass: `size(...)` covers the documented `default`, `xs`, `sm`, `lg`, `icon`, `icon-xs`, `icon-sm`, and `icon-lg` options.
- Pass: `leading_children(...)` / `trailing_children(...)` now cover the upstream `data-icon="inline-start|inline-end"` child-composition path for dynamic affordances such as `Spinner`, without forcing authors into a full content override.
- Pass: `ButtonRender::Link` is the Fret equivalent of the second upstream `Link` section; semantic link rendering stays button-owned instead of widening the public surface with a generic `asChild`/`compose()` API.

### Layout & default-style ownership

- Pass: recipe-owned defaults cover the intrinsic button chrome: height, horizontal/vertical padding, typography, `rounded-md`, variant colors, focus ring, and disabled styling.
- Pass: logical inline child slots now compact only the occupied inline side and reverse correctly under RTL, matching the intent of upstream `data-icon="inline-start|inline-end"` examples more closely than the old symmetric-compact fallback.
- Pass: caller-owned refinements stay explicit for page/container negotiation (`w-full`, `min-w-0`, `flex-1`, wrapping, max width) and for one-off call-site polish such as the `rounded-full` examples.
- Note: the upstream `Cursor` section is Tailwind CSS-specific; in Fret, hover cursor policy belongs to runtime / pressable behavior, not the `Button` recipe defaults.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream docs path first: `Demo`, `Usage`, `Cursor`, `Size`, `Default`, `Outline`, `Secondary`, `Ghost`, `Destructive`, `Link`, `Icon`, `With Icon`, `Rounded`, `Spinner`, `Button Group`, `Link (Semantic)`, and `RTL`, followed by `API Reference`.
- Pass: the second upstream `Link` section is represented explicitly as `Link (Semantic)` so authors can compare the semantic-link path without confusing it with the `link` variant.
- Pass: `Variants Overview (Fret)` remains after `API Reference` to preserve compact visual comparison and existing diagnostics without displacing the docs-first order.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- `cargo test -p fret-ui-shadcn button_inline_slot`
- Existing shadcn-web gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`button-demo`)
- Existing diag scripts: `tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-with-icon-non-overlap.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-variants-width-zinc-dark.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-loading-screenshots-zinc-dark.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-loading-screenshots-zinc-light.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-rtl-row-screenshots.json`
