# Milestones

## M0: Phase A shipped (containment hardening)

Done when:

- Outside-press containment and branch exclusion do not rely on parent pointers.
- New regression test covers “stale parent pointers” without needing a full app harness.
- `cargo nextest run -p fret-ui` is green (targeted filters are acceptable for local iteration).

Status: Done.

## M1: Phase B shipped (prevent default suppresses focus clearing)

Done when:

- `prevent_default()` from dismissible outside-press hooks suppresses default focus clearing.
- Regression tests exist for:
  - prevented outside press keeps focus stable
  - non-prevented outside press clears focus (baseline)
- No behavior changes are introduced for non-dismissible observer users.

Status: Done.

## M2: Phase C design locked (dispatch snapshot)

Done when:

- A detailed “dispatch snapshot” design exists (data model + build phase + consumers).
- Migration is decomposed into 3–6 landable PRs with clear acceptance criteria.
- Evidence plan exists (diag script or debug report) to prove parity with Phase A/B invariants.

Status: Draft (design exists; migration breakdown drafted; diagnostics + implementation still TODO).

Notes:

- PR0 (types + builder entrypoint) is landed: `crates/fret-ui/src/tree/dispatch_snapshot.rs` and
  `crates/fret-ui/src/tree/ui_tree_debug/query.rs` (`debug_dispatch_snapshot`).
- PR1 (parity report) is landed: `crates/fret-ui/src/tree/ui_tree_debug/query.rs`
  (`debug_dispatch_snapshot_parity`).
- PR2 (outside-press uses snapshot, Phase A fallback) is landed:
  - `crates/fret-ui/src/tree/ui_tree_outside_press.rs`
  - `crates/fret-ui/src/tree/dispatch/window.rs`
