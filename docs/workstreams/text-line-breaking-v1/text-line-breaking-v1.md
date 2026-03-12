# Workstream: Text Line Breaking v1 (Wrap Quality + Editor-Grade Rules)

Status: M0 implemented (fixture-driven conformance harness); M1 implemented (Unicode break opportunities); M2 implemented (Parley paragraph line breaking for `TextWrap::Word`); M3 implemented (wrapped RTL/mixed-direction staging gates).

This document is **non-normative**. It complements:

- `docs/workstreams/standalone/text-system-v2-parley.md` (text system v2 tracker)
- `docs/workstreams/text-layout-integration-v1/text-layout-integration-v1.md` (historical integration hazards + invariants)

## Problem Statement

Historically, Fret performed wrapping by:

1) shaping a slice as a single line, then
2) choosing “cut points” using heuristic rules, then
3) re-shaping per line (or shape-once and slice clusters/glyphs for LTR-only paths).

That approach shipped quickly and enabled early editor-grade caret/selection semantics, but it
produced editor-visible issues (CJK punctuation, identifiers, URLs) and was hard to audit.

In v1, `TextWrap::Word` migrates to **Parley paragraph line breaking** and locks behavior behind
fixtures + invariants.

Evidence: `crates/fret-render-text/src/wrapper.rs`, `crates/fret-render-wgpu/src/text/mod.rs`,
`docs/workstreams/text-line-breaking-v1/text-line-breaking-v1-*.md`.

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

- Wrapper: `crates/fret-render-text/src/wrapper.rs`
  - `TextWrap::Word` uses Parley paragraph line breaking (wrap width drives line breaks).
  - `TextWrap::WordBreak` uses Parley paragraph line breaking with an explicit "break long tokens if needed"
    emergency policy (intended for prose surfaces that may contain URLs/paths/identifiers).
  - `TextWrap::Grapheme` uses `unicode_segmentation` grapheme cluster boundaries as the emergency
    break surface.
  - newline splitting (`\n`) is an outer paragraph boundary.
 - Invariants already gated by tests:
  - trailing whitespace at soft wrap is selectable:
    - `crates/fret-render-wgpu/src/text/mod.rs:6294`
  - wrapped RTL/mixed-direction staging gates:
    - `crates/fret-render-wgpu/src/text/mod.rs` (`rtl_word_wrap_hit_test_maps_line_edges_to_logical_ends`)
    - `crates/fret-render-wgpu/src/text/mod.rs` (`mixed_direction_word_wrap_selection_rects_for_rtl_range_are_nonempty`)
- Conformance harness:
  - `crates/fret-render-text/src/wrapper.rs` (`text_wrap_conformance_v1_fixtures`)
  - `crates/fret-render-wgpu/src/text/tests/fixtures/text_wrap_conformance_v1.json`

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

1) M0: Expand conformance tests and make failures visible (baseline harness is landed).
2) M1: Improve the wrapper cut-point selection (Option B) for better results immediately (initial
   Unicode break opportunities are landed; keep iterating with fixtures + perf gates).
3) M2: Replace the legacy wrapper with Parley-driven paragraph line breaking (Option A). Do not
   retain a long-lived compatibility wrapper: rely on conformance + invariants to keep the behavior
   auditable, and keep code wrap policy in the ecosystem/editor layer.
   - Editor wrap is *not* renderer-owned: editor surfaces segment display rows themselves and
     typically render each display row with `TextWrap::None`. UI wrap improvements must not change
     editor wrapping implicitly.
   - Evidence anchors:
     - `ecosystem/fret-code-editor-view/src/code_wrap_policy.rs` (`row_starts_for_code_wrap`)
     - `ecosystem/fret-code-editor-view/src/lib.rs` (`DisplayMap::new_with_code_wrap_policy`)

This sequencing keeps risk bounded while still converging on a simpler, more correct architecture.

## Reference: GPUI (Zed) line wrapping approach (informative)

GPUI (as used by Zed) ships a lightweight wrapper that:

- defines a custom `is_word_char` character set (including many punctuation characters),
- prefers wrapping at “word starts” (space → non-space transitions), but
- falls back to breaking at the current character when a single token exceeds the wrap width.

Evidence anchors (repo snapshots under `repo-ref/`):

- `repo-ref/zed/crates/gpui/src/text_system/line_wrapper.rs` (wrap candidate selection + `is_word_char`)
- `repo-ref/zed/crates/gpui/src/text_system/line_layout.rs` (wrap boundary computation over shaped glyphs)

This approach is pragmatic for editor-first UIs, but it is inherently policy-heavy and requires
maintaining a hand-curated character set. In Fret v1, we prefer using Parley/UAX#14 break
opportunities for `TextWrap::Word`, and we use explicit modes (`WordBreak` / `Grapheme`) when
authors want emergency mid-token breaks.

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

- `docs/workstreams/text-line-breaking-v1/text-line-breaking-v1-milestones.md`
- `docs/workstreams/text-line-breaking-v1/text-line-breaking-v1-todo.md`
