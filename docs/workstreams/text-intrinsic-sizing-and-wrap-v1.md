# Text Intrinsic Sizing + Wrap Semantics v1

Status: Active
Owner: (unassigned)
Start: 2026-02-19

## Why this workstream exists

We want shadcn/Tailwind-aligned component ecosystems without layout surprises. The highest-value
failure mode we keep hitting is:

- shrink-wrapped parents + word-wrapped text ⇒ pathological narrow intrinsic measurements
  (per-character wrapping / “vertical text”),
- or the opposite: long tokens force huge min-widths when authors actually wanted wrapping.

This workstream tracks the refactor to make text intrinsic sizing deterministic and policy-driven,
and to prevent ecosystem drift by providing clear authoring guidance and helpers.

## Sources of truth

- ADR 0251: Text intrinsic sizing semantics (min/max-content) (v1)
- ADR 0221: Text overflow (ellipsis) and line clamp (v1)
- ADR 0045 / ADR 0046: geometry queries and multiline semantics

## Current state (as of 2026-02-19)

Landings:

- Wrap strategy split:
  - `TextWrap::Word` avoids mid-token breaks.
  - `TextWrap::WordBreak` allows breaking long tokens when needed.
  - `TextWrap::Grapheme` breaks between grapheme clusters.
- Paint baseline alignment fixes to avoid double half-leading in fixed line boxes.

Known gaps:

- `min-content` for `TextWrap::Word` is not yet the true “longest token width”.
  Some UI layers still use a “placeholder width normalization” to avoid the worst regressions.
- Explicit multiline truncation (`line-clamp`) is not implemented (and must not be simulated via
  `wrap + ellipsis`).

## Definition of done

1) Text intrinsic sizing semantics match ADR 0251 across backends.
2) UI Gallery regressions covered by deterministic tests and/or diag scripts.
3) Ecosystem has clear helpers so component authors do not hand-roll wrap policy.

## Key invariants (regression targets)

- I1: `TextWrap::Word` does not produce per-character vertical wrapping under shrink-wrap.
- I2: `TextWrap::WordBreak` prevents long tokens (URLs/paths/identifiers) from forcing large
  min-widths in prose/markdown surfaces.
- I3: Measurement and paint agree on wrap width for any resolved definite width.
- I4: Geometry queries remain valid (caret/hit-test/selection rects) after wrap strategy changes.

## Work breakdown

See:

- `docs/workstreams/text-intrinsic-sizing-and-wrap-v1-milestones.md`
- `docs/workstreams/text-intrinsic-sizing-and-wrap-v1-todo.md`

## Evidence anchors

- Wrapper algorithms: `crates/fret-render-wgpu/src/text/wrapper.rs`
- Shaping/baseline: `crates/fret-render-wgpu/src/text/parley_shaper.rs`
- Intrinsic sizing integration: `crates/fret-ui/src/declarative/host_widget/measure.rs`
- Authoring helpers: `ecosystem/fret-ui-kit/src/declarative/text.rs`,
  `ecosystem/fret-ui-kit/src/ui_builder.rs`
- Diag repro (example): `tools/diag-scripts/ui-gallery-shadcn-tabs-demo-screenshot.json`

