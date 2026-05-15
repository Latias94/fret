# M2 Follow-Up: Syntax Replay Key Content Equality

Date: 2026-05-15

## Summary

`RowSceneSyntaxReplayKey` no longer requires pointer identity for the row line, display row spans,
or syntax spans. It still accepts pointer identity as the fast case, but equivalent content now
matches too. This removes prepaint replay-plan skips where paint could later recover from an
equivalent syntax replay key.

## Evidence

Focused gates:

```bash
cargo fmt --check
cargo check -p fret-code-editor --features syntax-rust --all-targets
cargo nextest run -p fret-code-editor syntax_replay_key_matches_equivalent_current_inputs --features syntax-rust --no-fail-fast
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan --features syntax-rust --no-fail-fast
cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast
```

Result on 2026-05-15: passed.

Perf repro:

```bash
target/release/fretboard-dev diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --sort time \
  --top 15 \
  --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --dir target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-syntax-key-content-eq-20260515 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Worst bundle:

- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-syntax-key-content-eq-20260515/1778835965902/bundle.schema2.json`

Aggregate p95 from the perf run:

- total: `1270us`
- paint: `729us`
- prepaint: `255us`
- layout: `354us`

Worst-bundle `code_editor_paint_perf.p95` after this follow-up:

- `us_total`: `191us`
- `us_row_content_resolve`: `116us`
- `us_row_scene_prepaint_plan`: `75us`
- `rows_scene_prepaint_skip_key_mismatch`: `0`
- `rows_scene_prepaint_skip_no_cache`: `1`
- `rows_scene_fast_miss_no_entry`: `1`
- `rows_scene_full_miss_no_entry`: `1`
- `rows_scene_stored_at_visible_end`: `1`
- `rows_scene_stored_at_visible_start`: `0`

## Interpretation

This is a useful candidate-cost reduction: key mismatch no longer explains the replay-plan tail, and
the code-editor paint subtotal improves materially in the worst bundle. It does not complete the lane
goal. The remaining full miss is still a no-cache row at the newly exposed visible end, so the next
slice must prebuild or stage that edge row before its visible paint.

Do not revive the rejected paint-time offscreen seeding approach: it only moved work into paint,
kept one no-entry full miss per worst frame, and worsened aggregate p95.
