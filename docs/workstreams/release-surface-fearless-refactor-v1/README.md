# Release Surface Fearless Refactor v1

Goal: make the publishable Fret surface boring, bounded, and mechanically releasable before the
first public release.

This workstream focuses on:

- the smallest user-facing crate set we actually want to teach,
- the crates.io publish closure behind those entry points,
- removing `publish = false` dependencies from publishable crates,
- shrinking obviously misplaced facade surface area before release.

Non-goals:

- redesigning runtime contracts or component behavior,
- making every in-tree ecosystem crate release-ready in one pass,
- preserving pre-release facade shortcuts just because they already exist.

## Current stance

- Default app authors should still learn `fret`.
- Advanced/manual assembly should still center on `fret-framework` + `fret-bootstrap`.
- Component/policy authors should still center on `fret-ui-kit` and `fret-ui-shadcn`.
- Tooling stays tooling: `fretboard` and `fret-diag` are not part of the minimal library release.
- `fret-bootstrap` is feature-first: recommended bootstrap integrations stay on that crate unless
  they become a genuinely separate authoring surface that users should learn independently.
- `fret` root features should stay limited to lanes that form a real app-facing teaching surface;
  maintainer conveniences and niche design-system/domain crates should stay on direct owning
  crates, even if `fret` keeps a few commented compatibility aliases for discoverability.

## First landed slice

- `fret` no longer carries the editor-only `workspace_shell` module on its public root surface.
- `fret` no longer directly depends on `fret-workspace`.
- `fret-router-ui` is promoted from `publish = false` to a publishable thin adoption crate so the
  optional `fret/router` lane no longer blocks `fret` release packaging.

## Wave 2 current slice

- `fret-ui-shadcn` no longer pulls `fret-chart` into its default closure.
- chart recipes now sit behind an explicit `chart` feature, with `fret-ui-gallery` opting in only
  on its chart-specific gallery lanes.
- `fret-bootstrap` is now explicitly feature-first: command palette shadcn defaults and
  diagnostics WS stay on explicit bootstrap features instead of multiplying bridge crates.

## Documents

- Design: [docs/workstreams/release-surface-fearless-refactor-v1/DESIGN.md](./DESIGN.md)
- TODO: [docs/workstreams/release-surface-fearless-refactor-v1/TODO.md](./TODO.md)
- Milestones: [docs/workstreams/release-surface-fearless-refactor-v1/MILESTONES.md](./MILESTONES.md)
