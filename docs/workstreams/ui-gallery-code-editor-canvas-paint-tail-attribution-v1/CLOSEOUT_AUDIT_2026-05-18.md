# UI Gallery Code Editor Canvas Paint Tail Attribution Closeout Audit - 2026-05-18

Status: Closed
Last updated: 2026-05-18

## Final Verdict

This lane closed with a runtime mechanism fix in `fret-ui`.

The original `Canvas` tail was a downstream symptom of a wrong inner scroll viewport. The immediate
paint owner was code-editor row painting, but the root owner was positioned/pass-through wrapper
final child sizing under an outer scroll overflow probe.

Final owner:

- `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`

Runtime change:

- Non-absolute static/relative positioned-container children still probe-measure for intrinsic
  container size.
- Final child layout resolves `Fill` and `Fraction` axes from the wrapper base size.
- `Auto` and `Px` axes keep the measured child size.

Regression coverage:

- `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
  - `text_input_region_preserves_fill_scroll_viewport_for_tall_canvas_child`
  - `nested_page_scroll_preserves_inner_windowed_scroll_viewport_for_tall_canvas_child`

## Evidence

Owner proof:

- `CPT_030_CPT_040_OWNER_PROOF_2026-05-18.md`

Before bundle:

- `target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt030/1779092655064/bundle.schema2.json`
- Worst stats: `total=380102us`, `paint=376387us`
- Scroll evidence: `viewport_h=320064`, `content_h=320064`
- Row evidence: `rows_painted=20004`

After bundle:

- `target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt040/1779099328829/bundle.schema2.json`
- Worst stats: `total=1425us`, `paint=398us`
- p95 stats: `total=1425us`, `paint=418us`
- Scroll evidence: `viewport_h=518`, `content_h=320064`
- Row evidence: `rows_painted=289`

## Closeout Decision

Do not continue this lane into renderer, `ViewCache`, or code-editor row-surface work.

Recommended follow-on policy:

- Start a new narrow lane only if a fresh bounded-viewport bundle shows a remaining owner.
- Keep this lane as the attribution and mechanism-fix record for the wrong-viewport paint tail.

## Verification

Fresh local verification is recorded in `EVIDENCE_AND_GATES.md`.
