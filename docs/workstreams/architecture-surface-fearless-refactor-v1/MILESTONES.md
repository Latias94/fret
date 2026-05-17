# Architecture Surface Fearless Refactor v1 — Milestones

Status: Active
Last updated: 2026-05-17

## M0 — Scope And Evidence Freeze

Exit criteria:

- The architecture findings are recorded.
- The target state says explicitly that old compatibility paths may be deleted.
- The first proof target and gate set are listed.

Primary evidence:

- `docs/workstreams/architecture-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/architecture-surface-fearless-refactor-v1/TODO.md`
- `docs/workstreams/architecture-surface-fearless-refactor-v1/EVIDENCE_AND_GATES.md`

## M1 — Minimal App Authoring Profile

Exit criteria:

- `fret --no-default-features` and `fret --no-default-features --features app` have honest,
  documented dependency behavior.
- Backend crates and renderer crates are absent from backend-free profiles or intentionally required
  by a documented feature.
- The consumption profile gate prevents regression.

Primary gates:

- `cargo tree -p fret --no-default-features -e normal --depth 4`
- `cargo tree -p fret --no-default-features --features app -e normal --depth 4`
- `cargo check -p fret --no-default-features`
- `cargo check -p fret --no-default-features --features app`
- `python tools/check_consumption_profiles.py`

## M2 — Bootstrap Plan vs Launch Adapter

Exit criteria:

- `fret-bootstrap --no-default-features` can be used without implicit launch/render dependencies, or
  the crate is renamed/repositioned so the dependency behavior is explicit.
- Concrete runner/render adapters are feature-gated or moved to the owning launch layer.
- First-party templates and demos use the target path.

Primary gates:

- `cargo tree -p fret-bootstrap --no-default-features -e normal --depth 4`
- targeted `cargo check` / `cargo nextest` for changed packages.

## M3 — Public Facade Narrowing

Exit criteria:

- `fret::app::prelude::*` has a small approved budget.
- Advanced interop, raw model escape hatches, and lower-level component authoring helpers require
  explicit imports.
- Public surface tests lock the distinction.

Primary gates:

- `cargo nextest run -p fret`
- targeted tests under `ecosystem/fret/tests` or source-level public-surface gates.

## M4 — Ecosystem Taxonomy Closure

Exit criteria:

- One representative primitive family proves the headless/primitives/kit/recipe split.
- Compatibility re-exports are deleted or explicitly quarantined.
- ADR 0154 alignment is refreshed with code and test anchors.

Primary gates:

- `python tools/check_layering.py`
- targeted package tests for the representative primitive family.

## M5 — Shared Menu/Select Policy

Exit criteria:

- At least one repeated menu/select behavior is owned by a shared module and consumed by multiple
  recipe surfaces, or the extraction is rejected with evidence.
- Recipe files keep taxonomy/style ownership rather than duplicating shared behavior policy.

Primary gates:

- targeted `cargo nextest run -p fret-ui-shadcn <menu-or-select-filter>`
- targeted unit tests in the owner module.

## M6 — Renderer Facade Decision

Exit criteria:

- `fret-render` is either collapsed as a shallow facade or deepened as the renderer interface.
- The decision is reflected in docs, Cargo features, and at least one compile/test gate.

Primary gates:

- targeted `cargo check` for the chosen renderer profile.
- renderer-specific tests if code moves.

## M7 — Closeout

Exit criteria:

- Final gates are recorded in `EVIDENCE_AND_GATES.md`.
- Remaining tasks are completed, explicitly deferred, or split into narrower workstreams.
- `WORKSTREAM.json` status reflects the lane state.
