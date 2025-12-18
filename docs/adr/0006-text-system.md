# ADR 0006: Text System Boundary (TextBlob + Metrics)

Status: Accepted

## Context

Fret targets editor-grade UI, which requires high quality text for:

- inspector/property panels (short labels, values),
- logs and diagnostics,
- eventually a code editor (large documents, selections, IME).

Text is a frequent source of architectural rewrites if the boundary between UI/layout and shaping/atlas/rendering is unclear.

## Decision

Introduce a **text system boundary** based on two products:

1. **Metrics** (for layout): sizes, baselines, line breaks.
2. **`TextBlobId`** (for paint): an opaque handle to a shaped, render-ready representation.

The UI layer (`fret-ui`) must not perform shaping, atlas allocation, or GPU uploads. It can only:

- request measurement and blob creation,
- store `TextBlobId` as cached paint data,
- emit `SceneOp::Text { text: TextBlobId, ... }`.

The renderer side (`fret-render`) owns the implementation details:

- shaping backend (eventually `cosmic-text`),
- glyph atlas and uploads,
- caching keyed by text/style/constraints,
- resource lifetime for `TextBlobId`.

## API Shape (Contract)

Core types live in `fret-core::text` and are backend-agnostic.

- `TextStyle`: font, size, and other style attributes.
- `TextConstraints`: wrapping and maximum width.
- `TextMetrics`: measurement results required for layout.

Creation is explicit:

- `prepare(text, style, constraints) -> (TextBlobId, TextMetrics)`
- `release(TextBlobId)` (best-effort)

## Consequences

- Property-panel text can start with a minimal implementation and be upgraded to `cosmic-text` later without changing `UiTree` or `SceneOp`.
- The code editor becomes “just another consumer” of the same text contract, with more sophisticated caching.
- `fret-core` stays wgpu-free; all GPU specifics live in `fret-render`.

## Notes / Future Work

- IME: represent composition events as data in the platform input layer and keep it separate from shaping.
- Resource lifetime: `TextBlobId` may be refcounted/interned; `release` can be delayed and drained via the app/runner loop.

