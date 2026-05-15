# TODO: UI Layout Dirty Breadth Data Table v1

Status: Closed
Last updated: 2026-05-15

## M0 - Baseline Attribution

- [x] Re-run retained data-table suite from the current committed state.
- [x] Re-run view-cache data-table torture suite from the current committed state.
- [x] Run `diag stats --sort invalidation --top 30` on the worst retained and view-cache bundles.
- [x] Decide whether node-level layout profiling is needed for the first slices.
      Existing `cache_roots`, `layout_request_build_roots`, and invalidation-walk diagnostics were
      sufficient to identify Input RAF churn and parent-dependent page content cache breadth.
- [x] Record bundle paths and first attribution notes in `EVIDENCE_AND_GATES.md`.

## M1 - Dirty-Cause Diagnostics

- [x] Audit existing boundary/layout invalidation diagnostics for enough cause information.
- [x] If needed, add a mechanism-owned dirty-cause field or counter without leaking table policy into
      `crates/fret-ui`.
- [x] Add a focused test or snapshot assertion for the diagnostic surface.
      No new diagnostic surface was added; existing fixture/test coverage remains the gate for
      `ViewCacheLayoutDirtyExpansion`.
- [x] Update `EVIDENCE_AND_GATES.md` with the exact interpretation rules.

## M2 - Policy Churn Reduction

- [x] Remove high-frequency decorative filter-input chrome motion from data-table recipes while
      preserving default shadcn Input transition parity.
- [x] Make data-table torture and retained-table torture page content caches contained when bounds
      are known, so local table state changes do not dirty the whole content pane.
- [x] Audit retained table filter/sort/pinning/reset paths for non-idempotent state writes.
      Existing toolbar sync paths guard unchanged `global_filter`, `column_filters`,
      `column_pinning`, visibility, and faceted model mirrors before writing. Reset intentionally
      writes only when filters exist. The remaining row/cell structural churn is from legitimate
      row/window membership changes, not a stale table-local mirror.
- [x] Avoid rebuilding or resyncing table-local derived state when values are unchanged.
      No additional `fret-ui-kit` state-write slice was justified for this lane; the retained
      table code already uses guarded output/model updates for the audited paths. Further row/cell
      structure reduction should be a narrower table-subtree follow-on.
- [x] Keep shadcn recipe changes in `ecosystem/fret-ui-shadcn`; keep headless table changes in
      `ecosystem/fret-ui-kit`.
- [x] Run retained table unit gates after each landable slice.

## M3 - Mechanism Dirty-Breadth Reduction

- [x] If attribution shows broad runtime invalidation, narrow the mechanism in `crates/fret-ui`.
      Mount-time `set_children_in_mount` now skips the redundant structural invalidation walk when
      attaching children to a detached parent that is already dirty on layout/paint/hit-test.
- [x] Preserve contained view-cache and retained boundary semantics.
- [x] Add focused tests for any invalidation narrowing.
- [x] Run `python3 tools/check_layering.py`.

## M4 - Closeout

- [x] Re-run retained and view-cache diag suites after the final slice.
- [x] Record before/after `diag stats` summaries and bundle paths for the first two slices.
- [x] Record final before/after `diag stats` summaries and bundle paths.
- [x] Update `WORKSTREAM.json`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md`.
- [x] Add `CLOSEOUT_AUDIT_2026-05-15.md` or a later dated closeout audit.
- [x] Run `python3 tools/check_workstream_catalog.py` and `git diff --check`.
