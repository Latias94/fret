# imui Ecosystem Facade v3 - TODO Tracker

Status: In progress (M0+ pending)
Last updated: 2026-02-06

This tracker covers:

- `docs/workstreams/imui-ecosystem-facade-v3.md`

Related:

- `docs/workstreams/imui-ecosystem-facade-v2.md` (baseline)
- `docs/workstreams/docking-multiwindow-imgui-parity.md` (OS-window tear-off parity)
- `docs/workstreams/code-editor-ecosystem-v1.md` (text/editor ecosystem)

Legend:

- [ ] open
- [~] in progress
- [x] done
- [!] blocked / needs decision

Tracking format:

- ID: `IMUIECO3-{area}-{nnn}`
- Areas:
  - `scope` (contracts and boundaries)
  - `float` (floating windows / z-order / focus)
  - `dock` (docking handshake touchpoints)
  - `adapter` (ecosystem ABI and seam evolution)
  - `resp` (response signal graduation decisions)
  - `text` (text/editor integration hooks)
  - `perf` (allocation/perf gates)
  - `test` (nextest/diag/compile gates)
  - `docs` (guides/migration)

---

## M0 - Scope lock + admission checklist

Exit criteria:

- v3 boundaries relative to docking and text ecosystems are explicit.
- breaking-change criteria for floating flags/behavior are documented.

- [ ] IMUIECO3-scope-000 Lock v3 scope boundaries and explicit deferrals.
- [ ] IMUIECO3-docs-001 Add floating/z-order/focus admission checklist items.

---

## M1 - Floating window primitives (ImGui-aligned, in-window)

Exit criteria:

- `window(...)`/floating surface has an explicit options/flags API (subset).
- bring-to-front + focus restore are deterministic and gated.
- at least one diag script covers floating + popup coexistence under the new rules.

- [ ] IMUIECO3-float-010 Add `WindowFlags`/options surface for in-window floating windows.
- [ ] IMUIECO3-float-011 Add deterministic bring-to-front + focus choreography for floatings.
- [ ] IMUIECO3-float-012 Add minimal in-window z-order model that composes with overlay arbitration.
- [ ] IMUIECO3-test-013 Add nextest gates for window flag semantics (close/collapse/resize/move).
- [ ] IMUIECO3-test-014 Add/extend `fretboard diag` script(s) for floating + popup + drag/resize.

---

## M2 - Docking/multi-window handshake (tracked, docking-owned)

Exit criteria:

- imui facade touchpoints needed for docking parity are listed and linked to docking workstreams.

- [ ] IMUIECO3-dock-020 Document docking handshake touchpoints and required signals/metadata.

---

## M3 - Ecosystem extension ABI v1

Exit criteria:

- adapter seam template remains stable and is proven by a non-shadcn example.
- any metadata evolution is justified with duplication-reduction evidence.

- [ ] IMUIECO3-adapter-030 Audit adapter seam v2 and list v3 ABI changes (if any).
- [ ] IMUIECO3-adapter-031 Add one \"external widget crate\" style example (in-tree scaffold is OK).

---

## M4 - Text/editor bridge

Exit criteria:

- text/editor integration is explicit and delegated to the code-editor ecosystem (no fork).

- [ ] IMUIECO3-text-040 Define adapter hooks for editor-grade text surfaces.
- [ ] IMUIECO3-docs-041 Publish \"do not fork text engine\" integration guidance.

---

## M5 - Perf + regression gate upgrade

Exit criteria:

- perf gates are cheap, repeatable, and cover at least one floating hot path.

- [ ] IMUIECO3-perf-050 Expand perf guard tests beyond the v2 smoke baseline (target floating hot paths).
- [ ] IMUIECO3-test-051 Add a small CI-friendly gate matrix (contracts + perf + diag scripts).
