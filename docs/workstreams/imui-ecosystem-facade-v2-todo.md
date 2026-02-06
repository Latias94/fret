# imui Ecosystem Facade v2 - TODO Tracker

Status: In progress (M0-M3 complete; M4+ pending)
Last updated: 2026-02-06

This tracker covers:

- `docs/workstreams/imui-ecosystem-facade-v2.md`

Related:

- `docs/workstreams/imui-ecosystem-facade-v1.md`
- `docs/workstreams/imui-ecosystem-facade-v1-todo.md`
- `docs/workstreams/imui-ecosystem-facade-perf-v1.md`

Legend:

- [ ] open
- [~] in progress
- [x] done
- [!] blocked / needs decision

Tracking format:

- ID: `IMUIECO2-{area}-{nnn}`
- Areas:
  - `scope` (contracts and boundaries)
  - `resp` (`ResponseExt` stabilization)
  - `adapter` (delegation seam and extension model)
  - `float` (floating/popup interaction polish)
  - `perf` (allocation/perf gates)
  - `test` (nextest/diag/compile gates)
  - `docs` (guides/migration)

---

## M0 - Baseline lock

Exit criteria:

- v1 baseline is captured and referenced.
- v2 contribution admission criteria are documented.

- [x] IMUIECO2-scope-000 Capture v1 frozen baseline and change policy.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v2-m0-contracts.md` (sections 1 and 2).
- [x] IMUIECO2-scope-001 Define "breaking response behavior" criteria.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v2-m0-contracts.md` (section 3).
- [x] IMUIECO2-docs-002 Publish wrapper contribution checklist (tests + docs + delegation evidence).
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v2-m0-contracts.md` (sections 4 and 5).

---

## M1 - `ResponseExt` stabilization

Exit criteria:

- Stable vs experimental response signals are explicitly partitioned.
- Potential `fret-authoring` graduation path is concrete.

- [x] IMUIECO2-resp-010 Define stable-core candidate response fields.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v2-m1-response-stability.md` (section 1).
- [x] IMUIECO2-resp-011 Define compatibility/deprecation policy for stable core.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v2-m1-response-stability.md` (sections 2 and 3).
- [x] IMUIECO2-test-012 Add compile smoke checks around shared response boundaries.
  - Evidence: `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
  - Evidence (local): `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke`

---

## M2 - Adapter seam for ecosystem extensibility

Exit criteria:

- Adapter seam contract is documented and validated by examples.
- Third-party-style extension template is available.

- [x] IMUIECO2-adapter-020 Specify minimal signal reporter/delegation contract.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v2-m2-adapter-seam.md` (section 1).
  - Evidence: `ecosystem/fret-ui-kit/src/imui/adapters.rs` (`AdapterSignalRecord`, `AdapterSeamOptions`, `report_adapter_signal`).
- [x] IMUIECO2-adapter-021 Add one adapter template doc for external widget crates.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v2-m2-adapter-seam.md` (section 2).
- [x] IMUIECO2-adapter-022 Land one non-shadcn adapter example using the seam.
  - Evidence: `ecosystem/fret-ui-kit/src/imui/adapters.rs` (`button_adapter`, `checkbox_model_adapter`).
  - Evidence (local): `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke`

---

## M3 - Floating/popup interaction polish

Exit criteria:

- ImGui-aligned behavior improvements are made without policy/runtime duplication.
- Regressions are covered by scripted diagnostics.

- [x] IMUIECO2-float-030 Improve popup/select wrapper choreography under adapter-first rules.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v2-m3-popup-floating-polish.md` (section 1).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`select_model_ex`).
- [x] IMUIECO2-float-031 Verify focus restore + dismiss consistency after polish.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v2-m3-popup-floating-polish.md` (section 2).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`select_popup_escape_closes_and_restores_trigger_focus`).
- [x] IMUIECO2-test-032 Add/extend `fretboard diag` coverage for floating/popup coexistence.
  - Evidence: `tools/diag-scripts/imui-float-window-select-popup-coexistence.json`
  - Evidence: `apps/fret-examples/src/imui_floating_windows_demo.rs`

---

## M4 - Performance hardening

Exit criteria:

- Perf guide is reflected in enforceable review/test gates.

- [ ] IMUIECO2-perf-040 Convert perf guide into review checklist items.
- [ ] IMUIECO2-perf-041 Add targeted gate for hot wrapper allocation regression.
- [ ] IMUIECO2-perf-042 Ensure large-list examples default to keyed identity + virtualization.

---

## M5 - Readiness review

Exit criteria:

- v2 stable subset is published with explicit deferrals.

- [ ] IMUIECO2-scope-050 Re-audit layering boundaries and dependency ownership.
- [ ] IMUIECO2-docs-051 Publish v2 stable subset + deferred backlog.
