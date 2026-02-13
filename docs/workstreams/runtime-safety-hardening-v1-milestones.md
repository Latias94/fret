# Runtime Safety Hardening v1 — Milestones

## M0 — Plan + link (same day)

Deliverables:

- Workstream docs exist and are linked:
  - `docs/workstreams/runtime-safety-hardening-v1.md`
  - `docs/workstreams/runtime-safety-hardening-v1-todo.md`
  - `docs/workstreams/runtime-safety-hardening-v1-milestones.md`
- Minimal gate set is documented and agreed:
  - `cargo nextest run -p fret-runtime`
  - `cargo nextest run -p fret-ui`
  - `cargo nextest run -p fret-app`
  - `python3 tools/check_layering.py`

Exit criteria:

- A branch/worktree plan exists for landing breaking API changes safely.

## M1 — ModelStore v2 (highest risk first)

Deliverables:

- ADR for `ModelStore v2` is written and accepted (or explicitly gated as proposed).
- Public leasing is removed/privatized; closure-based APIs are the only supported access path.
- `get_copied/get_cloned` are non-panicking and return explicit errors for `AlreadyLeased/TypeMismatch`.
- Regression tests exist for:
  - non-panicking lease violations,
  - unwind does not poison store state (when `panic=unwind`).

Exit criteria:

- `cargo nextest run -p fret-runtime` is green.
- First-party call sites compile without using the legacy leasing surface.

## M2 — Theme v2 (diagnostics + normalization)

Deliverables:

- Theme token contract ADR is written.
- Mechanism layer uses typed keys for core tokens.
- Missing tokens never panic by default; missing extension tokens generate diagnostics.

Exit criteria:

- `cargo nextest run -p fret-ui` is green.
- At least one targeted test covers missing-token behavior.

## M3 — Remove avoidable unsafe + globals hardening

Deliverables:

- Menu patch `unsafe` is removed from `crates/fret-runtime`.
- Global lease violations no longer panic by default (return `Result` errors; strict mode optional).

Exit criteria:

- `cargo nextest run -p fret-app` is green.

## M4 — Env flags caching (hot-path hygiene)

Deliverables:

- `FRET_*` debug flags are parsed once into a cached config struct.
- Hot-path code in `fret-ui` no longer reads environment variables directly.

Exit criteria:

- Perf-neutrality validated by existing perf baselines (where available).

