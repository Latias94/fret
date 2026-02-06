# imui Ecosystem Facade v2 (Stabilization + Authoring Alignment)

Status: In progress (M0 locked; M1+ pending; planning note, not an ADR)
Last updated: 2026-02-06

This workstream starts after `imui` ecosystem facade v1 is functionally complete.

v1 delivered the practical baseline:

- `fret-imui` stays minimal/policy-light,
- `fret-ui-kit::imui` hosts egui/imgui-like facade ergonomics,
- floating area/window primitives exist (in-window),
- overlays and response signals have regression coverage,
- non-docking OS-window promotion is explicitly deferred.

Related:

- `docs/workstreams/imui-ecosystem-facade-v1.md`
- `docs/workstreams/imui-ecosystem-facade-v1-todo.md`
- `docs/workstreams/imui-ecosystem-facade-perf-v1.md`
- `docs/workstreams/imui-shadcn-adapter-v1.md`
- `docs/workstreams/docking-multiwindow-imgui-parity.md`

---

## 1) Why v2

v1 prioritized capability and workflow velocity. v2 prioritizes stability and ecosystem-scale extensibility.

Main goals:

1) Define which `ResponseExt` semantics are stable enough to graduate into a shared authoring contract.
2) Make adapter seams explicit so third-party crates can add immediate wrappers without duplicating policy.
3) Keep ImGui-aligned interaction quality improving (especially floating + popup choreography).
4) Convert existing perf guidance into enforceable regression gates.

---

## 2) Invariants (Do Not Break)

- `fret-imui` remains an authoring frontend, not a parallel runtime.
- Canonical component state machines remain single-sourced; facade wrappers map/report signals.
- `UiBuilder` patch vocabulary remains the common styling/layout language.
- Docking remains the only v1/v2 path for OS-window tear-off/promotion.
- Non-docking floating windows keep in-window behavior with capability-gated future expansion only.

---

## 3) Milestones

### M0 - Baseline lock and admission criteria

- M0 lock details are normative in `docs/workstreams/imui-ecosystem-facade-v2-m0-contracts.md`.
- Snapshot current v1 behavior as the reference baseline.
- Define what counts as a "breaking" `ResponseExt` change.
- Define contribution requirements for new wrappers (tests + docs + no duplicated policy).

### M1 - `ResponseExt` stabilization and graduation

- Partition `ResponseExt` into:
  - stable core candidates (possible `fret-authoring` graduation),
  - facade-only experimental signals.
- Define compatibility policy:
  - additive-only in stable core,
  - deprecation window before removals/renames.
- Add compile-level smoke assertions for shared contract boundaries.

### M2 - Adapter seam contract for third-party ecosystems

- Define a minimal adapter contract for canonical components:
  - identity in,
  - signal reporter out,
  - optional metadata for geometry/focus restore.
- Publish extension guidelines with one canonical "author once, adapt many" template.
- Add at least one non-shadcn third-party-style adapter example.

### M3 - ImGui-aligned interaction polish (without runtime split)

- Improve popup/select/window interaction choreography where thin adapters are currently minimal.
- Keep focus/restore and dismiss behavior aligned with overlay contracts.
- Gate changes with `fretboard diag` scripts for drag/resize/menu coexistence.

### M4 - Perf hardening from guidance to gates

- Turn perf guide rules into review checklist items.
- Add at least one micro/behavior gate that catches allocation-heavy regressions in hot wrappers.
- Ensure large-list wrapper examples use keyed identity and virtualization by default.

### M5 - v2 readiness review

- Re-audit layering boundaries (`fret-imui` vs `fret-ui-kit` vs recipe layers).
- Reassess whether any facade subset should move to a dedicated crate.
- Publish "v2 stable subset" and defer everything else explicitly.

---

## 4) Deliverables

- v2 design note + TODO tracker.
- One extension template for external widget authors.
- Updated tests/diag scripts proving no regression in core interaction flows.
- Clear stable/experimental labeling for response and adapter APIs.

---

## 5) Exit Criteria

v2 is considered complete when:

- stable `Response` subset and compatibility rules are documented and enforced,
- adapter seam is documented and validated by real adapters,
- interaction polish changes are covered by tests/diag,
- perf guidance has at least basic automated enforcement,
- workstream TODOs are either done or explicitly deferred with rationale.

---

## 6) Risks and Mitigations

1) Contract freeze too early
- Mitigation: graduate only high-confidence signals; keep others facade-local.

2) Hidden policy duplication in adapters
- Mitigation: require source-of-truth mapping evidence in reviews.

3) Perf regressions from convenience wrappers
- Mitigation: treat perf checklist + targeted tests as merge gates.

4) Scope creep into runtime concerns
- Mitigation: reject proposals that require a second immediate runtime or break existing layering.

---

## 7) Open Questions

1) Which specific `ResponseExt` fields are safe to graduate first into shared authoring contracts?
2) Should adapter seam metadata include optional a11y/focus intents in v2, or defer to v3?
3) What is the minimum perf gate that is cheap enough to run in normal CI yet useful?
