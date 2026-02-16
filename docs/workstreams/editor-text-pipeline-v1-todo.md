# Editor Text Pipeline v1 — TODO


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Scope: `docs/workstreams/editor-text-pipeline-v1.md`

## M0 — Document boundary + invariants

- [x] Identify and document the current call chain from editor paint → renderer `TextSystem`.
- [x] List invariants to preserve:
  - [x] byte/utf16 mapping rules,
  - [x] cursor/selection geometry alignment,
  - [x] wrap stability under resize jitter.

## M1 — Row text caching

- [x] Add a row text cache (visible rows as `Arc<str>`).
- [x] Key the cache by:
  - [x] buffer revision,
  - [x] display row index,
  - [x] wrap cols / width bucket (best-effort),
  - [x] fold/inlay epochs (to keep decorated display rows stable).
- [x] Add a regression test that guards against whole-buffer `to_string()` on paint.
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`paint_source_does_not_materialize_whole_buffer_string`)

## M2 — Syntax spans per row

- [~] Define a row-local span model (ecosystem-owned):
  - [x] span ranges are **UTF-8 byte offsets** relative to the row `Arc<str>`,
  - [x] ranges are char-boundary aligned (`is_char_boundary`) and grapheme-safe for emoji clusters (best-effort clamps on mapping),
  - [x] spans are non-overlapping and cover the row text (gaps filled with default style).
  - Evidence:
    - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`materialize_row_rich_text`)
    - `ecosystem/fret-code-editor-view/src/row_spans.rs`
- [~] Produce row-local spans from tree-sitter highlighting events:
  - [x] map base-buffer row ranges → composed display-row ranges (folds/inlays/preedit),
  - [x] clamp/normalize out-of-date ranges after edits (best-effort, deterministic),
  - [x] split spans at row boundaries (no cross-row spans) (row clip in paint).
  - Evidence:
    - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (row clip + `row_spans` mapping)
    - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`normalize_syntax_spans_for_text`)
- [x] Introduce a stable “highlight class → paint style” mapping:
  - [x] keep it paint-only (`TextPaintStyle`), do not set shaping-affecting fields for syntax.
  - Evidence:
    - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`syntax_color`)
- [x] Pass `AttributedText` into the renderer:
  - [x] per row, call the renderer's attributed-text preparation with `TextWrap::None` (editor-owned wrapping),
  - [~] avoid per-span reshaping on paint-only changes (relies on renderer shape-key gates; add editor-level gate).
  - Evidence:
    - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`rich_text_with_blob`)
- [x] Regression gates:
  - [x] “paint-only span changes do not reshape”:
    - [x] assert `AttributedText::shaping_eq` holds across theme-only updates.
      - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`paint_only_syntax_color_changes_do_not_affect_rich_text_shaping_eq`)
    - [x] add a `TextSystem` gate that shape-cache hits remain stable across paint-only edits for the
      same text + shaping style.
      - `crates/fret-render-wgpu/src/text/mod.rs` (`paint_only_changes_miss_blob_cache_but_hit_shape_cache`)
  - [x] span range correctness:
    - [x] emoji ZWJ/VS16 sequences never split.
      - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`normalize_syntax_spans_does_not_split_zwj_or_vs16_graphemes`)
    - [x] mixed scripts + surrogate-pair UTF-16 ranges remain deterministic at platform boundaries.
      - `crates/fret-ui/src/declarative/tests/semantics.rs`
        (`declarative_text_input_region_utf16_queries_are_deterministic_for_mixed_scripts_and_surrogates`)
- [x] Add a test that theme-only changes do not affect shaping keys.
  - `crates/fret-render-wgpu/src/text/mod.rs` (`multispan_paint_changes_do_not_affect_shape_key`)
 - [~] Add an editor-focused test suite (ecosystem) that:
   - [x] validates row-local mapping under fold/inlay/preedit composition,
   - [x] verifies paint-only changes do not invalidate row shaping.
   - Evidence:
     - `ecosystem/fret-code-editor-view/src/row_spans.rs` (end-to-end mapping tests)
     - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`paint_only_syntax_color_changes_do_not_affect_rich_text_shaping_eq`)

## M3 — Wrap policy separation

- [x] Land a deterministic policy core (presets + knobs + grapheme-safe emergency breaks), with
  unit tests.
  - Evidence: `ecosystem/fret-code-editor-view/src/code_wrap_policy.rs`
