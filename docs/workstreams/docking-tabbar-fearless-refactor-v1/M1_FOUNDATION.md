# Docking TabBar Fearless Refactor v1 — Milestone 1 (Foundation)

## Outcome

Establish a stable, testable “drop resolution” contract for docking TabBar and a minimal kernel that
can be unit-tested independent of rendering.

## Deliverables

- Documented contract surface (resolved zones + insert index definition).
- A `tab_bar_kernel` (pure) used by the docking tab bar.
- Baseline diagnostics gates that must stay green:
  - drop-at-end resolves `insert_index == tab_count`
  - cancel/escape leaves docking state unchanged

## Exit criteria

- Kernel unit tests cover at least:
  - 1-tab stack drop-end
  - 2-tab stack drop-end
  - cross-pane drop-end
- A docking-arbitration diag suite run can be used as evidence (bundle path recorded).

