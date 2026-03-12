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
- Related workstreams (non-normative):
  - `docs/workstreams/text-line-breaking-v1/text-line-breaking-v1.md` (wrap quality + conformance fixtures)
  - `docs/workstreams/text-layout-integration-v1/text-layout-integration-v1.md` (measurement/paint agreement hazards)

## Current state (as of 2026-02-19)

Landings:

- Wrap strategy split:
  - `TextWrap::Word` avoids mid-token breaks.
  - `TextWrap::WordBreak` allows breaking long tokens when needed.
  - `TextWrap::Grapheme` breaks between grapheme clusters.
- Paint baseline alignment fixes to avoid double half-leading in fixed line boxes.
- `TextWrap::Word` now participates in `min-content` intrinsic sizing by using a near-zero wrap width,
  which yields “longest unbreakable segment” semantics when mid-token breaks are disabled.

Known gaps:

- Tokenization rules for “unbreakable segments” under `TextWrap::Word` are not fully locked yet
  (whitespace vs punctuation vs identifier-like candidates). We need cross-backend determinism.
- UI-level placeholder-width normalization (`available.width == 0` as “unknown”) still exists and
  should be audited for correctness and performance, but it is distinct from min/max-content probes.
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

## Authoring guidance (ecosystem)

- Prose/UI copy (default): use `TextWrap::Word` (UI kit: `text_prose`).
- Prose with long tokens (URLs/paths): use `TextWrap::WordBreak` (UI kit: `text_prose_break_words` or builder `.break_words()`).
- Editor/code-like surfaces: use `TextWrap::Grapheme` when mid-token wrapping is explicitly desired (UI kit: `text_code_wrap`).
- Markdown/document surfaces: default to `TextWrap::WordBreak` for paragraphs/headings to avoid long-token overflow
  (e.g. `ecosystem/fret-markdown`).

## Layout constraints (required for predictable wrap)

Wrap policy is only meaningful when the text widget is measured with a **definite** width. In practice:

- Ensure some ancestor resolves a width constraint (`w_full`, `max_w`, explicit px width, etc.).
- In flex layouts, ensure the *flex item that contains text* opts into shrinking below min-content:
  - set `min_w_0()` on the text container (the equivalent of CSS `min-width: 0`),
  - otherwise long tokens can force overflow or cause surprising intrinsic sizing outcomes.

If your page layout uses a shrink-wrapped column (`w_fit`) around prose/markdown, prefer switching it
to `w_full().min_w_0()` and constraining it via an outer max-width instead.

## API quick reference (ecosystem)

Declarative helpers:

- `fret_ui_kit::text_nowrap` / `text_truncate` (Tailwind `whitespace-nowrap` / `truncate` intent)
- `fret_ui_kit::text_prose` / `text_prose_break_words`
- `fret_ui_kit::text_code_wrap`

Builder helpers:

- `fret_ui_kit::UiBuilder<...>::nowrap()` / `.break_words()`
- `fret_ui_kit::UiBuilder<...>::wrap(TextWrap::...)` for explicit policy

## Work breakdown

See:

- `docs/workstreams/text-intrinsic-sizing-and-wrap-v1/text-intrinsic-sizing-and-wrap-v1-milestones.md`
- `docs/workstreams/text-intrinsic-sizing-and-wrap-v1/text-intrinsic-sizing-and-wrap-v1-todo.md`

## Evidence anchors

- Wrapper algorithms: `crates/fret-render-text/src/wrapper.rs`
- Shaping/baseline: `crates/fret-render-text/src/parley_shaper.rs`
- Intrinsic sizing integration: `crates/fret-ui/src/declarative/host_widget/measure.rs`
- Authoring helpers: `ecosystem/fret-ui-kit/src/declarative/text.rs`,
  `ecosystem/fret-ui-kit/src/ui_builder.rs`
- Diag repro (examples):
  - `tools/diag-scripts/ui-gallery-tabs-wrap-and-baseline-screenshots.json`
  - `tools/diag-scripts/ui-gallery-text-measure-overlay-wrap-modes-screenshots.json`
