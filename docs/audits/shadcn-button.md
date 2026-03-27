# shadcn/ui v4 Audit - Button

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- MUI Base UI: https://github.com/mui/base-ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Button` against the upstream shadcn/ui v4 docs and
registry sources in `repo-ref/ui`, using Base UI as an additional headless reference for
semantic-link caveats and the underlying button primitive contract.

## Upstream references (source of truth)

- Docs pages: `repo-ref/ui/apps/v4/content/docs/components/base/button.mdx`, `repo-ref/ui/apps/v4/content/docs/components/radix/button.mdx`
- Visual/chrome baseline: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/button.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/button-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/button-size.tsx`, `repo-ref/ui/apps/v4/examples/base/button-default.tsx`, `repo-ref/ui/apps/v4/examples/base/button-outline.tsx`, `repo-ref/ui/apps/v4/examples/base/button-secondary.tsx`, `repo-ref/ui/apps/v4/examples/base/button-ghost.tsx`, `repo-ref/ui/apps/v4/examples/base/button-destructive.tsx`, `repo-ref/ui/apps/v4/examples/base/button-link.tsx`, `repo-ref/ui/apps/v4/examples/base/button-icon.tsx`, `repo-ref/ui/apps/v4/examples/base/button-with-icon.tsx`, `repo-ref/ui/apps/v4/examples/base/button-rounded.tsx`, `repo-ref/ui/apps/v4/examples/base/button-spinner.tsx`, `repo-ref/ui/apps/v4/examples/base/button-render.tsx`, `repo-ref/ui/apps/v4/examples/base/button-rtl.tsx`, `repo-ref/ui/apps/v4/examples/radix/button-aschild.tsx`
- Base UI references: `repo-ref/base-ui/packages/react/src/button/Button.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/button.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/button.rs`

## Audit checklist

### Authoring surface

- Pass: `Button::new(label)` plus `variant(...)` covers the documented `default`, `outline`, `secondary`, `ghost`, `destructive`, and `link` recipe surface.
- Pass: `size(...)` covers the documented `default`, `xs`, `sm`, `lg`, `icon`, `icon-xs`, `icon-sm`, and `icon-lg` options.
- Pass: `leading_child(...)` / `trailing_child(...)` provide the ergonomic single-node composition lane for the upstream `data-icon="inline-start|inline-end"` path, while `leading_children(...)` / `trailing_children(...)` remain available for multi-node landed content.
- Pass: `child(...)` complements `children(...)` as the singular full-row override for explicit custom button content.
- Pass: `ButtonRender::Link` is the shared Fret mapping for the Base UI docs' `As Link` section and the Radix docs' `As Child` link example; semantic link rendering stays button-owned instead of widening the public surface with a generic root `asChild` / `compose()` API.
- Pass: no extra generic root `asChild` / composable children API is currently warranted; `leading_child(...)` / `trailing_child(...)` cover the documented inline icon/spinner lane, while `child(...)` / `children(...)` remain the explicit full-row override for intentional custom content.

### Layout & default-style ownership

- Pass: recipe-owned defaults cover the intrinsic button chrome: height, horizontal/vertical padding, typography, `rounded-md`, variant colors, focus ring, disabled styling, and the current `new-york-v4` chrome baseline.
- Pass: logical inline child slots now compact only the occupied inline side and reverse correctly under RTL, matching the intent of upstream `data-icon="inline-start|inline-end"` examples more closely than the old symmetric-compact fallback.
- Pass: caller-owned refinements stay explicit for page/container negotiation (`w-full`, `min-w-0`, `flex-1`, wrapping, max width) and for one-off call-site polish such as the `rounded-full` examples.
- Note: the upstream `Cursor` section is Tailwind CSS-specific; in Fret, hover cursor policy belongs to runtime / pressable behavior, not the `Button` recipe defaults.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream docs path first: `Demo`, `Usage`, `Cursor`, `Size`, `Default`, `Outline`, `Secondary`, `Ghost`, `Destructive`, `Link`, `Icon`, `With Icon`, `Rounded`, `Spinner`, `Button Group`, `As Link / As Child (Semantic)`, and `RTL`, followed by `API Reference`.
- Pass: the shared semantic-link section is represented explicitly as `As Link / As Child (Semantic)` so authors can map both docs surfaces onto the same Fret lane without implying a generic root `asChild` API.
- Pass: `Children (Fret)` and `Variants Overview (Fret)` remain after `API Reference` so the upstream docs flow stays intact while Fret-specific composition guidance remains copyable.
- Pass: each Button docs section now exposes a page-scoped stable `test_id` prefix (`ui-gallery-button-*`), which lets diagnostics gate the real page structure instead of only snippet-local nodes.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- `cargo test -p fret-ui-shadcn --lib button_single_child_helpers_append_without_replacing_existing_content`
- `cargo test -p fret-ui-shadcn --lib button_inline_slot`
- `cargo test -p fret-ui-gallery --lib gallery_button_core_examples_keep_upstream_aligned_targets_present`
- `cargo test -p fret-ui-gallery --lib gallery_button_notes_keep_stable_height_while_scrolling_into_view`
- Existing shadcn-web gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`button-demo`)
- Existing diag scripts: `tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-with-icon-non-overlap.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-variants-width-zinc-dark.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-loading-screenshots-zinc-dark.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-loading-screenshots-zinc-light.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-rtl-row-screenshots.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-docs-screenshots.json`
