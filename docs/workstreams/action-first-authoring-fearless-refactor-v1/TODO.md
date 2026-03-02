# Action-First Authoring + View Runtime (Fearless Refactor v1) — TODO

Status: Active
Last updated: 2026-03-01

Related:

- Design: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- Evidence/gates: `docs/workstreams/action-first-authoring-fearless-refactor-v1/EVIDENCE_AND_GATES.md`

ADRs (decision gates for this workstream):

- `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`
- `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done
- `[!]` blocked

ID format:

- `AFA-{area}-{nnn}`

---

## A. Decision + Contract Locking

- [ ] AFA-adr-001 Review ADR 0307 (actions) for scope/ownership boundaries.
- [ ] AFA-adr-002 Review ADR 0308 (view runtime) for hook order/keying rules and cache boundary semantics.
- [ ] AFA-adr-003 Update `docs/adr/README.md` jump table with new action/view ADR anchors.
- [x] AFA-adr-004 Decide keymap strategy (v1):
  - Decision: `ActionId == CommandId` (alias/wrapper; no keymap schema churn in v1).
  - Evidence: ADR 0307 “v1 decision (locked)”.
- [x] AFA-adr-005 Add a short action naming convention note (namespace + `.v1` suffix).
  - Goal: keep IDs predictable for GenUI and future frontends.
- [ ] AFA-adr-006 Add an observability checklist for action dispatch + view dirty/reuse.
  - Evidence: `docs/workstreams/action-first-authoring-fearless-refactor-v1/EVIDENCE_AND_GATES.md`

---

## B. Action System (Additive v1)

- [ ] AFA-actions-010 Define the `ActionId` type and metadata surface.
  - Evidence: `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`
- [ ] AFA-actions-011 Provide an ecosystem macro for defining typed unit actions with stable IDs.
  - Goal: avoid stringly `"my.action.id"` constants in app code.
- [ ] AFA-actions-012 Add a minimal action handler table API for views/frontends.
  - Goal: IR binds `ActionId`; handlers live in view/app layer.
- [ ] AFA-actions-013 Integrate action availability queries with input dispatch v2 semantics.
  - Evidence: `docs/adr/0218-input-dispatch-phases-prevent-default-and-action-availability-v2.md`
- [ ] AFA-actions-014 Add diagnostics traces for:
  - keymap resolution → action id,
  - availability gating outcome,
  - dispatch path resolution.
- [ ] AFA-actions-015 Converge command palette/menu invocation with action dispatch.
  - Goal: palette/menu triggers and pointer triggers share the same action pipeline.

---

## C. View Runtime + Hooks (Ecosystem)

- [x] AFA-view-020 Decide crate placement for the view runtime:
  - Decision: land in `ecosystem/fret` for v1; defer split crate until after adoption.
  - Evidence: ADR 0308 “v1 decision (locked)”.
- [ ] AFA-view-021 Implement a minimal `View` trait + `ViewCx` with:
  - action handler registration,
  - `notify()` dirty marking,
  - `use_state` (element/view state slots),
  - `use_selector` (re-export from `fret-selector`),
  - `use_query` (re-export from `fret-query`).
- [ ] AFA-view-022 Define and document hook keying rules:
  - stable callsite key for non-loop hooks,
  - required keyed variants for loops (`use_*_keyed`),
  - diagnostics for misuse (debug-only).
- [ ] AFA-view-023 Add a view-cache boundary helper aligned with ADR 0213:
  - “cached unless dirty” semantics,
  - inspection/picking disables reuse.
- [ ] AFA-view-024 Provide an adapter path for MVU:
  - keep MVU available while views are adopted,
  - document “when to use MVU vs View” in cookbook guidance.
- [ ] AFA-view-025 Add view-level observability:
  - “why did this view rebuild?”
  - “why was reuse skipped?”
  - “which models/globals were observed?”

---

## D. Frontend Convergence (Declarative + imui + GenUI)

- [ ] AFA-frontends-030 Add an imui seam to dispatch `ActionId` without string commands.
  - Evidence: `docs/workstreams/imui-authoring-facade-v2.md`
- [ ] AFA-frontends-031 Ensure imui outputs stable semantics/test IDs for diag scripts.
  - Evidence: `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- [ ] AFA-frontends-032 Align GenUI action bindings with `ActionId` conventions (namespace/versioning).
  - Evidence: `docs/workstreams/genui-json-render-v1.md`
- [ ] AFA-frontends-033 Add at least one cross-frontend demo:
  - a Rust view triggers an action,
  - an imui panel triggers the same action,
  - an optional GenUI spec triggers a catalog-approved action ID.

---

## E. Adoption (Cookbook + Gallery + Editor-grade shells)

- [ ] AFA-adopt-040 Migrate 2–3 cookbook demos to the new View + actions path.
  - Suggested: `apps/fret-cookbook/examples/hello.rs`, `overlay_basics.rs`, `commands_keymap_basics.rs`.
- [ ] AFA-adopt-041 Add at least one ui-gallery page/snippet using actions + view runtime.
- [ ] AFA-adopt-042 Add one editor-grade harness adoption:
  - docking/workspace shell uses actions for tab/command semantics (where appropriate).
- [ ] AFA-adopt-043 Update `fretboard` scaffold templates to prefer action-first patterns (once v1 is stable).
  - Rule: do not ship two different default paradigms in templates.

---

## F. Evidence + Regression Gates

- [ ] AFA-gates-050 Add at least one scripted diag repro that exercises:
  - a keybinding → action dispatch,
  - a button click → action dispatch,
  - action availability gating (disabled state) under a modal barrier.
- [ ] AFA-gates-051 Add compile-only wasm smoke gates for the new view runtime surface.
- [ ] AFA-gates-052 Add a small set of unit tests for action routing / handler table behavior.
- [ ] AFA-gates-053 Add a “risk matrix” review pass for M0/M1 (see `RISK_MATRIX.md`).

---

## G. Cleanup and Deletion (Leave it clean)

This phase is intentionally last.

- [ ] AFA-clean-060 Deprecate legacy routing glue that is no longer recommended in templates/docs.
  - Candidate: per-frame-only message routers in places that should be action-first.
- [ ] AFA-clean-061 Update docs and templates:
  - `docs/README.md` state management section shows actions + view runtime as the golden path.
  - `fretboard` templates generate action-first demos by default.
- [ ] AFA-clean-062 Delete or quarantine redundant APIs/modules once adoption is complete.
  - Rule: do not delete until all in-tree demos + ecosystem crates have migrated or have explicit “legacy” labeling.