- [~] Wire the policy into display-row segmentation for wrapped editor surfaces.
  - Status: mostly (v1 applies the policy to buffer text segments for both undecorated and
    decorated lines; fold placeholders / inlays / preedit remain atomic tokens and are not split by
    the policy).
  - Evidence:
    - `ecosystem/fret-code-editor-view/src/lib.rs` (`compute_wrapped_row_start_cols`)
    - `ecosystem/fret-code-editor-view/src/lib.rs`
      (`display_map_code_wrap_policy_with_inlays_keeps_inlay_atomic_and_prefers_identifier_breaks`)
    - `ecosystem/fret-code-editor-view/src/lib.rs`
      (`display_map_wrapped_rows_do_not_split_fold_placeholders`)
- [~] Define a “code wrap policy” surface at the ecosystem layer (editor-owned row segmentation).
  - Status: exposed as a runtime knob via `CodeEditorHandle::set_code_wrap_policy`, but not yet
    wired into higher-level configuration/builder surfaces.
  - Evidence: `ecosystem/fret-code-editor/src/editor/mod.rs` (`CodeEditorHandle::set_code_wrap_policy`)
  - [x] Provide presets (recommended):
    - [x] `Conservative` (mostly whitespace / punctuation),
    - [x] `Balanced` (adds identifier/path/url-friendly boundaries),
    - [x] `Aggressive` (more emergency breaks; still deterministic).
  - [x] Provide a small set of common knobs (do not overfit; keep it auditable), e.g.:
    - [x] allow breaks after path/url separators (`/`, `\\`, `?`, `&`, `#`),
    - [x] allow breaks around punctuation runs (`.`, `,`, `:`, `;`),
    - [x] allow breaks at identifier boundaries (snake `_`, camelCase transitions),
    - [x] allow breaks around common operator tokens (`::`, `->`, `.`, `=`) (optional).
- [x] Ensure the policy matches cursor movement and selection semantics (no drift between row
  segmentation and rendered geometry).
  - Evidence:
    - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
      (`move_caret_vertical_clamps_in_display_row_space_when_wrapped`)
    - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
      (`move_caret_vertical_steps_through_code_wrap_policy_rows`)
    - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
      (`move_caret_vertical_uses_row_fold_map_for_inlay_insertions_under_soft_wrap`)
    - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
      (`move_caret_vertical_uses_row_fold_map_for_fold_placeholders_under_soft_wrap`)
    - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
      (`home_end_move_within_wrapped_display_rows`)
    - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
      (`shift_vertical_extends_selection_in_display_row_space_when_wrapped`)
    - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
      (`shift_home_end_extends_selection_within_wrapped_display_rows`)
    - `ecosystem/fret-code-editor-view/src/lib.rs`
      (`display_map_code_wrap_policy_does_not_split_arrow_operator_token`)
    - `ecosystem/fret-code-editor-view/src/lib.rs`
      (`byte_to_display_point_respects_code_wrap_policy_rows`)
    - `ecosystem/fret-code-editor-view/src/lib.rs`
      (`inline_preedit_replacement_roundtrips_under_wrap_and_code_wrap_policy`)
- [~] Add fixture-driven conformance tests for the policy (ecosystem-owned, deterministic):
  - Status: initial JSON suite landed; expand coverage as new edge-cases are found.
  - Evidence:
    - `ecosystem/fret-code-editor-view/tests/code_wrap_policy_fixtures.rs`
    - `ecosystem/fret-code-editor-view/tests/fixtures/code_wrap_policy_v1.json`
  - [x] identifiers (snake/camel/digits),
  - [x] paths/URLs,
  - [x] punctuation (`.`) and operator tokens (`::`, `->`) avoid obviously-bad splits,
  - [x] grapheme clusters (ZWJ/VS16/combining marks) do not split inside clusters,
  - [x] long tokens have a bounded emergency-break behavior.
- [ ] Coordinate with `docs/workstreams/text-line-breaking-v1.md`:
  - UI wrap improvements must not change editor wrap policy implicitly.
  - Evidence anchors:
    - `ecosystem/fret-code-editor-view/src/code_wrap_policy.rs` (`row_starts_for_code_wrap`)
    - `ecosystem/fret-code-editor-view/src/lib.rs` (`DisplayMap::new_with_code_wrap_policy`)

## M4 — Platform text input interop (UTF-16 over composed view)

