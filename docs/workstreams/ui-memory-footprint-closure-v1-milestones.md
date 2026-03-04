# UI Memory Footprint Closure (v1) — Milestones

## M0 — Baseline evidence (done / in progress)

- A stable “steady state” script exists for a representative demo (todo).
- Bundles include app-side GPU stats (Metal current allocated size on macOS).

## M1 — Structured CPU footprint attribution

- `vmmap -summary` is parsed into structured fields in `resource.footprint.json`.
- Top contributors (region types + malloc zone stats where available) are visible without manual parsing.

## M2 — Minimal scenario matrix

- `empty-idle`, `text-heavy`, and `image-heavy` scripts exist and run reliably in `diag repro`.
- Each script has a documented expected shape (which counters should be near-zero vs non-zero).

## M3 — First bounded optimization with a gate

- At least one additional optimization is landed that reduces either CPU footprint or GPU allocated size
  measurably in one scenario.
- A gate is added to prevent regressions, with an explicit drift policy.

