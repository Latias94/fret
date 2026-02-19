# Workstream: Text Infrastructure v1 (Audit + Gaps + Ownership)

Status: Active.

This document is **non-normative**. It is an *engineering tracker* intended to:

- enumerate the current text surfaces in Fret,
- clarify mechanism vs ecosystem ownership,
- list the most important correctness/perf/a11y gaps,
- point to existing ADRs and deeper workstreams.

If you are landing a change, prefer adding evidence anchors to the more specific trackers first.

## Source of truth (existing trackers)

- Intrinsic sizing + wrap semantics:
  - ADR: `docs/adr/0251-text-intrinsic-sizing-min-max-content-v1.md`
  - Workstream: `docs/workstreams/text-intrinsic-sizing-and-wrap-v1.md`
- Line breaking quality and conformance:
  - Workstream: `docs/workstreams/text-line-breaking-v1.md`
- Interactive spans (links, diagnostics-friendly activation):
  - Workstream: `docs/workstreams/text-interactive-spans-v1.md`

## Text surfaces (what exists today)

- `fret-ui` leaf elements:
  - `Text` (simple)
  - `StyledText` (rich runs, non-interactive)
  - `SelectableText` (selection + caret + interactive spans)
- Ecosystem policy layers:
  - `fret-ui-kit` text helpers (prose, break-words, nowrap, etc.)
  - `fret-markdown` (prose default wrap + rich links via spans)
  - `fret-code-view` / editor ecosystems (typically segment rows externally; render with `TextWrap::None`)

## Ownership map (mechanism vs policy)

Mechanism layer (stable contract, `crates/`):

- Hit testing (index ↔ position), selection geometry, caret affinity.
- Wrap modes and intrinsic sizing semantics:
  - `TextWrap::Word` should not mid-token break.
  - `TextWrap::WordBreak` is the explicit “break long tokens if needed” mode.
- Decorations as rendering primitives (underline/strikethrough).
- Interactive span activation routing (pointer + keyboard) as a mechanism-only capability.
- Semantics correctness: selection ranges and inline spans are defined over `SemanticsNode::value`
  and therefore require it to be present when those features are published.

Ecosystem/policy layer (`ecosystem/`):

- Default wrap policy per surface:
  - body copy / markdown prose defaults to `WordBreak`,
  - UI labels default to `Word` or `None` (with truncation),
  - editors own their row segmentation and typically use `None`.
- Visual affordances for interactive spans (link color, underline, hover states).
- Component-level decisions: padding, row heights, hover intent, focus trap/restore, etc.

## High-value gaps (short list)

### A11y and diagnostics for interactive spans

- `SelectableText` interactive spans do not currently surface per-span semantics nodes (e.g. role=link).
- v1 provides metadata-only span hints via ADR 0283 (`SemanticsNode::inline_spans`), but this is not
  yet mapped to platform accessibility navigation or span-level diagnostics selectors.
- Diagnostics selectors are currently element-level (`test_id`, role+name, node id); span-level targeting
  requires additional mechanism.

### Hover/pressed visual states for spans

- Baseline affordance (underline) exists for markdown rich links.
- Hover underline / pressed style needs a span hover state machine + theme policy.

### Line breaking policy for editor-class tokens

- For UI: `Word` vs `WordBreak` is explicit and fixture-gated.
- For editor surfaces: keep “wrap policy” in the ecosystem layer; render each display row with `None`.

### Internationalization + typography

Not in v1 scope, but candidates for future milestones:

- hyphenation (language-aware),
- justification,
- fine-grained punctuation rules beyond current staging gates,
- better bidi staging for complex mixed-direction paragraphs.

### Line box vertical alignment (baseline centering)

In GPU-first layout it is easy to accidentally center text by *glyph bounds* (what the renderer sees)
instead of by the *line box* (what UI authors expect from CSS-like systems). The visible failure mode
is “bottom-heavy” centering in fixed-height controls (tabs, buttons, pills).

When a `line_height` is configured, the intended baseline placement is the common “half-leading”
model used by CSS and GPUI:

- `padding_top = (line_height - ascent - descent) / 2`
- `baseline_y = padding_top + ascent`

Until we have a single mechanism-level knob that guarantees “line box centering” regardless of the
allocated height, ecosystem components may opt into the fixed line box behavior by setting both:

- `line_height_px(line_height)`, and
- `h_px(line_height)`

This forces the text widget to paint inside a stable line box and avoids per-component “nudge” hacks.

Next step (mechanism, v1):

- Add an opt-in “bounds-as-line-box” vertical placement mode for single-line text, so fixed-height
  controls can set the text element height to `Fill` and still get GPUI/CSS-like half-leading
  baseline placement without forcing `h_px(line_height)` everywhere.

## Guidance: choosing a wrap mode

- Use `TextWrap::Word` for UI labels/buttons/tabs (prevents mid-token breaks).
- Use `TextWrap::WordBreak` for prose surfaces that may contain long URLs/paths/identifiers (prevents overflow).
- Use `TextWrap::Grapheme` only when you want the most aggressive emergency breaking.
- For code editors: prefer external row segmentation + `TextWrap::None` per display row.

Evidence anchors:

- `ecosystem/fret-ui-kit/src/declarative/text.rs` (`text_prose_*` helpers)
- `ecosystem/fret-ui-kit/src/ui_builder.rs` (`UiBuilder::break_words`)
- `ecosystem/fret-markdown/src/lib.rs` (prose default `WordBreak`)

## Reference: GPUI (Zed) (informative)

GPUI (as used by Zed) historically used a pragmatic wrapper:

- heuristic word-character classification (`is_word_char`),
- prefer breaks at word boundaries,
- emergency break inside a long token when required.

GPUI’s fixed line box baseline placement uses a “half-leading” formula:

- `padding_top = (line_height - ascent - descent) / 2`
- `baseline_y = padding_top + ascent`

Evidence anchors (repo snapshots under `repo-ref/`):

- `repo-ref/zed/crates/gpui/src/text_system/line_wrapper.rs`
- `repo-ref/zed/crates/gpui/src/text_system/line_layout.rs`
- `repo-ref/zed/crates/gpui/src/text_system/line.rs`

In Fret v1 we prefer UAX#14 / Parley break opportunities for `Word`, and we keep explicit modes for
mid-token breaking (`WordBreak` / `Grapheme`) to avoid embedding policy into mechanism code.