- [x] `TextInputRegion` answers `PlatformTextInputQuery` from `a11y_value` deterministically:
  - surrogate pairs (e.g. 😀) clamp correctly,
  - selection/composition map from UTF-8 (ADR 0071) to UTF-16 (platform bridge).
  - Evidence: `crates/fret-ui/src/declarative/tests/semantics.rs`
- [x] Wire `TextInputRegionProps.ime_cursor_area` from the editor caret geometry (data-only):
  - Evidence: `ecosystem/fret-code-editor/src/editor/mod.rs`
- [x] Add ecosystem-owned bounds/hit-test support for `TextInputRegion` (not `fret-ui` mechanism):
  - `BoundsForRange` / `CharacterIndexForPoint` via cached row geometry + fallbacks.
- [x] Add ecosystem-owned replace support for platform text input (not `fret-ui` mechanism):
  - `replace_text_in_range_utf16` via window mapping + buffer ops.
  - `replace_and_mark_text_in_range_utf16` supports composing for:
    - empty `range` (caret-only), and
    - non-empty `range` (selection replacement represented in the composed view; composing text remains preedit-only).
- [x] (Staging) Unify selection-replacing preedit across paint + platform-facing composed view:
  - the display-row text used for shaping/paint matches the platform-facing composed window while
    `preedit_replace_range` is active.
  - Evidence:
    - `ecosystem/fret-code-editor-view/src/lib.rs` (`InlinePreedit.replace_range`,
      `materialize_display_row_text_replaces_range_with_preedit_unwrapped`)
    - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
      (`platform_replace_and_mark_non_empty_range_replaces_in_composed_view_without_mutating_base`)
- [x] Decide cancel semantics for selection-replacing composition (restore selection + keep base buffer unchanged) and lock it behind tests.
  - Reference (Zed / GPUI): the platform bridge forwards composition end via `unmarkText()` on macOS,
    which calls the view's `unmark_text`; Zed's editor clears its composition highlight and drops the
    IME transaction on `unmark_text`.
    - `repo-ref/zed/crates/gpui/src/platform/mac/window.rs` (`unmark_text`)
    - `repo-ref/zed/crates/editor/src/editor.rs` (`unmark_text`, `replace_and_mark_text_in_range`)
  - Hypothesis to validate: "cancel" often arrives as a `setMarkedText` with an empty string and a
    replacement range that deletes the current marked range, followed by `unmarkText`.
  - Evidence:
    - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
      (`platform_replace_and_mark_empty_text_cancels_and_restores_selection`)
    - `tools/diag-scripts/ui-gallery-code-editor-a11y-composition-baseline.json` + `crates/fret-diag/src/stats.rs`
      (`check_bundle_for_ui_gallery_code_editor_a11y_composition_json`)
- [x] Observe `TextFontStackKey` and invalidate editor-local geometry caches so platform queries never use stale row geometry after font changes.

## M5 — Row geometry cache boundary (future fearless refactor)

Goal: make editor geometry queries (caret/hit-test/IME bounds) **cache-stable and auditable** under:
folds/inlays/preedit, resize jitter, font stack changes, and theme changes.

- [~] Define the stable cache key for row geometry:
  - [~] text revision + display window mapping epoch (handled by cache invalidation on
    `(revision, wrap_cols, folds_epoch, inlays_epoch)` and by pointer-identity row keys),
  - [x] shaping-relevant style key (font/axes/features/letter spacing) via `RowGeomKey`,
  - [x] constraints key (wrap/overflow/max width bucket, scale factor) via `RowGeomKey` (64px ceil bucket for wrap=None + align=Start),
  - [x] `TextFontStackKey` revision via `RowGeomKey` + cache clears on font stack change.
  - Evidence:
    - `ecosystem/fret-code-editor/src/editor/geom/mod.rs` (`RowGeomKey`)
    - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (geometry cache hit uses `RowGeomKey`)
- [x] Ensure paint-only changes never invalidate row geometry caches.
  - Regression gate:
    - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`row_geom_key_ignores_paint_only_changes`)
- [ ] Add a catastrophic regression guard for “resize jitter + visible viewport”:
  - reuse the UI wrap smoke gate as a baseline,
  - [x] add a code-editor-specific smoke gate (no calibrated baseline required):
    - `tools/perf/diag_code_editor_resize_jitter_smoke_gate.py`
    - default script: `tools/diag-scripts/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`
