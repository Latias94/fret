# shadcn/ui v4 Audit - Toggle

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Toggle` against the current main-worktree shadcn/ui v4
docs path, the `new-york-v4` recipe and examples, the base/radix registry examples, and the
existing toggle web, Gallery, diagnostics, and packet gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/toggle.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/toggle.tsx`
- New York examples: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-outline.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-with-text.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-sm.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-lg.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-disabled.tsx`
- Base/radix examples: `repo-ref/ui/apps/v4/registry/bases/base/ui/toggle.tsx`, `repo-ref/ui/apps/v4/registry/bases/base/examples/toggle-example.tsx`, `repo-ref/ui/apps/v4/registry/bases/radix/ui/toggle.tsx`, `repo-ref/ui/apps/v4/registry/bases/radix/examples/toggle-example.tsx`
- Existing goldens: `goldens/shadcn-web/v4/new-york-v4/toggle-demo.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-demo.focus.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-outline.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-with-text.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-sm.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-lg.json`, `goldens/shadcn-web/v4/new-york-v4/toggle-disabled.json`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/toggle.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/toggle.rs`
- Matrix packet: `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/toggle_agent_packet_p0_v1.json`

## Audit checklist

### Authoring surface

- Pass: `toggle_uncontrolled(cx, false, |cx| ..)` and `toggle(cx, model, |cx| ..)` are the current first-party builder-preserving lane for upstream-shaped child content.
- Pass: `Toggle::uncontrolled(false)` plus `variant(...)`, `size(...)`, `disabled(...)`, `a11y_label(...)`, and `children([...])` covers the documented toggle surface and the landed-content follow-up.
- Pass: `children([...])` is the source-aligned Fret equivalent of upstream child content when callers already own built elements, while `label(...)` remains the ergonomic shortcut for common icon-plus-text cases.
- Pass: the gallery now includes a focused `Children (Fret)` follow-up that teaches `Toggle::uncontrolled(...).children([...])` for caller-owned or reusable landed content without displacing the default helper-based usage lane.
- Pass: `Toggle::new(model)` and `Toggle::from_pressed(...)` continue to cover controlled and action-first authoring without widening the public surface further.
- Pass: no extra generic `asChild` / `compose()` API is needed here because `children([...])` already covers the composable content story.

### Layout & default-style ownership

- Pass: toggle chrome, size presets, horizontal padding, and pressed-state colors remain recipe-owned because the upstream toggle source defines those defaults on the component itself.
- Pass: surrounding toolbar layout, wrapping behavior, and page/grid negotiation remain caller-owned.
- Pass: pressed, hover, focus-visible, size geometry, and chrome outcomes continue to be covered by the existing toggle unit, web-golden, and chrome gates; this pass does not reveal a mechanism-layer gap.

### Gallery / docs parity

- Pass: the gallery now mirrors the current Toggle docs path first after collapsing the top `ComponentPreview` / `Default` duplicate into `Demo`: `Demo`, `Usage`, `Outline`, `With Text`, `Small`, `Large`, and `Disabled`.
- Pass: the current upstream `Small` and `Large` examples are separate sections instead of the older aggregate `Size` section.
- Pass: the gallery now states the layering decision explicitly: existing toggle semantics/chrome gates cover the Radix/Base UI behavior axis, so the remaining work here is docs/public-surface parity rather than a `fret-ui` mechanism fix.
- Pass: `Children (Fret)` now follows the upstream path as an explicit Fret-only authoring note, so callers can see the landed-element equivalent of JSX children without confusing it with the default copyable lane.
- Pass: the docs page now keeps the builder-preserving helper lane (`toggle_uncontrolled(cx, ...)` / `toggle(cx, ...)`) as the source-shaped composable-children path while leaving `Toggle::children([...])` as the landed-content follow-up.
- Pass: `Label Association` remains a focused Fret follow-up after the upstream path because it documents the Fret-specific `control_id(...)` bridge.
- Pass: the RTL snippet remains a Fret/base-radix follow-up and uses a translated visible label while staying on the same source-shaped helper lane.
- Pass: `API Reference` remains the concise ownership summary after the Fret-specific follow-ups.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/toggle_agent_packet_p0_v1.json | Out-Null`
- `Get-ChildItem -Path tools/diag-scripts/ui-gallery/toggle -Filter *.json | ForEach-Object { python -m json.tool $_.FullName | Out-Null }`
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail toggle`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_toggle --status-level fail toggle_`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail toggle`
- `cargo nextest run -p fret-ui-gallery --test toggle_docs_surface --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail toggle`
