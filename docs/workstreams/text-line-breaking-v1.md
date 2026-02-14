# Workstream: Text Line Breaking v1 (Wrap Quality + Editor-Grade Rules)

Status: Draft (design); implementation planned as an incremental replacement of wrapper heuristics.

This document is **non-normative**. It complements:

- `docs/workstreams/text-system-v2-parley.md` (text system v2 tracker)
- `docs/workstreams/text-layout-integration-v1.md` (historical integration hazards + invariants)

## Problem Statement

Fret currently performs wrapping by:

1) shaping a slice as a single line, then
2) choosing “cut points” using heuristic rules, then
3) re-shaping per line (or shape-once and slice clusters/glyphs for LTR-only paths).

Evidence: `crates/fret-render-wgpu/src/text/wrapper.rs`.

This approach is fast to ship and offers direct control over caret/selection semantics, but it
produces editor-visible issues:

- line breaks at surprising points for code identifiers, file paths, and URLs,
- weak CJK punctuation handling (breaks before/after forbidden characters),
- inconsistent “emergency break” behavior for long tokens,
- limited explainability and limited conformance coverage.

## Goals

1) Improve wrap quality for:
   - general UI labels,
   - editor-class text (code, paths, CJK, emoji sequences).
2) Keep existing correctness invariants:
   - measurement/paint agree on wrap inputs,
   - soft-wrap trailing whitespace remains selectable (editor-grade UX baseline).
3) Make line-breaking behavior auditable via tests and diagnostics.
4) Preserve or improve performance under resize jitter and long paragraphs.

## Non-goals (v1)

- Full CSS Text Level 4 conformance.
- A full bidi line-breaking model across mixed-direction runs in v1 (we will stage this).
- A platform-native line breaker dependency (keep the stack Rust-first).

## Current Implementation Snapshot (Evidence)

Key files:

- Wrapper: `crates/fret-render-wgpu/src/text/wrapper.rs`
  - word wrap uses `is_word_char` heuristics:
    - `crates/fret-render-wgpu/src/text/wrapper.rs` (`is_word_char`, `is_cjk_char`)
  - grapheme wrap uses `unicode_segmentation` boundaries.
- Invariants already gated by tests:
  - trailing whitespace at soft wrap is selectable:
    - `crates/fret-render-wgpu/src/text/mod.rs:6294`

## Design Options

### Option A — Use Parley’s line breaking

Parley style supports:

- `WordBreak` (strength),
- `OverflowWrap` (emergency behavior),
- `TextWrapMode` (wrap vs no-wrap).

Approach:

- Replace (or supplement) the wrapper with a Parley-driven “shape paragraph with wrap width”
  operation that returns multiple lines.
- Preserve current `TextWrap` modes by mapping:
  - `TextWrap::None` → `TextWrapMode::NoWrap`
  - `TextWrap::Word` → `TextWrapMode::Wrap` + a chosen `WordBreak` strength
  - `TextWrap::Grapheme` → `OverflowWrap::Anywhere` (or `BreakWord`) + `Wrap`
- Keep newline splitting (`\n`) as an outer paragraph boundary (current code already does this).

Pros:

- Better break opportunities out of the box.
- Less duplicated logic for line breaking.
- Likely reduces re-shaping churn by letting Parley compute line breaks in one pass.

Cons / risks:

- Must re-validate caret/selection mapping and affinity semantics.
- Must preserve editor-grade behavior like “selectable trailing whitespace” at a soft wrap boundary.
- Must stage RTL/mixed-direction behavior carefully.

### Option B — Keep the wrapper, but replace the cut-point heuristic with Unicode break opportunities

Approach:

- Keep the “shape once then cut” strategy, but compute candidate breakpoints using a Unicode
  line-breaking algorithm (UAX#14) rather than `is_word_char`.
- Keep a separate editor policy for “token-aware” breaks (paths/URLs/code).

Pros:

- Smaller behavioral delta; easier to preserve existing geometry semantics.
- Allows a staged rollout with targeted conformance fixtures.

Cons:

- Still duplicates “line breaking” logic outside Parley.
- Mixed-direction behavior remains complex.

## Recommendation (v1)

Stage the work as:

1) M0: Expand conformance tests and make failures visible (no behavior change yet).
2) M1: Improve the wrapper cut-point selection (Option B) for better results immediately.
3) M2: Introduce Parley-driven line breaking behind a controlled switch and migrate (Option A),
   keeping the wrapper as a compatibility layer for cases not yet covered (RTL staging).

This sequencing keeps risk bounded while still converging on a simpler, more correct architecture.

## Conformance & Regression Gates

Add a dedicated set of wrap conformance strings that cover:

- CJK punctuation:
  - forbid breaks at certain leading/trailing punctuation positions,
  - avoid orphaned closing punctuation at the start of a line where possible.
- Code identifiers:
  - `snake_case`, `camelCase`, `SCREAMING_SNAKE_CASE`
  - digits in identifiers (`foo2bar`)
- Paths/URLs:
  - `C:\foo\bar\baz`
  - `/usr/local/bin`
  - `https://example.com/a/b?c=d#e`
- Emoji sequences:
  - ZWJ family emoji
  - VS16 presentation selectors
- Long tokens:
  - single 1k-character “word” should wrap in `Grapheme` mode.

## Milestones (High-Level)

- M0: Conformance suite + invariants
  - add wrap fixtures + expected breakpoints,
  - keep trailing-space-selectable invariant locked.
- M1: Wrapper heuristic upgrade (Option B)
  - use Unicode break opportunities for word wrap candidates,
  - keep `TextWrap::Grapheme` as emergency fallback.
- M2: Parley line-breaking integration (Option A)
  - shape paragraphs with wrap width and consume Parley-produced lines,
  - preserve caret/selection semantics and mapping tests.
- M3: RTL + mixed-script staging
  - keep correctness first; introduce additional mapping when needed.

For detailed milestone checklists and task breakdown:

- `docs/workstreams/text-line-breaking-v1-milestones.md`
- `docs/workstreams/text-line-breaking-v1-todo.md`

