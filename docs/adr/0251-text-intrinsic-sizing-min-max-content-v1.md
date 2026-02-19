# ADR 0251: Text Intrinsic Sizing (min/max-content) Semantics (v1)

Status: Proposed
Date: 2026-02-19

## Context

Fret’s layout engine (Taffy) relies on intrinsic sizing probes (`min-content`, `max-content`) to
resolve shrink-wrapped and auto-sized layouts.

For text, intrinsic sizing interacts with wrapping in ways that are easy to get wrong:

- If `min-content` is treated as “near zero” for word-wrapped text, common UI labels can become
  pathologically narrow (e.g. per-character vertical wrapping) when parents shrink-wrap based on
  that intrinsic measurement.
- If `min-content` is treated as “unconstrained” for all wrap modes, long tokens (URLs, paths,
  identifiers) can force large min-widths and break layouts that expect long content to wrap.

This is a contract-level concern because it affects:

- layout stability (no surprise shrink-wrap outcomes),
- component recipe authoring (shadcn/Tailwind parity),
- measurement vs paint agreement (avoid overlap from height underestimation).

Related ADRs:

- ADR 0045: Text geometry queries (hit-testing, caret metrics)
- ADR 0046: Multiline text layout and geometry queries
- ADR 0059: Text overflow (ellipsis) and truncation
- ADR 0221: Text overflow (ellipsis) and line clamp (v1)

## Goals

1) Define stable, deterministic intrinsic sizing semantics for text.
2) Prevent “vertical text” failures for common UI labels under shrink-wrap layouts.
3) Preserve explicit author control for long-token behavior via wrap modes.
4) Keep semantics consistent across backends (native + wasm).

## Non-goals (v1)

- Locale-specific line-breaking and hyphenation policies.
- Implicit multiline truncation (`line-clamp`) via `wrap + ellipsis` (forbidden by ADR 0221).

## Decision

### D1 — Define intrinsic width outputs as a function of `TextWrap`

For a given `TextInput` + `TextStyle` (and shaping-affecting span attributes), the text system
defines:

- `max_content_width`: the width of the text if laid out on a single visual line (no soft wrap).
- `min_content_width`: the smallest width the text can take without creating additional overflow
  beyond what the wrap mode explicitly allows.

Semantics by wrap mode:

- `TextWrap::None`
  - `min_content_width = max_content_width`
  - Rationale: no soft wrap is permitted; callers use overflow policy (`clip`/`ellipsis`) at the
    constraint level.

- `TextWrap::Word`
  - `max_content_width = single-line width`
  - `min_content_width = width(longest unbreakable token)`
  - Token definition (v1): split on Unicode whitespace and explicit newlines; punctuation policy is
    backend-defined but must be deterministic and documented. (See Open Questions.)
  - Rationale: word-wrapped text must not report near-zero min-content, or shrink-wrapped parents
    will measure a pathological width and cause per-character wrapping.

- `TextWrap::WordBreak`
  - `max_content_width = single-line width`
  - `min_content_width ~= 0` (or the minimum representable cluster width)
  - Rationale: `WordBreak` explicitly allows breaking long tokens; it is the escape hatch for long
    URLs/paths/identifiers in prose surfaces.

- `TextWrap::Grapheme`
  - `max_content_width = single-line width`
  - `min_content_width ~= 0` (or the minimum representable grapheme cluster width)
  - Rationale: intended for editor-like surfaces where mid-token wrapping is required.

### D2 — Intrinsic sizing must not depend on caller “placeholder widths”

During intrinsic probes, the layout engine may pass placeholder widths (including zero) to mean
“unknown”.

The text system must treat intrinsic sizing as an explicit mode and compute intrinsic widths
without relying on those placeholder widths.

### D3 — Measurement and paint must agree

When a layout is resolved with a definite width, the wrap width used for measurement MUST match the
wrap width used for preparing the paintable blob, so that parent heights reserve sufficient space.

## Implementation Notes (non-normative)

Recommended architecture:

- Keep intrinsic measurement logic in the renderer text wrapper layer (not in UI widgets), so all
  surfaces (labels, selectable text, text inputs, editor views) share the same semantics.
- Cache `max_content_width` and `min_content_width` alongside other shaping metrics; treat them as
  shaping-affecting (style/spans) but independent of paint-only attributes.

## Open Questions

1) Tokenization policy for `TextWrap::Word`:
   - Should we adopt a GPUI-like “word character” set for candidates (e.g. treating `_`/`-`/`.` as
     token characters), or stick to a whitespace-only split for v1?
   - How should CJK punctuation participate?
   - Should code/editor surfaces eventually get a distinct policy (e.g. “code token wrap”) rather
     than reusing `Word` vs `WordBreak`?

2) Cross-backend consistency:
   - Ensure wasm and native produce the same intrinsic widths for the same inputs.

## References

- Zed GPUI wrap candidates and token character policy:
  - `repo-ref/zed/crates/gpui/src/text_system/line_wrapper.rs` (`LineWrapper::wrap_line`, `LineWrapper::is_word_char`)
  - `repo-ref/zed/crates/gpui/src/text_system/line_layout.rs` (`LineLayout::compute_wrap_boundaries`)

## Consequences

- Component recipes can rely on stable shrink-wrap behavior without per-page hacks.
- Long-token behavior becomes explicit:
  - keep `Word` for typical UI/prose,
  - opt into `WordBreak`/`Grapheme` when long tokens must wrap.
