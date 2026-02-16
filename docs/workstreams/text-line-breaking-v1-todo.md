# Text Line Breaking v1 — TODO

Scope: `docs/workstreams/text-line-breaking-v1.md`

## M0 — Conformance suite

- [x] Add a fixture-driven wrap conformance harness (baseline goldens):
  - `crates/fret-render-wgpu/src/text/wrapper.rs` (`text_wrap_conformance_v1_fixtures`)
  - `crates/fret-render-wgpu/src/text/tests/fixtures/text_wrap_conformance_v1.json`
- [x] Expand fixture coverage and expectations (initial set):
  - [x] CJK punctuation (leading/trailing forbiddens),
  - [x] identifiers (snake/camel),
  - [x] paths/URLs,
  - [x] emoji ZWJ/VS16,
  - [x] long-token emergency wraps.
- [~] Add a dedicated “wrap invariants” module (optional):
  - Status: invariants are currently gated in `crates/fret-render-wgpu/src/text/mod.rs`; extract
    into a dedicated module if the list grows further.
  - [x] trailing whitespace at soft wrap is selectable:
    - `crates/fret-render-wgpu/src/text/mod.rs` (`trailing_space_at_soft_wrap_is_selectable`)
    - `crates/fret-render-wgpu/src/text/mod.rs` (`trailing_whitespace_run_at_soft_wrap_is_selectable`)

## M1 — Wrapper heuristic upgrade

- [x] Replace `is_word_char`-based candidate selection in:
  - `crates/fret-render-wgpu/src/text/wrapper.rs`
  - with Unicode line break opportunities (UAX#14) via `swash::text::analyze` (keep a small heuristic fallback for now).
- [x] Keep the current behavior for:
  - [x] newline splitting,
  - [x] ellipsis truncation.
- [x] Add a micro perf regression guard for long paragraphs / resize jitter.

## M2 — Parley line breaking integration

- [x] Replace the legacy wrapper with Parley-driven paragraph line breaking:
  - [x] Add a shaper entry point: “shape paragraph with wrap width” returning:
    - multiple lines,
    - cluster/glyph mapping sufficient for geometry queries.
  - [x] Wire it through the renderer wrapper for `TextWrap::Word` (keep newline splitting as an
    outer paragraph boundary).
- [x] Regression gates (must hold under the Parley path):
  - [x] measurement/paint agree on wrap inputs (no layout drift under intrinsic sizing),
  - [x] “soft wrap trailing whitespace is selectable” invariant remains true,
  - [x] caret/hit-test/selection rects remain deterministic across wrap boundaries.
- [x] Conformance coverage:
  - [x] run the fixture-driven wrap conformance suite under the Parley path,
  - [x] document any known gaps as explicit TODOs (avoid silent behavior drift).
- [~] Performance guard:
  - [~] ensure no O(n²) regressions on long paragraphs (Parley path is linear; keep a dedicated
    long-paragraph probe in the wrapper test suite),
  - [x] keep resize jitter bounded:
    - add a diag perf script focused on `TextWrap::Word` under window resize jitter:
      `tools/diag-scripts/ui-gallery-text-measure-overlay-window-resize-drag-jitter-steady.json`
    - catastrophic regression smoke gate:
      `tools/perf/diag_text_wrap_resize_jitter_smoke_gate.py`
- [x] Cleanup:
  - [x] delete the legacy wrapper implementation once the Parley path passes the gates above (no
    compatibility branch retained).

## Customization seam (recommended)

- [x] Define and document the supported customization approach:
  - [x] general UI uses `TextWrap` + future narrow knobs (if needed),
  - [x] editor/code surfaces should own row segmentation and thus “wrap policy” at the ecosystem
    layer (do not push code wrap heuristics into the renderer).

## Open questions

- [x] Which Unicode line-breaking implementation should we standardize on for M1 (if not Parley)?
  - Answer: Parley paragraph line breaking is the baseline for UI `TextWrap::Word`.
- [x] Do we want a dedicated “code wrap mode” distinct from UI `TextWrap::Word`?
  - Answer: yes, but it lives in the ecosystem/editor layer via display-row segmentation
    (`CodeWrapPolicy`), not in the renderer wrapper.
