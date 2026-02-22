# Renderer Upstream Semantics Parity v1 — TODO

## M0 — Setup (minimal)

- [ ] Create a single “parity note” template section in this file (copy/paste friendly).
- [ ] Pick 1 upstream seam to start with (recommended: scissor coordinate spaces).

## M1 — First parity note (scissor spaces)

- [ ] Zed/GPUI: identify how render-target origin offsets and scissors are represented and applied.
- [ ] Fret: record current representation and mapping:
  - `AbsoluteScissorRect` vs `LocalScissorRect`
  - `RenderSpace` mapping and scissor translation/clamping
- [ ] Decide: gap vs deliberate difference; record rationale.
- [ ] If gap: add the smallest guardrail (validator or test) *before* changing implementation.

## M2 — Clip/mask composition parity

- [ ] Compare push-time capture semantics for clip path / image mask stacks.
- [ ] Compare cache key strategy and reuse heuristics for mask targets.
- [ ] Add one conformance test that breaks if clip capture semantics drift.

## M3 — Intermediate reuse / lifetime parity

- [ ] Compare intermediate allocation/reuse strategy vs upstream:
  - lifetime model,
  - eviction/budgeting policy,
  - determinism under contention.
- [ ] Add one targeted unit test for “release after last use” stability in plan shape.

## Notes / parity template

Copy/paste for each seam:

- Seam:
- Upstream evidence anchors:
- Fret evidence anchors:
- Observed behavior:
- Differences (gap vs deliberate):
- Proposed guardrail:
- Follow-up refactor steps:

