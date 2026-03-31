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
  maintainer conveniences should stay as commented advanced aliases, and heavier
  editor/design-system/policy ecosystems should stay on direct owning crates instead of root
  feature proxies.
- release-plz scope is allowed to be broader than the taught entry set, but only for actual support
  closure crates; app-only and deferred ecosystem surfaces should stay out.

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
- `fret` now keeps only app-facing and maintainer-oriented lanes on its root feature matrix:
  `devloop` / `tracing` stay as advanced aliases, while Material 3, AI UI, docking, and editor
  policy now stay on owning crates instead of `fret` root features.

## Wave 3 freeze

- Frozen user-facing entry set:
  - `fret`
  - `fret-framework`
  - `fret-bootstrap`
  - `fret-ui-kit`
  - `fret-ui-shadcn`
  - `fret-selector`
  - `fret-query`
- Frozen publish closure:
  - `release-plz.toml` is the source of truth for the crates.io whitelist.
  - The current closure intentionally includes `fret-chart` and `delinea` because the published
    `fret-ui-shadcn/chart` lane remains supported, but they are not part of the primary teaching
    surface.
  - `python3 tools/release_closure_check.py --config release-plz.toml` now reports
    `internal dependency issues: 0`.
  - `docs/release/v0.1.0-publish-order.txt` is synced to the current 49-crate publish order.

## Documents

- Design: [docs/workstreams/release-surface-fearless-refactor-v1/DESIGN.md](./DESIGN.md)
- TODO: [docs/workstreams/release-surface-fearless-refactor-v1/TODO.md](./TODO.md)
- Milestones: [docs/workstreams/release-surface-fearless-refactor-v1/MILESTONES.md](./MILESTONES.md)
