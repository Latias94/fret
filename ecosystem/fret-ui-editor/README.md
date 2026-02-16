# `fret-ui-editor`

Editor-grade primitives, controls, and composites for Fret (ecosystem layer).

This crate is intentionally **policy-heavy** compared to `crates/fret-ui`:

- `crates/fret-ui`: mechanism-only substrate (tree, routing, hit-test, focus, capture, overlays).
- `ecosystem/fret-ui-editor`: editor interaction + composition policy (scrubbing, density, property rows).
- Optional style kits (e.g. shadcn) should live in separate adapter crates.

## Dependency boundaries

Allowed (expected):

- `crates/fret-ui`
- `ecosystem/fret-ui-kit`
- `ecosystem/fret-ui-headless`
- Optional: `ecosystem/fret-selector`, `ecosystem/fret-query`, `ecosystem/fret-undo` (feature-gated)

Forbidden (by design intent):

- `ecosystem/fret-ui-shadcn` (style/recipes should be an adapter, not a hard dependency)
- Any runner/platform/render crates (`fret-runner-*`, `fret-platform-*`, `fret-render`, `wgpu`, `winit`)
- Domain ecosystem crates (node/plot/chart/etc.) inside core controls

## Features

- `state-selector`: selector-derived state helpers (depends on `fret-selector/ui`)
- `state-query`: query status/error rendering helpers (kept UI-light)
- `state-undo`: undo/coalescing helpers (policy remains app-owned by default)
- `state`: umbrella for the above
- `imui`: optional immediate-mode authoring facade (thin adapters only)

## v1 workstream

See:

- `docs/workstreams/ui-editor-v1.md`
- `docs/workstreams/ui-editor-v1-todo.md`

