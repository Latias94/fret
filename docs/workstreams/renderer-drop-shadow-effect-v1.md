Status: Done (ADR + contract + wgpu implementation + conformance + perf baseline)

This workstream defines a bounded, portable **blur-based drop shadow** mechanism surface for
general UI content (not just text). The goal is to support the common “elevation shadow” vocabulary
in a way that:

- is deterministic and capability-gated,
- remains viable on wasm/WebGPU and mobile GPUs,
- and does not require an unbounded custom shader contract.

Text already has a cheap `TextShadowV1` (single layer, no blur). This workstream targets the
general, blur-based shadow that many UI designs expect for cards, popovers, and overlays.

## Why this exists

Without a first-class shadow surface, authors often approximate shadows by:

- emitting many quads (expensive op count, difficult to standardize),
- or building bespoke offscreen pipelines at call sites (drift-prone, hard to gate).

A bounded shadow contract lets the renderer:

- centralize intermediate reuse and downsample policy,
- keep batching and budgeting honest,
- and provide stable conformance + perf gates.

## Non-goals (v1)

- No “CSS filter: drop-shadow()” parity surface with arbitrary blur/spread semantics.
- No inner shadows.
- No multi-layer elevation stacks in core (recipes remain ecosystem policy).

## Proposed contract surface (v1)

Add a new effect step variant to `fret-core::scene::EffectStep`:

- `EffectStep::DropShadowV1(DropShadowV1)`

Contract properties:

- **single layer**
- **bounded blur radius** (clamped)
- **solid color**
- `offset_px` is in logical pixels (pre-scale-factor)
- `downsample` is a bounded hint (1–4) that participates in deterministic budgets

This step is intended for `EffectMode::FilterContent` (content is rendered to an intermediate).
Under `EffectMode::Backdrop`, renderers deterministically degrade by skipping the step (no shadow).

## Semantics (v1)

For an effect layer with children:

1. Render children into an offscreen intermediate (as `FilterContent` already requires).
2. Compute the shadow image from the children’s coverage:
   - blur coverage by `blur_radius_px` (downsample permitted),
   - tint by `color`,
   - translate by `offset`.
3. Composite shadow **behind** the original children content within the effect bounds.
4. Continue any subsequent effect steps (if the chain includes additional steps).

Deterministic degradation rules:

- If `EffectMode::Backdrop`: skip.
- If intermediate budgets cannot satisfy the blur: skip.

## Cost model notes

- Blur implies extra passes and bandwidth (offscreen + filter).
- Shadow is most stable when:
  - bounds are tight and scissored,
  - intermediate reuse is effective,
  - and downsample is budget-driven and deterministic.

## Gates (required)

- `python3 tools/check_layering.py`
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
- GPU readback conformance proving:
  - shadow exists behind content,
  - offset is applied,
  - color is applied,
  - degradation is deterministic when disabled/budgeted.
- Perf gate with a checked-in baseline for a shadow-heavy scene (cards/popovers).
  - `tools/diag-scripts/drop-shadow-v1-steady.json`
  - `docs/workstreams/perf-baselines/drop-shadow-v1-steady.windows-rtx4090.v1.json`
  - `tools/perf/diag_drop_shadow_v1_gate.ps1`

## Tracking

- TODOs: `docs/workstreams/renderer-drop-shadow-effect-v1-todo.md`
- Milestones: `docs/workstreams/renderer-drop-shadow-effect-v1-milestones.md`

Related ADR:

- `docs/adr/0286-drop-shadow-effect-step-v1.md`
