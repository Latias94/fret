# Text Parley Unification v1 — Fearless Refactor Workstream

Status: Active (implementation-driven tracker; ADRs remain the source of truth)

Tracking files:

- `docs/workstreams/text-parley-unification-v1-todo.md`
- `docs/workstreams/text-parley-unification-v1-milestones.md`

This workstream is a focused execution slice of the broader text-system direction:

- Normative contracts:
  - `docs/adr/0006-text-system.md`
  - `docs/adr/0142-text-system-v2-parley-attributed-spans-and-quality-baseline.md`
- Related living roadmap:
  - `docs/workstreams/text-system-v2-parley.md`

## 0) Why this workstream exists

We want a single, coherent text pipeline with **Parley** as the shaping engine, such that:

- UI widgets (`TextInput`, `TextArea`, selectable text) get editor-grade geometry queries:
  - caret rects, selection rects, hit-testing, first-line metrics
- Text metrics are stable and sane for edge cases:
  - empty string, IME preedit, trailing whitespace, soft-wrap boundaries
- The layering remains intact:
  - `crates/fret-ui` stays mechanism-only and does not depend on renderer/shaping crates
  - shaping and rasterization remain renderer-owned (`crates/fret-render-*`)

This is a “fearless refactor” workstream: ship in small steps, add gates early, and delete hacks
only after a stable contract exists.

## 1) Invariants (do not break)

1. **Mechanism vs policy split**
   - `crates/fret-ui` stays mechanism/contract-level (no Radix/shadcn interaction policy).
2. **Crate layering**
   - No `wgpu`/`winit`/`web-sys` leaks into `fret-core`/`fret-runtime`/`fret-ui`.
3. **Determinism**
   - Cache keys must be explicit (font stack revision, shaping-affecting attributes, quality knobs).
4. **Gates before deletion**
   - Add a unit test or `fretboard diag` script before removing a fallback/hack.
5. **Small landable steps**
   - Prefer narrowly scoped PR-sized changes with measurable outcomes.

## 2) Scope (v1)

Primary outcomes:

- Unify the shaping path on Parley and ensure all geometry queries are Parley-backed.
- Ensure `TextMetrics`, `FirstLineMetrics`, caret/selection rects are **non-degenerate** for:
  - empty strings
  - IME preedit / composition ranges
  - selection ranges that align to soft-wrap boundaries
- Align coordinate mapping between:
  - text “content space” (glyph positions, baseline, line metrics)
  - widget “box space” (padding, vertical placement, clipping)

Non-goals (v1):

- A complete code-editor virtualization layer (separate workstream).
- A full component policy layer for text interactions (ecosystem-owned).

## 3) Evidence anchors (current implementation touchpoints)

Renderer (Parley shaping + geometry):

- `crates/fret-render-wgpu/src/text/parley_shaper.rs`
- `crates/fret-render-wgpu/src/text/wrapper.rs`
- `crates/fret-render-wgpu/src/text/mod.rs` (unit tests)

UI integration (vertical placement + selection/caret mapping):

- `crates/fret-ui/src/text/coords.rs`
- `crates/fret-ui/src/text/input/widget.rs`
- `crates/fret-ui/src/text/area/widget.rs`

## 4) Proposed execution plan (high level)

1. **Close renderer-side degeneracy**
   - Make empty/preedit/selection geometry non-degenerate at the renderer boundary where possible.
2. **Unify UI coordinate mapping**
   - Prefer a single shared mapping helper for vertical placement/baseline alignment.
3. **Add focused gates**
   - Unit tests for geometry edge cases + small diag scripts for interactive IME and caret drift.
4. **Delete UI fallbacks**
   - Remove widget-local “measure a space” or “inflate zero-height rect” hacks once renderer
     guarantees are stable and tested.
5. **Cross-reference GPUI/Zed behavior**
   - Use `repo-ref/zed` as a non-normative reference for editor-grade caret/selection behavior.

