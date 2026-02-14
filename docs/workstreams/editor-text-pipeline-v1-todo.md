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
- [ ] Add a test that theme-only changes do not affect shaping keys.

## M3 — Wrap policy separation

- [ ] Define “code wrap policy” at the ecosystem layer.
- [ ] Ensure the policy matches cursor movement and selection semantics.
- [ ] Coordinate with `docs/workstreams/text-line-breaking-v1.md` so UI wrap improves without
  breaking editor expectations.
