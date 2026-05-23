# Editor Canvas Paint Replay Fast Path v1 Handoff

Date: 2026-05-24

## Current State

Active. This lane starts from the closed r64 row-setup attribution workstream. `ECPR-FP-010` and
`ECPR-FP-020` have passed local gates.

## Next Action

Commit the local mechanism, then run `ECPR-FP-030` target-machine editor-paint validation:

1. `python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r65-row-fast-path-baseline --keep-going`
2. `cargo build -p fretboard-dev -p fret-ui-gallery --release --features fret-ui-gallery/gallery-full`
3. `python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r65-row-fast-path-attrib --with-paint-perf --keep-going`
4. Run artifact verifier and closeout over those directories.

## Validation

Local gates passed on 2026-05-24:

- focused code-editor planned replay nextest set plus
  `retained_row_scene_origin_preserves_bounds_offset`
- `cargo check -p fret-code-editor --tests --features syntax-rust`
- `cargo fmt -p fret-code-editor --check`
- workstream JSON, parent JSON, catalog, and diff checks

## Cautions

- Do not alter renderer behavior or generic Canvas contracts.
- Do not change checked-in perf baselines in the local implementation commit.
- Do not reopen the closed row-setup diagnostics lane.
