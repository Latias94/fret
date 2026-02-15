# Diagnostics perf profiling infra v1 — Milestones

## M0 — Additive CPU delta signals

Exit criteria:

- Bundles export UI thread CPU deltas (time + cycles).
- `diag stats` and `diag trace` surface those fields.
- Runbooks call out how to interpret CPU delta vs wall/phase time.

## M1 — Typical perf is review-grade

Exit criteria:

- `p50` / `p95` are first-class in `diag stats --json` and perf threshold reports.
- Gates can be configured explicitly for `max` vs `p95` (`--perf-threshold-agg` is documented).

## M2 — Perf key contract is explicit

Exit criteria:

- A perf key registry exists (names + units + scope + suggested aggregates).
- Contract tests prevent accidental renames / unit drift.
- Field inventory is generated or kept in sync from one source of truth.

## M3 — One-command attribution

Exit criteria:

- From `check.perf_thresholds.json`, we can jump to the worst bundle and identify the responsible
  phase with `diag stats` / `diag trace` without manual spelunking.
- At least one exemplar "schedule noise" case and one "real work" case are documented.
