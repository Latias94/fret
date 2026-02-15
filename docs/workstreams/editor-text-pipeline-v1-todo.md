# Editor Text Pipeline v1 — TODO

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

- [ ] Produce row-local spans from tree-sitter highlighting events.
- [ ] Pass `AttributedText` into the renderer (avoid per-span reshaping on paint-only changes).
- [x] Add a test that theme-only changes do not affect shaping keys.
  - `crates/fret-render-wgpu/src/text/mod.rs` (`multispan_paint_changes_do_not_affect_shape_key`)

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
- [ ] Ensure the policy matches cursor movement and selection semantics (no drift between row
  segmentation and rendered geometry).
- [~] Add fixture-driven conformance tests for the policy (ecosystem-owned, deterministic):
  - Status: initial JSON suite landed; expand coverage as new edge-cases are found.
  - Evidence:
    - `ecosystem/fret-code-editor-view/tests/code_wrap_policy_fixtures.rs`
    - `ecosystem/fret-code-editor-view/tests/fixtures/code_wrap_policy_v1.json`
  - [~] identifiers (snake/camel/digits),
  - [~] paths/URLs,
  - [x] emoji sequences (ZWJ/VS16) do not split inside clusters,
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
- [ ] (Future) Decide cancel semantics for selection-replacing composition (restore vs keep deletion) and lock it behind tests/diagnostics.
- [x] Observe `TextFontStackKey` and invalidate editor-local geometry caches so platform queries never use stale row geometry after font changes.
