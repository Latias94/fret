# Example Suite (Fearless Refactor v1) — Milestones

This file defines milestones for the workstream in `design.md`.

## Milestone 0 — Define the example product surface (catalog + ladder)

Outcome:

- A single “easy → hard” ladder exists and is referenced by onboarding docs.
- A v1 example catalog is defined (what we maintain as user-facing vs maintainer-only).

Deliverables:

- `design.md` finalized with:
  - a ladder table,
  - an example catalog table,
  - quality bars (gates + `test_id` stability).

Exit criteria:

- A new user can answer “what do I run next?” without reading the entire repo.

## Milestone 1 — Cookbook crate (fast, focused examples)

Outcome:

- A small, user-facing cookbook exists as Cargo `examples/`, with minimal dependencies.
- The ladder examples compile fast and are easy to read (one file ≈ one lesson).

Deliverables:

- A new cookbook crate (name TBD) with 8–12 runnable examples.
- Each example has stable `test_id`s and at least one gate (diag script or small test).

Exit criteria:

- “Learn by copy/paste” works: each example is small, runnable, and teaches one core idea.

## Milestone 2 — Registry consolidation (one source of truth)

Outcome:

- `fretboard` becomes the canonical way to discover and run examples/demos.
- Demo lists do not drift across native/web/tooling.

Deliverables:

- `fretboard-dev list ...` and `--choose` reflect the catalog (official first, maintainer hidden by default).
- Reduced duplication between:
  - native demo bins,
  - cookbook `examples/`,
  - web demo selection.
- A documented ownership split:
  - `docs/examples/README.md` is the single examples index,
  - runnable examples stay owned by cookbook/gallery/app crates rather than a root package.

Exit criteria:

- Adding a new official example requires updating a single “catalog surface” (or is auto-discovered).
- Discoverability does not depend on turning the workspace root into a single-package `cargo run --example` story.

## Milestone 3 — High ceiling tracks (interop + renderer labs)

Outcome:

- Interop and renderer effects are showcased as “Labs” without polluting the onboarding path.
- Custom effects (CustomV1/V2/V3 + pass semantics) are teachable and capability-gated.
- A small set of app-scale reference apps is scoped (may be docs-only in v1).

Deliverables:

- A curated set of interop examples (viewport, external textures, gizmo).
- A curated set of renderer labs with explicit budgets/capabilities.
- A scoped plan for 2–3 reference apps (workbench/viz-studio/shader-lab), including at least one
  smoke workflow gate per app.
- At least one scripted diag scenario per lab.

Exit criteria:

- Users can discover “cool” features safely, and maintainers have reproducible regressions.
