# M2 Proof Surface — 2026-04-10

Status: closed proof record

Related:

- `docs/workstreams/workstream-catalog-integrity-gate-v1/DESIGN.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
- `docs/workstreams/README.md`
- `docs/workstreams/standalone/README.md`
- `tools/check_workstream_catalog.py`
- `tools/gates_fast.py`
- `tools/pre_release.py`

## Proof commands

```bash
python3 tools/check_workstream_catalog.py
python3 tools/gates_fast.py --skip-fmt --skip-nextest
python3 -m py_compile tools/check_workstream_catalog.py tools/gates_fast.py tools/pre_release.py
python3 tools/pre_release.py --skip-fmt --skip-clippy --skip-nextest --skip-icons --skip-release-closure --skip-portable-time --skip-diff-check
git diff --check
```

## Observed proof

### 1) Direct checker

- `python3 tools/check_workstream_catalog.py`
  passed after the README fixes.

This proves the checker can parse the intended sections without confusing ordinary prose links for
catalog rows.

### 2) Fast gate integration

- `python3 tools/gates_fast.py --skip-fmt --skip-nextest`
  passed.

This proves the catalog checker now runs through the common fast maintainer gate alongside existing
workspace policy checks.

### 3) Gate entrypoint syntax

- `python3 -m py_compile tools/check_workstream_catalog.py tools/gates_fast.py tools/pre_release.py`
  passed.

This proves the new checker wiring is syntactically valid across all three Python entrypoints.

### 4) Pre-release chain observation

- `python3 tools/pre_release.py --skip-fmt --skip-clippy --skip-nextest --skip-icons --skip-release-closure --skip-portable-time --skip-diff-check`
  executed the new `Workstream catalog integrity` step first and then stopped on an unrelated
  pre-existing ADR duplicate failure.

This proves the checker is wired into the broader pre-release policy chain even though the overall
chain is not green in the current repository state.

### 5) Diff hygiene

- `git diff --check`
  passed after the docs and tooling updates.

## Proof conclusion

The lane's intended outcome is now demonstrated:

- the workstream catalog has a deterministic structural guard,
- the direct checker and fast gate exercise it successfully,
- the broader pre-release chain now includes it ahead of unrelated existing ADR failures,
- and the current README drift has been repaired.
