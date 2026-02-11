# ADR 0146: TextInput v2 (Attributed Spans) - Core Contract & RichText Deprecation

- Status: Proposed
- Date: 2026-01-13

## Context

Fret currently exposes rich inline styling via `RichText` + `TextRun` in `fret-core` (`crates/fret-core/src/text.rs`).
This v1 surface works for basic Markdown/syntax highlighting, but it couples shaping/layout caches to paint attributes
(notably run colors), which forces reshaping/re-wrapping on theme-only updates and makes future text features harder to
evolve.

ADR 0142 locks the *direction* for the renderer text system v2:

- Parley is the shaping engine (line/chunk shaping as the primitive),
- wrapping/ellipsis is wrapper-owned (not backend-owned),
- attributed spans split shaping-affecting vs paint-only attributes.

What is still missing is a precise, forward-compatible **core contract** for attributed text that:

1. Replaces `RichText`/`TextRun` with a span model that is compatible with caching, wrapping, and geometry queries.
2. Leaves room for future extensions (font features/variations, language/script hints, decorations, background, etc.)
   without forcing repeated breaking changes.

This ADR refines the `TextInput`/`TextSpan` sketch in ADR 0142 into an explicit `fret-core` API and migration plan.

## Decision

### 1) Introduce `TextInput` + `TextSpan` in `fret-core`

`fret-core` will add the following core types (names and field meanings locked by this ADR).

Notes:

- These are **data model** types and do not imply where shaping happens (UI must not shape; ADR 0006 / ADR 0066).
- They are marked `#[non_exhaustive]` to allow future additions without repeated breaking changes.

```rust
#[non_exhaustive]
pub enum TextInput {
    Plain {
        text: Arc<str>,
        style: TextStyle,
    },
    Attributed {
        text: Arc<str>,
        base: TextStyle,
        spans: Arc<[TextSpan]>,
    },
}

#[non_exhaustive]
pub struct TextSpan {
    /// Span length in UTF-8 bytes.
    pub len: usize,
    pub shaping: TextShapingStyle,
    pub paint: TextPaintStyle,
}

#[non_exhaustive]
pub struct TextShapingStyle {
    pub font: Option<FontId>,
    pub weight: Option<FontWeight>,
    pub slant: Option<TextSlant>,
    pub letter_spacing_em: Option<f32>,
}

#[non_exhaustive]
pub struct TextPaintStyle {
    pub fg: Option<Color>,
}
```

Constraints (must be validated at the stable boundary, see §2):

- `Attributed` spans must cover the entire string: `sum(spans.len) == text.len()`.
- Span boundaries must be valid UTF-8 char boundaries.
- For v2, `size` and `line_height` remain uniform per layout (`TextStyle` / `base` governs them).

Non-goals for the initial v2 core surface (these are expected to be added later via `#[non_exhaustive]`):

- text decorations (underline/strikethrough styles and colors),
- background runs (inline `bg`),
- per-span font features/variations and language/script hints.

### 2) Move `TextService` to a single entry point taking `TextInput`

`TextService` evolves to:

- `prepare(input: &TextInput, constraints: TextConstraints) -> (TextBlobId, TextMetrics)`
- `measure(input: &TextInput, constraints: TextConstraints) -> TextMetrics`

The existing `prepare(&str, &TextStyle, ...)` shape becomes a convenience wrapper (or is removed once migrations land).

Span invariants must be handled deterministically at this boundary:

- Debug builds should `debug_assert!` on invalid spans.
- Release builds must clamp/normalize (drop empty spans, clamp to char boundaries, and/or fall back to `Plain`).
- The strategy must be consistent across backends and platforms (avoid “it panics only on macOS” drift).

### 3) Deprecate `RichText` + `TextRun`

`RichText`/`TextRun` are replaced by the v2 span model and become deprecated once the ecosystem migrations land.

Transitional compatibility is allowed (for an agreed window), but the stable surface area goal remains:

- `fret-ui`, `fret-markdown`, and `fret-code-view` should converge on emitting `TextInput::Attributed` instead of
  `RichText`.

## Rationale

Why spans (instead of v1 runs)?

- **Correct caching**: shaping/layout keys are derived only from shaping-affecting attributes and constraints; paint-only
  changes (theme recolor, selection tint, link underline color) should not invalidate shaping.
- **Wrapper-owned policy**: spans survive wrapping/truncation deterministically and can be mapped back to byte indices for
  caret/hit-testing (ADR 0045 / ADR 0046).
- **Future extensibility**: v2 spans can grow to cover decorations/background and later shaping hints without rewriting
  every call site repeatedly.

Why `#[non_exhaustive]`?

- Text is a “hard-to-change” surface area. We want to add features such as font features/variations, language/script
  hints, and per-span fallback policy without forcing repeated ecosystem-wide breaking changes.

## Migration Plan (high level)

1. `fret-core`:
   - Add the v2 types and update `TextService` to accept `TextInput`.
   - Keep v1 helpers temporarily (wrappers) to reduce churn while migrating call sites.
2. `fret-ui`:
   - Update `TextProps` / `StyledTextProps` / `SelectableTextProps` to carry `TextInput` (or build it deterministically).
3. Ecosystem:
   - `fret-markdown`: map inline styling + decorations to spans (removing fallback hacks).
   - `fret-code-view`: represent highlighting as spans and keep wrapping correct under long lines.
4. Renderer:
   - Implement span-aware paint assignment at draw time (shaping cache keys must not include paint-only fields).

Progress tracking for the work is maintained in `docs/workstreams/text-system-v2-parley.md`.

## Alternatives Considered

### A) Keep `RichText` and extend `TextRun`

Pros:

- Minimal churn now.

Cons:

- Still couples caches to paint unless we add an additional split model anyway.
- Becomes harder to add decorations/background consistently and align with geometry queries.

### B) Use a fully generic “attributes map” per span

Pros:

- Extremely flexible.

Cons:

- Harder to validate, harder to keep deterministic, harder to make efficient without many allocations.
- Type-safety loss at the framework boundary.

## Consequences

- Call sites will migrate from `RichText` to `TextInput::Attributed`.
- Renderer text caching will become more correct (theme-only changes stop forcing shaping work).
- We retain freedom to evolve shaping/paint surfaces as v2 grows (due to `#[non_exhaustive]`).

## Open Questions (deferred)

- Variable fonts: how to represent variations/features in a stable, serializable way (likely `#[non_exhaustive]` fields
  on `TextShapingStyle` with careful cache-keying).
- Decoration layout: underline/strikethrough placement and thickness rules (align with ADR 0102).
- Per-script fallback policy and language/script hints (ADR 0029 follow-up).
