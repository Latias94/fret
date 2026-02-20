# Text Layout Alignment v1 (Parley-first) — Fearless Refactor Charter

Status: Active (2026-02-20)

This workstream is a **fearless refactor** focused on aligning Fret’s text **layout and geometry**
capabilities around a single source of truth: the renderer-owned Parley shaping + wrapping pipeline.

Normative contracts remain ADR-driven (not this document). Primary references:

- Text boundary: `docs/adr/0006-text-system.md`
- v2 Parley text system roadmap: `docs/workstreams/text-system-v2-parley.md`

## Why this workstream exists

Editor-grade UI lives and dies by text correctness. In practice, correctness is not only about
“glyphs render”, but about the **geometry queries** and their **coordinate mapping**:

- caret positioning and affinity
- hit-testing (point -> caret)
- selection rectangles (including mixed-direction text)
- span-aware paint overlays (backgrounds, underlines) that line up with the final placement

When these pieces drift across layers (UI vs renderer, or different coordinate spaces), we end up
with “works in common cases, breaks in fixed-height / centered text / IME / bidi”.

## Scope (v1)

In-scope (mechanism-level):

- Make Parley-derived layout geometry the single source of truth for:
  - `caret_rect`, `hit_test_point`, `selection_rects(_clipped)`
  - span overlay geometry (e.g. `TextPaintStyle.bg` rendered as quads)
- Fix coordinate-space drift in UI consumers:
  - ensure span overlays / selection highlights / interactive-span hit regions include the same
    vertical placement offset as the text draw op
- Improve mixed-direction geometry fidelity:
  - allow selection rectangles to be multi-segment per line when necessary (bidi)

Non-goals (for v1):

- Changing the public `TextService` contract surface.
- Per-span font size / per-span line-height (still uniform per prepared layout).
- Replacing Parley or adding an alternative shaping backend.

## Current state snapshot (2026-02-20)

Implementation anchors:

- UI runtime text widgets (IME/editing): `crates/fret-ui/src/text/`
- Declarative painting for text elements:
  - `crates/fret-ui/src/declarative/host_widget/paint.rs`
- Renderer Parley pipeline + geometry queries:
  - `crates/fret-render-wgpu/src/text/mod.rs`

Known gaps / risks to close (high level):

- UI layers can accidentally use text-local geometry without applying vertical placement offsets,
  causing overlays and hit-testing to drift in fixed-height/centered text.
- Selection geometry is currently optimized for common cases; bidi correctness must be validated
  against real fixtures and hardened with unit tests.

## Strategy (fearless, but shippable)

We ship this as a sequence of small, evidence-backed milestones:

- Each milestone lands with a focused test gate.
- Keep the contract stable; refactor internals freely.
- Prefer “make the wrong thing impossible” (shared helpers and centralized mapping code) over
  scattered fixes.

See:

- TODO tracker: `docs/workstreams/text-parley-layout-alignment-v1-todo.md`
- Milestones: `docs/workstreams/text-parley-layout-alignment-v1-milestones.md`

## Validation gates (minimum)

- `cargo fmt`
- `cargo nextest run -p fret-ui`
- `cargo nextest run -p fret-render-wgpu`

Optional (when a behavior is user-visible):

- Add a `fretboard diag` scripted repro capturing:
  - bidi selection/caret
  - span backgrounds
  - fixed-height/centered text hit-testing

