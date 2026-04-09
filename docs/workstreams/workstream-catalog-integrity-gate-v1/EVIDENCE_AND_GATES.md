# Workstream Catalog Integrity Gate v1 — Evidence and Gates

Status: Closed
Last updated: 2026-04-10

## Smallest current repro

Use this sequence before changing the workstream catalog README surfaces or the checker:

```bash
python3 tools/check_workstream_catalog.py
python3 tools/gates_fast.py --skip-fmt --skip-nextest
python3 -m py_compile tools/check_workstream_catalog.py tools/gates_fast.py tools/pre_release.py
git diff --check
```

What this proves now:

- the dedicated-directory catalog is complete and count-correct,
- the standalone file catalog is complete and count-correct,
- the checker is exercised through common maintainer entrypoints,
- and the docs patch is syntactically clean.

## Current evidence set

- `docs/workstreams/workstream-catalog-integrity-gate-v1/BASELINE_AUDIT_2026-04-10.md`
  freezes the concrete drift that motivated the lane:
  - missing directory entries,
  - and stale standalone bucket count.
- `docs/workstreams/workstream-catalog-integrity-gate-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
  freezes the checker boundary:
  - validate catalog sections, not every prose link,
  - validate tracked count lines,
  - and run through common gate entrypoints without turning full historical order normalization into
    this lane's scope.
- `docs/workstreams/workstream-catalog-integrity-gate-v1/M2_PROOF_SURFACE_2026-04-10.md`
  closes the proof on:
  - checker behavior,
  - gate integration,
  - repaired catalog coverage/counts,
  - and diff hygiene.
- `docs/workstreams/workstream-catalog-integrity-gate-v1/CLOSEOUT_AUDIT_2026-04-10.md`
  closes the lane on the shipped gate and follow-on policy.

## Gate set

### Direct checker

```bash
python3 tools/check_workstream_catalog.py
```

### Fast gate integration

```bash
python3 tools/gates_fast.py --skip-fmt --skip-nextest
```

### Gate entrypoint syntax

```bash
python3 -m py_compile tools/check_workstream_catalog.py tools/gates_fast.py tools/pre_release.py
```

### Diff hygiene

```bash
git diff --check
```

## Evidence anchors

- `docs/workstreams/workstream-catalog-integrity-gate-v1/DESIGN.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/TODO.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/MILESTONES.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/BASELINE_AUDIT_2026-04-10.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/M2_PROOF_SURFACE_2026-04-10.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/README.md`
- `docs/workstreams/standalone/README.md`
- `tools/check_workstream_catalog.py`
- `tools/gates_fast.py`
- `tools/pre_release.py`

## Reference posture

- Keep the README files curated and human-authored.
- Guard the catalog sections and tracked count lines deterministically.
- Treat broader README generation, full order normalization, or docs automation as separate future
  lanes.
