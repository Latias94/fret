# Bootstrap Known Startup Failure Taxonomy v1 — Milestones

Status: Closed
Last updated: 2026-04-09

## M0 — Baseline and scope freeze

Exit criteria:

- The broader bootstrap taxonomy problem is explicitly separated from the closed icon-reporting
  lane.
- The current returned-error and panic-hook surfaces are audited.
- The lane's non-goals are explicit.

Primary evidence:

- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/DESIGN.md`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/TODO.md`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`

Current status:

- Opened and closed on 2026-04-09 as a narrow follow-on to the icon-reporting lane.
- M0 baseline freeze closed on 2026-04-09.

## M1 — Contract freeze

Exit criteria:

- The taxonomy home is explicit.
- The lifecycle non-goal is explicit.
- The `fret` facade bridge and root-surface budget are explicit.

Primary evidence:

- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/DESIGN.md`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/M1_CONTRACT_FREEZE_2026-04-09.md`

Current status:

- M1 contract freeze closed on 2026-04-09.

## M2 — Proof surface

Exit criteria:

- Bootstrap taxonomy types and mappings are real.
- `fret::Error` exposes one bridge method.
- Diagnostics logs one bootstrap-level field family.
- Tests lock the new bridge on both `fret-bootstrap` and `fret`.

Primary gates:

- `cargo nextest run -p fret-bootstrap`
- `cargo nextest run -p fret --lib`
- `cargo check -p fret-bootstrap --features diagnostics`

Current status:

- M2 proof surface closed on 2026-04-09.
- See `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/M2_PROOF_SURFACE_2026-04-09.md`.

## M3 — Docs and closeout

Exit criteria:

- The gate set is explicit.
- Roadmap/workstream indexes point at the follow-on.
- The lane closes explicitly.

Primary gates:

- `cargo nextest run -p fret-bootstrap`
- `cargo nextest run -p fret --lib`
- `cargo check -p fret-bootstrap --features diagnostics`
- `python3 tools/check_layering.py`
- `git diff --check`

Current status:

- M3 docs and closeout closed on 2026-04-09.
- The lane is now closed on
  `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
