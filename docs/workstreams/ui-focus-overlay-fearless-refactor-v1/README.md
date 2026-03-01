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
- Related ADRs: `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`

