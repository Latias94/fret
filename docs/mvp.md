# MVP Plan (Overview)

This file is intentionally kept **short and actionable**.

- Full active plan (details, definitions, status notes): `docs/mvp/active-plan.md`
- Completed stage definitions: `docs/mvp-archive.md`
- Long-horizon priorities and refactors: `docs/roadmap.md`

## What’s Next

Maintain the next MVP items here as a small, high-signal queue. If a task changes a “hard-to-change” contract, update or add an ADR before broadening usage.

- MVP 49: Land the declarative component authoring model (ADR 0028 + ADR 0039) as an end-to-end, usable path (not just a state store): `IntoElement` + `Render`/`RenderOnce` + composition ergonomics for building component trees.
- MVP 50: Introduce a composable, component-friendly virtualized list contract (GPUI-style) so list rows are not constrained to `VirtualListRow { text/secondary/trailing... }`; migrate at least one real component (Command palette or Tree) to prove the model.
- MVP 51: Consolidate the “standard surfaces” fully into `fret-components-ui` (remove remaining UI-kit-shaped runtime widgets like `TreeView`/runtime `Toolbar` where feasible) once MVP 49/50 exist. These are now gated behind `fret-ui`'s `legacy-widgets` feature (`crates/fret-ui/src/legacy_widgets/*`).

## Current Status (Snapshot)

See `docs/mvp/active-plan.md` for the authoritative, expanded status list and per-MVP details.
