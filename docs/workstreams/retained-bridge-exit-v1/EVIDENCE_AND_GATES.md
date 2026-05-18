# Retained Bridge Exit v1 Evidence and Gates

## 2026-05-18 - RBX-M1-010 Docking retained bridge audit

Claim verified:

- `fret-docking` retained bridge usage has been audited and classified.
- The first implementation slice has been selected as `RBX-M1-020`: extract docking split geometry
  and handle painting from `fret_ui::retained_bridge`.

Evidence:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_010_DOCKING_RETAINED_BRIDGE_AUDIT_2026-05-18.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking`
  - Result: passed, 111 tests.
  - Scope proven: existing docking behavior remains green after the audit/documentation update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: retained bridge allowlist and crate layering still pass.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed.
  - Scope proven: workstream catalog indexes remain valid.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed documentation has no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M1-010` is an audit/documentation task; the task-local docking gate and layering
    gate are sufficient.
