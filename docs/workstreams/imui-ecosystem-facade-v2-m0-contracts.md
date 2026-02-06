# imui Ecosystem Facade v2 M0 Contracts (Baseline + Contribution Admission)

Status: Accepted (M0 locked)
Last updated: 2026-02-06

This note defines the concrete contracts for v2 milestone M0.

Related:

- `docs/workstreams/imui-ecosystem-facade-v2.md`
- `docs/workstreams/imui-ecosystem-facade-v2-todo.md`
- `docs/workstreams/imui-ecosystem-facade-v1.md`
- `docs/workstreams/imui-ecosystem-facade-v1-todo.md`
- `docs/workstreams/imui-ecosystem-facade-perf-v1.md`

---

## 1) Frozen v1 Baseline

The v2 baseline is frozen at the point where v1 TODO completion and perf guidance were landed.

Baseline anchors:

- `f9e5bf7` (`docs(imui): add perf guide and close remaining v1 TODOs`)
- `becf940` (`docs(imui): add ecosystem facade v2 milestones and tracker`)

Normative v1 behavior snapshot:

- `ResponseExt` provides shipped v1 signals (click variants, drag lifecycle, context-menu request,
  long-press/holding, geometry helpers).
- Immediate wrappers for common controls exist and are covered in `fret-imui` tests.
- Floating area/window primitives are in-window and layered through existing overlay/layer contracts.
- Non-docking OS-window promotion is out of scope and remains deferred.
- Perf guidance is defined in `imui-ecosystem-facade-perf-v1.md`.

Non-goals for v2 M0:

- no semantic redesign of existing v1 wrappers,
- no runtime ownership migration,
- no generalized window promotion implementation.

---

## 2) Post-Freeze Change Policy

1) Existing v1 behavior is the compatibility baseline unless a workstream item explicitly allows breakage.
2) Additive APIs are preferred (`*_ex`, new option fields, new helpers) over mutation/removal.
3) Any behavior-changing patch must update:
   - `docs/workstreams/imui-ecosystem-facade-v2-todo.md`,
   - evidence anchors (tests/diag/docs),
   - migration notes when call-site expectations change.
4) For interaction choreography changes (focus, dismiss, drag/resize handoff), a scripted diag path is
   required before closure.

---

## 3) Breaking Response Behavior Criteria

The following are treated as breaking behavior for existing wrappers:

1) Edge-signal timing contract change
- Example: `clicked` no longer clear-on-read/once-per-event, or fire timing moves across frames.

2) Semantic polarity change
- Example: a signal changes truth semantics (`hovered`, `changed`, `context_menu_requested`) without
  an explicit renamed API.

3) Geometry contract change
- Example: switching from last-frame stabilized bounds to same-frame-only semantics for existing APIs.

4) Focus/dismiss chain change
- Example: popup/window close behavior no longer restores focus according to current overlay policy.

5) Default behavior contract change
- Example: default options change interaction outcomes (open/close/resize) for existing call sites.

Allowed non-breaking changes:

- additive fields/methods with unchanged defaults,
- new opt-in option flags,
- additional diagnostics metadata that does not alter behavior.

---

## 4) Wrapper Contribution Checklist (Admission Gate)

Every new or modified wrapper should pass all items below before merge.

### 4.1 Layering and ownership

- Keep `fret-imui` policy-light; place policy-heavy behavior in `fret-ui-kit`/recipe layers.
- Prefer canonical adapter delegation; do not duplicate complex interaction state machines.
- Keep API shape aligned with `UiWriter` + current facade extension style.

### 4.2 Identity and performance

- Use stable keyed identity for reorderable/dynamic collections.
- Avoid per-frame heap allocations in hot paths when a borrowed or reusable form exists.
- Respect the v1 perf guide (`imui-ecosystem-facade-perf-v1.md`).

### 4.3 Test and regression evidence

- Add at least one targeted `cargo nextest` behavior test for the wrapper change.
- For multi-step choreography (drag/resize/focus/popup), add or update one `fretboard diag` script.
- Preserve wasm compile smoke for facade surfaces when shared contracts are touched.

### 4.4 Documentation and tracking

- Update relevant workstream TODO item with evidence anchors.
- Document user-visible behavior changes and migration notes if expectations change.
- Keep extension guidance in sync when adapter seams are affected.

---

## 5) Review Matrix (Fast Triage)

- Small wrapper tweak (no choreography): test + TODO evidence required.
- Interaction choreography change: test + diag + TODO evidence required.
- Contract-level change (`ResponseExt`/shared surface): test + compile smoke + docs + migration note required.

---

## 6) M0 Completion Mapping

- `IMUIECO2-scope-000`: sections 1 and 2.
- `IMUIECO2-scope-001`: section 3.
- `IMUIECO2-docs-002`: section 4 and section 5.
