# Hotpatch Devloop Alignment v1 — Milestones

## M0 — Baseline UX + observability (short)

Deliverables:

- Workstream docs exist and are linked:
  - `docs/workstreams/hotpatch-devloop-alignment-v1.md`
  - `docs/workstreams/hotpatch-devloop-alignment-v1-todo.md`
  - `docs/workstreams/hotpatch-devloop-alignment-v1-milestones.md`
- `fretboard dev native --hotpatch` is the recommended path and prints a stable startup summary.
- Logs are easy to find and referenced in the summary.

Exit criteria:

- A user can answer “am I actually hotpatching Rust code?” from the first 10 lines of output.

## M1 — Predictable fallback ladder (time-boxed)

Deliverables:

- A single documented fallback ladder is implemented:
  - patch applied → runner reload boundary
  - known-unsafe view-level hotpatch → explicit warning + boundary-only behavior
  - repeated crash → restart guidance (and optionally a supervised restart)

Exit criteria:

- Windows “patched view crash” does not present as a silent failure; the user gets a clear next action.

## M2 — Theme reload (no compile) (time-boxed)

Deliverables:

- Theme reload contract + a minimal implementation for one demo.
- A simple repro gate exists (manual instructions or a `fretboard diag` script).

Exit criteria:

- Change a theme token file and see UI update without rebuilding.

## M3 — Asset reload (no compile) (time-boxed)

Deliverables:

- Asset invalidation contract + a minimal implementation for one asset type (svg/png/fonts).

Exit criteria:

- Replace an asset file and see UI update without rebuilding.

## M4 — Hot literals (no compile) (time-boxed)

Deliverables:

- Hot-literals file format + a minimal implementation for one demo.

Exit criteria:

- Update a label/tooltip via a data file and see UI update without rebuilding.

