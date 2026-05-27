# shadcn/ui v4 Audit — Skeleton

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Skeleton` against the current upstream shadcn/ui v4 docs,
the new-york-v4 recipe and examples, the base/radix example expansion, the current gallery/docs
surface, and the absence of any dedicated headless `Skeleton` primitive in the Base UI / Radix
primitives reference axes.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/skeleton.mdx`
- Component implementations: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/skeleton.tsx`,
  `repo-ref/ui/apps/v4/registry/bases/base/ui/skeleton.tsx`,
  `repo-ref/ui/apps/v4/registry/bases/radix/ui/skeleton.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/skeleton-demo.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/skeleton-card.tsx`,
  `repo-ref/ui/apps/v4/registry/bases/base/examples/skeleton-example.tsx`,
  `repo-ref/ui/apps/v4/registry/bases/radix/examples/skeleton-example.tsx`
- Web goldens: `goldens/shadcn-web/v4/new-york-v4/skeleton-demo.json`,
  `goldens/shadcn-web/v4/new-york-v4/skeleton-card.json`
- Headless references: no dedicated `Skeleton` primitive exists under `repo-ref/primitives` or
  `repo-ref/base-ui`; those axes therefore confirm that this family is recipe/docs work, not a
  missing mechanism contract.

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/skeleton.rs`
- Layout and marker gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/skeleton.rs`,
  `ecosystem/fret-ui-shadcn/tests/web_vs_fret_misc_targeted.rs`
- Motion gate: `ecosystem/fret-ui-shadcn/tests/reduced_motion_continuous_frames.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/skeleton.rs`
- Gallery tests: `apps/fret-ui-gallery/tests/skeleton_docs_surface.rs`,
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
- Diagnostics: `tools/diag-scripts/ui-gallery/skeleton/*.json`

## Audit checklist

### Authoring surface

- Pass: `Skeleton::new()` covers the upstream leaf primitive path where callers set size and shape explicitly.
- Pass: `Skeleton::block()` remains a focused Fret convenience (`w-full h-4`) for common loading rows without changing the upstream default path.
- Pass: `Skeleton` is a visual leaf primitive, so Fret intentionally does not add a generic `compose()` builder here.
- Pass: No composable children API is needed here; the upstream shadcn/base/radix surfaces all expose `Skeleton` as a leaf `div`/placeholder boundary rather than a compound parts family.

### Visual defaults and ownership

- Pass: Default chrome uses `accent` background with `rounded-md` corners.
- Pass: Pulse animation is enabled by default, matching the upstream `animate-pulse` outcome.
- Pass: Explicit width, height, aspect ratio, and fully rounded avatar shapes remain caller-owned rather than recipe defaults.

### Mechanism boundary

- Pass: `repo-ref/primitives` and `repo-ref/base-ui` do not define a dedicated `Skeleton`
  primitive, so there is no missing mechanism/headless contract to port into `fret-ui` or
  `fret-ui-kit`.
- Pass: Existing `web_vs_fret_layout::skeleton_*` and reduced-motion tests already cover the
  runtime/layout side; the remaining work here is public-surface/docs alignment.

### Gallery / docs parity

- Pass: The gallery mirrors the current upstream Skeleton docs path through `Demo`, `Usage`, and
  `Card`, then adds base/radix example sections (`Avatar`, `Text`, `Form`, and `Table`) before the
  Fret-only `RTL`, `API Reference`, and `Notes` follow-ups.
- Pass: `API Reference` remains a compact Fret follow-up summarizing ownership because upstream
  treats Skeleton as a very small leaf primitive.
- Pass: `Notes` now record the source axes and explicitly document why no extra composable children
  API or mechanism-layer work is needed.

## Validation

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/skeleton_agent_packet_p0_v1.json | Out-Null`
- `Get-ChildItem -Path tools/diag-scripts/ui-gallery/skeleton -Filter *.json | ForEach-Object { python -m json.tool $_.FullName | Out-Null }`
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail skeleton`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail skeleton`
- `cargo nextest run -p fret-ui-shadcn --test reduced_motion_continuous_frames --status-level fail skeleton_respects_reduced_motion_and_does_not_request_frames`
- `cargo nextest run -p fret-ui-gallery --test skeleton_docs_surface --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail skeleton`
