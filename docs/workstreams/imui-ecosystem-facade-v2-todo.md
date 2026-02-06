# imui Ecosystem Facade v2 - TODO Tracker

Status: Draft
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

- [ ] IMUIECO2-scope-000 Capture v1 frozen baseline and change policy.
- [ ] IMUIECO2-scope-001 Define "breaking response behavior" criteria.
- [ ] IMUIECO2-docs-002 Publish wrapper contribution checklist (tests + docs + delegation evidence).

---

## M1 - `ResponseExt` stabilization

Exit criteria:

- Stable vs experimental response signals are explicitly partitioned.
- Potential `fret-authoring` graduation path is concrete.

- [ ] IMUIECO2-resp-010 Define stable-core candidate response fields.
- [ ] IMUIECO2-resp-011 Define compatibility/deprecation policy for stable core.
- [ ] IMUIECO2-test-012 Add compile smoke checks around shared response boundaries.

---

## M2 - Adapter seam for ecosystem extensibility

Exit criteria:

- Adapter seam contract is documented and validated by examples.
- Third-party-style extension template is available.

- [ ] IMUIECO2-adapter-020 Specify minimal signal reporter/delegation contract.
- [ ] IMUIECO2-adapter-021 Add one adapter template doc for external widget crates.
- [ ] IMUIECO2-adapter-022 Land one non-shadcn adapter example using the seam.

---

## M3 - Floating/popup interaction polish

Exit criteria:

- ImGui-aligned behavior improvements are made without policy/runtime duplication.
- Regressions are covered by scripted diagnostics.

- [ ] IMUIECO2-float-030 Improve popup/select wrapper choreography under adapter-first rules.
- [ ] IMUIECO2-float-031 Verify focus restore + dismiss consistency after polish.
- [ ] IMUIECO2-test-032 Add/extend `fretboard diag` coverage for floating/popup coexistence.

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
