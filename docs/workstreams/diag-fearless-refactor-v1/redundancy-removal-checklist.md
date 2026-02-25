title: Diagnostics Fearless Refactor v1 (Redundancy Removal Checklist)
status: draft
date: 2026-02-24
scope: diagnostics, tooling, runtime, debt-removal
---

# Redundancy removal checklist (risk-tiered)

This checklist is the "remove the baggage" companion to the schema2-first (Plan 1) migration.

Principles:

- Prefer removing **duplicated logic** first (no behavior change).
- Then remove **deprecated compatibility outputs** (keep reading/accepting them longer than we write them).
- Remove **hard contracts / default changes** only after clear exit criteria and at least one regression gate exists.

Regression gates (recommended before removing any medium/high-risk item):

- `cargo check -p fret-ui-gallery`
- `cargo test -p fret-diag` (at least the "sidecars-only" and pack/repro AI-only tests)

## Low risk (do now; no behavior change)

- [x] Deduplicate "latest bundle" resolution helpers across the crate (not just `commands/*`).
  - Target: `diag_perf.rs`, `post_run_checks.rs`, `paths.rs` call sites.
  - Goal: one shared helper for "latest bundle dir" and one for "latest bundle artifact".
- [x] Deduplicate "bundle artifact hint -> choose bundle.schema2.json or bundle.json or sidecars-only" logic everywhere.
  - Prefer routing through `crates/fret-diag/src/commands/ai_packet.rs` (`ensure_ai_packet_dir_best_effort`).
  - Status: `ensure_ai_packet_dir_best_effort` now uses `resolve_bundle_artifact_path_no_materialize` (2026-02-24).
  - Status: `pack_zip` and `doctor` now use shared helpers for schema2/raw/bundle artifact presence checks (2026-02-24).
  - Status: no remaining manual schema2/raw presence branching found outside `paths.rs` (2026-02-25).
- [x] Reduce crate-root "prelude" style imports for stats gates.
  - Status: `diag_stats` / `post_run_checks` now call `stats::check_bundle_for_*`; crate root no longer imports `check_bundle_for_*` from `stats` (2026-02-25).
  - Status: moved `check_bundle_for_idle_no_paint_min` into `stats::stale` (2026-02-25).
- [x] Extract script gate policy helpers from crate root.
  - Status: `crates/fret-diag/src/diag_policy.rs` owns UI-gallery script classification helpers (2026-02-25).
- [x] Extract pixels-changed gate from crate root.
  - Status: `crates/fret-diag/src/stats/pixels_changed.rs` owns the screenshots-driven pixels-changed gate (2026-02-25).
- [ ] Delete unused helpers and dead code blocks as they appear during refactors.
  - Rule of thumb: if it has no call sites and no tests rely on it, remove it.

## Medium risk (after a deprecation window; keep reads longer than writes)

### Tooling output aliases

- [ ] Stop writing legacy JSON alias keys in tooling outputs once in-tree consumers have migrated.
  - Example: outputs that currently write both `bundle_artifact` (canonical) and `bundle_json` (legacy alias).
  - Plan:
    1. Keep reading both.
    2. Stop writing the legacy alias.
    3. Later: remove legacy alias reads.

### CLI flag aliases

- [ ] Remove redundant flag aliases after at least one milestone cycle.
  - Example: `--schema2-only` alias for `--pack-schema2-only` (if we keep it, document it as canonical instead).

### Sidecar placement conventions

- [ ] Tighten the `_root/` vs top-level sidecar placement policy once pack/repro flows are stable.
  - Goal: fewer candidate-path scans and fewer "where did this file go" edge cases.

## High risk (requires explicit exit criteria + migration plan)

### Raw `bundle.json` no longer required for common flows

- [ ] Decide whether scripted runs can default to **not** writing raw `bundle.json`.
  - Candidate knob (draft): `FRET_DIAG_BUNDLE_WRITE_RAW=0`.
  - Preconditions:
    - Sidecars + `bundle.schema2.json` exist reliably for launched runs.
    - AI packet / AI-only zips remain sufficient for first-pass triage.
    - Offline viewer-friendly share path is documented and stable.

### Remove schema v1 support paths

- [ ] Define a "read support" policy for schema v1 bundles.
  - Tooling read support should be retained longer than runtime write support.
  - Only remove readers once we have:
    - a documented migration recipe,
    - a `diag doctor` plan/fix story,
    - and at least one regression fixture proving expected failures are friendly.

## "What to delete next?" decision guide

Pick the highest-impact item that:

1. is low-risk, or
2. has a clear deprecation window and exit criteria,
3. can be locked by a small regression test.

Avoid:

- changing runtime defaults without a migration plan,
- deleting readers before writers (compat-first rule),
- removing "rare but needed" deep-debug paths before they are documented as "advanced / optional".
