# Workstream: Text Strut + Leading Distribution v1 (Multiline Stable Line Boxes)

Status: Active.

This document is **non-normative**. It is an engineering tracker for adding a mechanism-level
paragraph style that can produce **stable line boxes across multiple lines**, even when fallback
fonts or emoji participate in shaping.

Primary motivation:

- UI controls already prefer stable single-line layout (`TextLineHeightPolicy::FixedFromStyle`).
- Multiline surfaces (text areas, rich content blocks, markdown, terminals) still need a way to:
  - keep per-line metrics stable when desired (UI/form use cases),
  - but also opt into “expand-to-fit” correctness when clipping is unacceptable (content use cases).

## Background (why v1 is needed)

In practice, “first line becomes taller” is only one symptom. For multiline paragraphs, a single
fallback run in any line can cause:

- inconsistent per-line ascent/descent,
- visible baseline jitter between lines,
- and layout shifts when text changes or fonts load.

Fret currently expresses stability mostly via:

- `TextStyle.line_height` + `TextLineHeightPolicy::FixedFromStyle`, and
- `TextVerticalPlacement::BoundsAsLineBox` for fixed-height control centering.

That is a good v1 for **single-line** controls. For multiline paragraphs, we want an explicit
paragraph-level mechanism similar to Flutter’s `StrutStyle` + `forceStrutHeight`.

## Upstream reference (informative)

Flutter uses a paragraph-level “strut” to constrain line box metrics:

- `StrutStyle` (font family/size/height, leading distribution, leading override),
- `forceStrutHeight` to enforce the strut for all lines,
- `TextLeadingDistribution` for half-leading vs proportional leading.

Evidence (repo snapshot under `repo-ref/`):

- `repo-ref/flutter/engine/src/flutter/lib/web_ui/lib/src/engine/canvaskit/text.dart`
  (`toSkStrutStyleProperties`: `halfLeading`, `forceStrutHeight`, `heightMultiplier`).

## Proposed mechanism surface (draft)

Add a paragraph-level mechanism style that can be fed into shaping/layout:

- `TextStrutStyle`:
  - family (ui/mono or explicit `FontId`),
  - size,
  - line height (px or multiplier),
  - leading distribution:
    - `Even` (half-leading) vs `Proportional`,
  - `force` (enforce strut for every line).

Optional follow-up (not yet implemented):

- `TextHeightBehavior` (disable first ascent / last descent adjustments for tighter blocks).

Design constraints:

- This is **mechanism**, not policy: lives in `crates/fret-core` and is implemented by
  `crates/fret-render-text`.
- Ecosystem components choose defaults via `fret-ui-kit::typography` presets (ADR 0287).

## Scope (v1)

In scope:

- Mechanism types in `crates/fret-core` (no ecosystem policy baked in).
- Render-text implementation path (Parley) to enforce strut/leading distribution.
- At least one regression gate that exercises strut stability under emoji + fallback runs.

Out of scope (v1):

- Full text system v2 rewrite (tracked elsewhere).
- Hyphenation/justification.
- Per-script fallback policy tuning.

## Acceptance / gates (v1)

1. Multiline paragraph with mixed scripts + emoji shows stable per-line line boxes when strut is
   enabled (no line-to-line baseline jitter; stable measured height).
2. Content paragraphs can opt out (expand-to-fit) and avoid clipping when strut is disabled.
3. Ecosystem presets can expose this as an opt-in policy without leaking into `crates/fret-ui`.

## Tracking

- TODO tracker: `docs/workstreams/text-strut-and-leading-distribution-v1/text-strut-and-leading-distribution-v1-todo.md`
- Milestones: `docs/workstreams/text-strut-and-leading-distribution-v1/text-strut-and-leading-distribution-v1-milestones.md`
