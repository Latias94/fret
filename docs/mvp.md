# MVP Plan (Overview)

This file is intentionally kept **short and actionable**.

- Full active plan (details, definitions, status notes): `docs/mvp/active-plan.md`
- Completed stage definitions: `docs/mvp-archive.md`
- Long-horizon priorities and refactors: `docs/roadmap.md`

## What’s Next

Maintain the next MVP items here as a small, high-signal queue. If a task changes a “hard-to-change” contract, update or add an ADR before broadening usage.

- MVP 49 (in progress): Make the declarative component authoring model (ADR 0028 + ADR 0039) the primary, end-to-end usable path (not just a state store): `IntoElement` + `Render`/`RenderOnce` + composition ergonomics, plus a clear `render_root(...)` contract (when it must be called, and what it guarantees).
- MVP 50 (in progress): Consolidate virtualization around composable, declarative row content (GPUI-style): migrate remaining list-like surfaces off fixed-schema runtime rows (`VirtualListRow { text/secondary/trailing... }`) and retire the legacy path where feasible.
- MVP 51 (in progress): Tighten the framework/components boundary by moving “standard surfaces” (popover/dialog/menu/tooltip/toast/command palette/menubar) fully into `fret-components-ui`, keeping `fret-ui` as runtime substrate + performance primitives. Legacy retained widgets stay behind `fret-ui`’s `legacy-widgets` feature (`crates/fret-ui/src/legacy_widgets/*`) until removal.
- MVP 55 (next): Expand style patch → layout bridging so Tailwind-like recipes can drive declarative layout without widget-local magic numbers: map additional sizing/flex/overflow knobs into declarative `LayoutStyle` (beyond the current minimal subset).
- MVP 58 (next): Implement Tailwind layout primitives at the runtime vocabulary level (margin, position/inset, grid, aspect-ratio) per ADR 0062, so shadcn-style layouts (badges, input icons, simple grids) are expressible without bespoke components.
- MVP 56 (in progress): Add missing shadcn “polish primitives” as reusable contracts (not per-widget hacks): shadow/elevation baseline is implemented (ADR 0060); focus ring baseline is implemented (ADR 0061, including a minimal focus-visible heuristic); richer scroll ergonomics is partially implemented (vertical scrollbar thumb drag + track paging); remaining: horizontal/bidirectional scroll and scroll-to-child.
- MVP 57 (done): Declarative icon helper (glyph-based) so shadcn-style list/menu rows can compose leading/trailing icons without falling back to retained widgets.

## Current Status (Snapshot)

See `docs/mvp/active-plan.md` for the authoritative, expanded status list and per-MVP details.
