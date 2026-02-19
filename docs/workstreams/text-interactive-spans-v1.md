# Text Interactive Spans v1 (Links, Hit-Testing, Decorations)

Status: Draft
Owner: (unassigned)
Start: 2026-02-19

## Why this workstream exists

Markdown/document surfaces need:

- **clickable links** inside paragraphs,
- correct **wrap behavior** for long tokens (URLs/paths) inside a single paragraph line box, and
- consistent **decorations** (underline/strikethrough) across wrapped lines.

Today, `ecosystem/fret-markdown` falls back to a tokenized inline flow when links are interactive.
That preserves per-token interaction, but it breaks important text invariants:

- wrapping happens only *between* tokens (flex-wrap), not within a long link token,
- decorations are approximated at the element level (not per wrapped line),
- selection/hit-testing is per token rather than “one paragraph model”.

This is acceptable for early demos, but it is not a stable ecosystem foundation.

## Desired outcomes (v1)

1) A **single paragraph text model** can contain link spans (and other tagged spans) while still:
   - wrapping within long tokens when policy requests it (`TextWrap::WordBreak` / `Grapheme`),
   - producing stable caret/selection geometry (where applicable),
   - allowing per-span pointer hit-testing and activation.
2) Decorations (underline/strikethrough) are drawn correctly across wrapped lines.
3) Ecosystem authors do not need to re-implement link hit-testing or ad-hoc decoration overlays.

## Non-goals (v1)

- Full HTML layout or CSS inline formatting model.
- A generic rich-text editor surface (editing is a separate concern).
- Custom per-span layout (only paint + interaction tagging).

## Design options (to evaluate)

### Option A — Extend the existing text widget with span hit-testing

If we already have:

- layout-time shaping and line breaking,
- a hit-test function that maps local pointer coordinates to a byte/cluster index,

then we can add:

- a span table (byte ranges → tags),
- an activation callback that receives the resolved span tag.

This keeps rendering efficient (one paragraph blob) and keeps wrap semantics correct.

Open questions:

- How to expose the span tag to the declarative element tree without adding policy knobs to `fret-ui`.
- How to keep semantics/a11y stable (role/name for links) without leaking text in redacted diag mode.

### Option B — Introduce a dedicated `RichText` element

Add a new element (likely ecosystem-owned first) that:

- takes `AttributedText`,
- supports optional per-span tags (link ids / semantic ids),
- exposes `on_activate_span` and `hover_span` events,
- provides stable `test_id` hooks for diag scripts.

This reduces churn in `crates/fret-ui`, but risks duplicating lower-level text logic unless it
reuses the same text services and geometry mapping surfaces.

## Current state / short-term mitigation

- Markdown prose defaults to `TextWrap::WordBreak` for rich paragraphs to avoid long-token overflow.
- The remaining v1 gap is interactive links inside rich paragraphs without tokenization.

## Work breakdown

See: `docs/workstreams/text-interactive-spans-v1-todo.md`.

## Evidence anchors

- Markdown renderer (current tokenization fallback):
  - `ecosystem/fret-markdown/src/lib.rs`
- Text wrap semantics + intrinsic sizing:
  - `docs/adr/0251-text-intrinsic-sizing-min-max-content-v1.md`
  - `docs/workstreams/text-line-breaking-v1.md`
  - `docs/workstreams/text-intrinsic-sizing-and-wrap-v1.md`
- Text shaping/wrapping implementation:
  - `crates/fret-render-wgpu/src/text/parley_shaper.rs`
  - `crates/fret-render-wgpu/src/text/wrapper.rs`
