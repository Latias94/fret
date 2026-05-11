# Public Surface Audit - 2026-05-12

Status: Baseline audit for the new follow-on lane

## Commands Run

```powershell
python tools/audit_crate.py --crate fret-code-editor
rg -n "pub (struct|enum|trait|type|fn)|pub use|pub mod" ecosystem/fret-code-editor/src ecosystem/fret-code-editor-buffer/src ecosystem/fret-code-editor-view/src
rg -n "0185|code editor ecosystem v1|fret-code-editor" docs/adr docs/workstreams -g "*.md"
```

## Assumptions

- Area: lane ownership
  - Assumption: this should be a narrow follow-on rather than more unchecked scope inside
    `code-editor-ecosystem-v1`.
  - Evidence: `code-editor-ecosystem-v1` already owns broad editor ecosystem rollout, Markdown
    downstream proof, display-map growth, IME, diagnostics, and perf attribution.
  - Confidence: Likely
  - Consequence if wrong: the follow-on may duplicate a slice that should be merged back into the
    old lane.

- Area: architectural baseline
  - Assumption: the buffer/view/surface split is directionally correct and should not be replaced.
  - Evidence: ADR 0185, `fret-code-editor-buffer`, `fret-code-editor-view`, `fret-code-editor`,
    and implementation alignment mark the ADR as aligned with known gaps.
  - Confidence: Confident
  - Consequence if wrong: public API work would preserve the wrong seams.

- Area: next priority
  - Assumption: public API and extension model work should precede large new feature additions.
  - Evidence: `fret-code-editor/src/lib.rs` exports a narrow facade while `CodeEditorHandle` owns
    many configuration/debug/cache methods; diagnostics/completion/hover/gutter are not yet stable
    first-class extension models.
  - Confidence: Likely
  - Consequence if wrong: this lane may over-invest in API inventory before feature pressure proves
    the desired shapes.

- Area: performance
  - Assumption: current performance is good enough to proceed with API work, as long as existing
    gates remain in force.
  - Evidence: `ui-perf-zed-smoothness-v1` tracks editor p50/p95/max and renderer payload contracts;
    recent evidence shows complex editor wheel top total around low milliseconds after row-scene
    replay and glyph pin-key fixes.
  - Confidence: Likely
  - Consequence if wrong: a hidden performance cliff could make API work target the wrong
    abstraction.

## Current Public Surface Snapshot

`ecosystem/fret-code-editor/src/lib.rs` exports only:

- `CodeEditor`
- `CodeEditorCacheStats`
- `CodeEditorHandle`
- `CodeEditorInteractionOptions`
- `CodeEditorPaintPerfFrame`
- `CodeEditorTorture`
- `PreeditState`
- `Selection`

This is intentionally narrow, but it also means most editor subsystem concepts are either private
implementation detail or exposed through `CodeEditorHandle` methods.

## Crate Audit Snapshot

`tools/audit_crate.py --crate fret-code-editor` reported:

- top files:
  - `src/editor/tests/mod.rs` - 4633 lines
  - `src/editor/mod.rs` - 3983 lines
  - `src/editor/paint/mod.rs` - 3952 lines
  - `src/editor/input/mod.rs` - 1074 lines
  - `src/editor/a11y/mod.rs` - 648 lines
  - `src/editor/geom/mod.rs` - 633 lines
- public surface quick scan:
  - `pub mod`: 0
  - `pub use`: 1
- direct workspace dependencies:
  - `fret-code-editor-buffer`
  - `fret-code-editor-view`
  - `fret-core`
  - `fret-runtime`
  - `fret-syntax`
  - `fret-ui`
  - `fret-ui-kit`
  - `fret-undo`

Interpretation: the crate has a clean facade at the root, but the integration module is still large
enough that ownership boundaries are easy to blur during feature work.

## Strong Existing Architecture

- Buffer contract exists: `TextBuffer`, `DocId`, `DocUri`, `Revision`, `Edit`, transaction and
  inverse edit support.
- View contract exists: `DisplayMap`, `DisplayPoint`, row materialization, folds, inlays, inline
  preedit, and code wrap policy.
- Surface contract exists: `CodeEditor`, windowed rows, input/IME, a11y projection, paint caches,
  and perf snapshots.
- ADR 0185 records the intended buffer/view/surface split.
- ADR 0188 records the direction for composed display fragments and unified mapping.
- Existing perf workstreams track p50/p95/max and renderer payload for editor stressors.

## Gaps

P0 gaps:

- No explicit stable/experimental/internal classification for current public items.
- `CodeEditorHandle` needs a method-by-method audit and grouping plan.
- Diagnostics, decorations, gutter markers, semantic tokens, hover, completion, and code actions do
  not yet have a first-class target interface state.
- Public editor docs do not yet say which layer owns feature data versus overlay/focus policy.

P1 gaps:

- Feature demos do not yet prove a realistic editor stack: diagnostics + gutter + hover/completion
  over the same buffer/view surface.
- Multi-cursor and rectangular selection remain future work.
- Command/keymap/undo grouping needs a public editor contract, not only internal behavior.
- Test ownership should be split so feature slices can land without fighting a monolithic test file.

P2 gaps:

- Performance gates are strong for current stressors, but future feature payloads need counters
  before thresholds are changed.
- Linux evidence is not currently available; keep platform-specific baseline labels explicit.

## Recommendation

Do not start with more micro-optimizations. The next landable slices should:

1. classify the current public surface,
2. audit and group `CodeEditorHandle` methods,
3. specify diagnostics/decorations/gutter as the first real extension contract,
4. add or reuse gates that prove those extension payloads do not break buffer revision, display-map
   mapping, a11y projection, or p50/p95/max performance contracts.
