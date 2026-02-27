# Framework Modularity (Fearless Refactor v1) — Milestones

This file defines milestones for the workstream in `design.md`.

## Milestone 0 — Define the product surface (profiles + entry points)

Outcome:

- Clear, stable consumption profiles A–D (contracts-only, UI substrate, manual assembly, batteries).
- Clear mapping from “what you want to do” → “which crate/features to depend on”.

Deliverables:

- `design.md` finalized with profile table.
- A short docs entry linking to this workstream from the canonical docs index (location TBD).

Exit criteria:

- A new user can answer “which crate do I add?” in under 60 seconds.

## Milestone 1 — Enforce modularity (profile build gates)

Outcome:

- Modularity becomes a *regression-tested property*, not a one-time doc statement.

Deliverables:

- Build gates for profiles A–D (see `todo.md` M1).
- Keep `tools/check_layering.py` required and green.

Exit criteria:

- Any accidental dependency growth (e.g. backends pulled into portable crates) fails fast.

## Milestone 2 — Contain glue complexity (launcher split)

Outcome:

- Launcher glue is decomposed so users can pick a platform without paying for others.
- Maintainers can change platform-specific wiring without destabilizing the kernel.

Deliverables:

- A documented split plan for `fret-launch`.
- Platform-specific crates/modules behind features with minimal “facade” re-exports.

Exit criteria:

- Desktop-only build does not compile web-specific codepaths (and vice versa).
- The “manual assembly” profile remains viable and documented.

## Milestone 3 — Ongoing hygiene (public surface governance)

Outcome:

- The repo can keep growing in ecosystem/components without destabilizing kernel portability.

Deliverables:

- A small “public entry crates” list and promotion rules for new public APIs.
- A deprecation/migration approach for entry points and feature bundles.

Exit criteria:

- The set of supported entry points is small, explicit, and testable.

