# UI Focus + Overlay Fearless Refactor (v1)

This workstream hardens the UI runtime’s most failure-prone interaction mechanisms:

- focus containment and focus-visible
- overlay stacks (menus/popovers/dialogs/tooltips)
- outside-press dismissal semantics (Radix/DOM-aligned)
- retained/view-cache edge cases (temporary parent-pointer inconsistency)

The plan is intentionally phased:

1) **A (Correctness hardening):** make outside-press containment robust under retained/view-cache by
   using child-edge reachability (not parent pointers) for critical decisions.
2) **B (Semantic alignment):** allow policy layers to prevent the *default side effects* of outside
   press (notably focus clearing) using `prevent_default`-style contracts.
3) **C (Architecture refactor):** move toward a GPUI-like per-frame “dispatch snapshot” so event
   routing, focus containment, tab order, and hit-test are anchored to a single coherent frame view.

See:

- `DESIGN.md` for the overall design and invariants
- `TODO.md` / `MILESTONES.md` for sequencing
- `EVIDENCE_AND_GATES.md` for regression artifacts and minimum gates
- `OPEN_QUESTIONS.md` for unresolved semantics
- `UPSTREAM_AUDIT_ZED_GPUI.md` for upstream mechanism notes (Zed / GPUI)
- Related ADRs: `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`

## Current status

- Phase A shipped: containment and branch exclusion use child-edge reachability (not parent pointers).
- Phase B shipped: `prevent_default()` suppresses the runtime’s default focus-clearing side effect on
  outside press.
- Phase C is in design and decomposition: `M2_DISPATCH_SNAPSHOT_DESIGN.md` (dispatch snapshot).
- View-cache + hover correctness hardening: HoverRegion “hover edge” transitions disable view-cache
  reuse for the containing cache roots (rerender-on-hover-edge) so hover-driven overlays and
  pseudoclass-driven style changes cannot be hidden behind cache hits.

Evidence anchors live in:

- `docs/adr/IMPLEMENTATION_ALIGNMENT.md` (row for ADR 0069)
- `crates/fret-ui/src/tree/tests/outside_press.rs`
- `crates/fret-ui/src/declarative/tests/interactions/dismissible.rs`

## Fast local gates

For local iteration (especially when the full `cargo nextest run -p fret-ui` suite is slow), prefer
the targeted gates in `EVIDENCE_AND_GATES.md`.
