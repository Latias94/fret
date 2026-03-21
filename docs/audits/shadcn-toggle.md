# shadcn/ui v4 Audit - Toggle

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Toggle` against the upstream shadcn/ui v4 base docs,
base examples, and the existing toggle web gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/toggle.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/toggle.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/toggle-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-outline.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-text.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-sizes.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-disabled.tsx`, `repo-ref/ui/apps/v4/examples/base/toggle-rtl.tsx`
- Existing chrome gates: `goldens/shadcn-web/v4/new-york-v4/toggle-demo.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-demo.focus.json`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/toggle.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/toggle.rs`

## Audit checklist

### Authoring surface

- Pass: `Toggle::uncontrolled(false)` plus `variant(...)`, `size(...)`, `disabled(...)`, and `a11y_label(...)` covers the documented toggle surface.
- Pass: `children([...])` is the source-aligned Fret equivalent of upstream child content, while `label(...)` remains the ergonomic shortcut for common icon-plus-text cases.
- Pass: the gallery now includes a focused `Children (Fret)` follow-up that teaches `Toggle::uncontrolled(...).children([...])` for caller-owned or reusable landed content without displacing the default helper-based usage lane.
- Pass: `Toggle::new(model)` and `Toggle::from_pressed(...)` continue to cover controlled and action-first authoring without widening the public surface further.
- Pass: no extra generic `asChild` / `compose()` API is needed here because `children([...])` already covers the composable content story.

### Layout & default-style ownership

- Pass: toggle chrome, size presets, horizontal padding, and pressed-state colors remain recipe-owned because the upstream toggle source defines those defaults on the component itself.
- Pass: surrounding toolbar layout, wrapping behavior, and page/grid negotiation remain caller-owned.
- Pass: pressed, hover, and focus-visible outcomes continue to be covered by the existing toggle chrome gates; this pass does not reveal a mechanism-layer gap.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream base Toggle docs path first: `Demo`, `Usage`, `Outline`, `With Text`, `Size`, `Disabled`, and `RTL`.
- Pass: `Children (Fret)` now follows the upstream path as an explicit Fret-only authoring note, so callers can see the landed-element equivalent of JSX children without confusing it with the default copyable lane.
- Pass: `Label Association` remains a focused Fret follow-up after the upstream path because it documents the Fret-specific `control_id(...)` bridge.
- Pass: `API Reference` remains the concise ownership summary after the Fret-specific follow-ups.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_toggle --failure-output final`
- `cargo test -p fret-ui-shadcn --lib toggle_children_accept_prebuilt_landed_content`
- `env CARGO_TARGET_DIR=target-codex-fretboard-toggle cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/toggle/ui-gallery-toggle-docs-smoke.json --dir target/fret-diag-toggle-audit --session-auto --timeout-ms 240000 --launch -- env CARGO_TARGET_DIR=target-codex-ui-gallery-toggle cargo run -p fret-ui-gallery`
- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- Existing chrome + focus gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`toggle-demo`, `toggle-demo.focus`)
