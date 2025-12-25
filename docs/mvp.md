# MVP Plan (Overview)

This file is intentionally kept **short and actionable**.

- Full active plan (details, definitions, status notes): `docs/mvp/active-plan.md`
- Completed stage definitions: `docs/mvp-archive.md`
- Long-horizon priorities and refactors: `docs/roadmap.md`

## What’s Next

Maintain the next MVP items here as a small, high-signal queue. If a task changes a “hard-to-change” contract, update or add an ADR before broadening usage.

- MVP 49: Land the declarative component authoring model (ADR 0028 + ADR 0039) as an end-to-end, usable path (not just a state store): `IntoElement` + `Render`/`RenderOnce` + composition ergonomics for building component trees.
- MVP 50: Introduce a composable, component-friendly virtualized list contract (GPUI-style) so list rows are not constrained to `VirtualListRow { text/secondary/trailing... }`; migrate at least one real component (Command palette) to prove the model.
- MVP 52: Fix declarative composable sizing semantics by introducing a typed layout style vocabulary + a Taffy-backed `Flex` container (ADR 0057). This is a primary blocker for Tailwind/shadcn parity.
- MVP 53: Expand typography expressiveness (font weight + line height + tracking) and theme support so shadcn-style text recipes are possible without per-widget hacks.
- MVP 54: Extend shadcn semantic palette compatibility keys (ADR 0050) so components can rely on `primary/secondary/destructive/input/card/...` vocabulary via aliases.
- MVP 55: Bridge component recipes to declarative props (e.g. `StyleRefinement` → declarative `Container`/`Flex` props) to eliminate magic numbers in composable surfaces and keep Tailwind/shadcn semantics component-owned.
- MVP 51: Consolidate the “standard surfaces” fully into `fret-components-ui` (remove remaining UI-kit-shaped runtime widgets like `TreeView`/runtime `Toolbar` where feasible) once MVP 49/50 exist. These are now gated behind `fret-ui`'s `legacy-widgets` feature (`crates/fret-ui/src/legacy_widgets/*`).

## Current Status (Snapshot)

See `docs/mvp/active-plan.md` for the authoritative, expanded status list and per-MVP details.
