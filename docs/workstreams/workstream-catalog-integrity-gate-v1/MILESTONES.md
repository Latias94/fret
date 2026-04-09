# Workstream Catalog Integrity Gate v1 — Milestones

Status: Closed
Last updated: 2026-04-10

## M0 — Baseline audit

Exit criteria:

- The current drift modes are explicit.
- The lane distinguishes top-level directory coverage from standalone file coverage.
- The lane's non-goals are explicit.

Primary evidence:

- `docs/workstreams/workstream-catalog-integrity-gate-v1/DESIGN.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/TODO.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/BASELINE_AUDIT_2026-04-10.md`
- `docs/workstreams/README.md`
- `docs/workstreams/standalone/README.md`

Current status:

- M0 baseline audit closed on 2026-04-10.

## M1 — Gate contract freeze

Exit criteria:

- The checker-owned sections are explicit.
- The checker-owned count lines are explicit.
- The adoption entrypoints are explicit.
- Full historical order normalization is explicitly deferred.

Primary evidence:

- `docs/workstreams/workstream-catalog-integrity-gate-v1/DESIGN.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/BASELINE_AUDIT_2026-04-10.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
- `tools/check_workstream_catalog.py`

Current status:

- M1 gate contract freeze closed on 2026-04-10.

## M2 — Proof surface

Exit criteria:

- The new checker passes.
- Common gate entrypoints exercise the checker successfully.
- The currently detected catalog drift is fixed.

Primary gates:

- `python3 tools/check_workstream_catalog.py`
- `python3 tools/gates_fast.py --skip-fmt --skip-nextest`
- `python3 -m py_compile tools/check_workstream_catalog.py tools/gates_fast.py tools/pre_release.py`
- `git diff --check`

Current status:

- M2 proof surface closed on 2026-04-10.
- See `docs/workstreams/workstream-catalog-integrity-gate-v1/M2_PROOF_SURFACE_2026-04-10.md`.

## M3 — Closeout

Exit criteria:

- The lane is discoverable from the workstream catalog.
- Follow-on policy is explicit.
- The lane closes without widening into generic docs automation.

Primary evidence:

- `docs/workstreams/workstream-catalog-integrity-gate-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/README.md`

Current status:

- M3 closeout closed on 2026-04-10.
