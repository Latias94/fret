# Bottom-up Fearless Refactor v1 — Milestones

## M0 — Baseline + tracking (1–2 days)

Deliverables:

- Workstream doc + TODO tracker exist and are linked:
  - `docs/workstreams/bottom-up-fearless-refactor-v1.md`
  - `docs/workstreams/bottom-up-fearless-refactor-v1-todo.md`
  - `docs/workstreams/bottom-up-fearless-refactor-v1-milestones.md`
- A crate-audit template is chosen (or documented) and used for the first audited crate.
- A minimal “inner loop” validation checklist is agreed on for refactor PRs:
  - `cargo fmt` (package-scoped when needed),
  - `cargo nextest run -p <crate>`.

Exit criteria:

- At least one small refactor is landed with a dedicated gate (unit test or diag script).

## M1 — Foundation crate audits (time-boxed)

Deliverables:

- Audit notes exist (even if incomplete) for:
  - `crates/fret-core`
  - `crates/fret-runtime`
- Each audit produces 3–10 actionable TODO items:
  - invariants to document,
  - missing tests,
  - cleanup candidates (API hygiene, module layout, error handling).

Exit criteria:

- `cargo nextest run -p fret-core` (or `cargo test -p fret-core` if needed) passes.
- `cargo nextest run -p fret-runtime` passes.

## M2 — `fret-ui` structure and contract hygiene (time-boxed)

Deliverables:

- A module ownership map is recorded:
  - what belongs in `tree/`, `layout/`, `elements/`, `text/`, `semantics/`, `input/`.
- A staged refactor plan for `crates/fret-ui/src` exists:
  - mechanical grouping first (paths/modules),
  - semantic cleanup second (API boundaries, naming, invariants).
- Add or refresh a small set of “contract” tests where regressions are costly:
  - outside-press dismissal,
  - focus routing edge cases,
  - identity stability / callsite mapping.

Exit criteria:

- The refactor plan is executable in small PR-sized steps (no “one mega move”).

## M3 — Ecosystem policy audits (time-boxed)

Deliverables:

- `ecosystem/fret-ui-kit` audit note and gate suggestions.
- `ecosystem/fret-ui-shadcn` audit note and gate suggestions:
  - reduce test helper duplication via `tests/support/`,
  - convert large repetitive suites to fixture-driven harnesses when appropriate.

Exit criteria:

- `cargo nextest run -p fret-ui-kit` passes.
- `cargo nextest run -p fret-ui-shadcn` passes.

## M4 — Compile/test speed improvements (iterative)

Deliverables:

- Identify and document the top sources of slow iteration:
  - repeated test helpers,
  - large test modules,
  - feature bloat or accidental dependencies.
- Add at least one “fast gate” script or documented command set for local iteration.

Exit criteria:

- A measurable improvement is recorded (even if small): fewer rebuilt test binaries or shorter
  `nextest` runtime for the targeted package.

