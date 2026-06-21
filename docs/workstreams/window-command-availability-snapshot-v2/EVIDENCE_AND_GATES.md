# Evidence And Gates

Status: Active
Last updated: 2026-06-21

## Repro

Primary repro:

```bash
cargo run -p fretboard-dev --release -- diag perf \
  --repeat 7 \
  --warmup-frames 5 \
  --reuse-launch \
  --prelude-each-run \
  --prewarm-script tools/diag-scripts/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --dir target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516 \
  --max-pointer-move-dispatch-us 1500 \
  --max-pointer-move-hit-test-us 500 \
  tools/diag-scripts/ui-gallery/perf/ui-gallery-overlay-pointer-move-steady.json \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Latest gate evidence:

- `target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516/1778905245660/bundle.schema2.json`
- `target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516/stats.command_availability.top10.json`
- `target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516/focus_traversal_snapshot.by_frame.json`
- Perf gate output: max dispatch/hit-test 12us/2us for the script summary.
- `diag stats` derived pointer-move max dispatch/hit-test: 51us/9us.
- Structured hotspot count: only frame 858 contains `focus_traversal_snapshot`, with count 1.

Previous route-attribution evidence:

- `target/fret-diag/codex-command-availability-pointer-after-focus-route-20260516/1778902482724/bundle.schema2.json`
- `target/fret-diag/codex-command-availability-pointer-after-focus-route-20260516/stats.command_availability.top3.json`
- Top command availability frame before the cache had multiple `focus_traversal_snapshot` samples
  in one slow frame: 105us, 102us, 99us, 99us.

Historical baseline:

- `target/fret-diag/codex-command-availability-pointer-before-20260516/1778897176686-ui-gallery-overlay-pointer-move-steady/bundle.schema2.json`
- Observed gate result: pointer-move dispatch max 13us, hit-test max 2us.

## Stats Inspection

Human-readable attribution:

```bash
cargo run -p fretboard-dev --release -- diag stats \
  target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516/1778905245660/bundle.schema2.json \
  --sort command_availability \
  --top 10
```

Machine-readable attribution:

```bash
cargo run -p fretboard-dev --release -- diag stats \
  target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516/1778905245660/bundle.schema2.json \
  --sort command_availability \
  --top 10 \
  --json > target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516/stats.command_availability.top10.json
```

Focus traversal hotspot count:

```bash
jq '[.windows[]?.snapshots[]? | {frame_id, tick_id, count: ([.debug.command_availability_hotspots[]? | select(.route == "focus_traversal_snapshot")] | length), samples: [.debug.command_availability_hotspots[]? | select(.route == "focus_traversal_snapshot") | {command, route, elapsed_us, outcome}]} | select(.count > 0)]' \
  target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516/1778905245660/bundle.schema2.json \
  > target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516/focus_traversal_snapshot.by_frame.json
```

Latest command-availability attribution:

- Top command availability frame: 454us total, 439us eval.
- Top hotspot routes include one `focus.previous@focus_traversal_snapshot=99us` sample and
  focused/default fallback samples.
- The latest bundle has no frame with more than one `focus_traversal_snapshot` hotspot.
- No frame with command availability publication >=500us was missing hotspots.

Direct-entry no-focus subtree-interest rerun evidence:

- `target/fret-diag/inspector-direct-entry-no-focus-interest-cache-rerun/1781969841415/bundle.json`
- `target/fret-diag/inspector-direct-entry-no-focus-interest-cache-rerun/1781969841415/bundle.schema2.json`
- `diag stats --sort command_availability --top 10` now reports the hottest frame at
  `window_runtime_snapshot.command_availability(widget_count/collect_us/eval_us)=4/3/28`, with
  `edit.copy@focused_or_default=10-12us` and `action_route_fallback_roots=0-1us` samples.
- `jq '[.windows[]?.snapshots[]?.debug.command_availability_hotspots[]? | select(.route == "subtree_no_focus_fallback")] | length'`
  returns `0` for the rerun bundle, so the direct-entry no-focus subtree fallback hotspot class is
  no longer present on this probe.

Direct-entry stable-root focus rerun evidence:

- `target/fret-diag/inspector-direct-entry-stable-root-focus-20260621/1782019574150/bundle.schema2.json`
- `diag stats --sort command_availability --top 10` now keeps the hottest frames on
  `edit.copy@focused_or_default` / `action_route_fallback_roots`, with the stable-root click path
  no longer surfacing `subtree_no_focus_fallback` hotspots.
- `jq '[.windows[]?.snapshots[]?.debug.command_availability_hotspots[]? | select(.route == "subtree_no_focus_fallback")] | length'`
  returns `0` for the stable-root rerun bundle.

Expected JSON path:

- `/top/*/command_availability_hotspots/*/command`
- `/top/*/command_availability_hotspots/*/route`
- `/top/*/command_availability_hotspots/*/elapsed_us`
- `/top/*/command_availability_hotspots/*/start_node`
- `/top/*/command_availability_hotspots/*/resolved_node`
- `/top/*/command_availability_hotspots/*/start_element`
- `/top/*/command_availability_hotspots/*/resolved_element`
- `/top/*/sort`

## Unit And Package Gates

Focused stats projection:

```bash
cargo nextest run -p fret-diag bundle_stats_projects_command_availability_hotspots --no-fail-fast
```

Focused focus traversal cache coverage:

```bash
cargo nextest run -p fret-ui \
  action_availability_snapshot_reuses_focus_traversal_within_frame \
  action_availability_snapshot_refreshes_focus_traversal_on_next_frame \
  action_availability_snapshot_publishes_focus_traversal_gating \
  --no-fail-fast
```

Latest result: 3 passed.

Focused pointer-move narrow snapshot coverage:

```bash
cargo nextest run -p fret-ui \
  pointer_move_publishes_input_context_without_command_availability_recompute \
  --no-fail-fast
```

Latest result: 1 passed.

Focused action-route fallback root timing coverage:

```bash
cargo nextest run -p fret-ui window_command_action_availability_snapshot --no-fail-fast
```

Latest result: 14 passed. This includes
`action_availability_snapshot_uses_explicit_action_route_fallback_root`, which proves explicit
action-route fallback roots still publish availability with focus present and that the debug hotspot
start node is the same resolved fallback root used by availability traversal.

Focused no-focus subtree-interest reuse coverage:

```bash
cargo nextest run -p fret-ui \
  action_availability_no_focus_subtree_fallback_reuses_subtree_interest_across_commands \
  --no-fail-fast
```

Latest result: 1 passed.

Focused no-focus edit pruning coverage:

```bash
cargo nextest run -p fret-ui \
  action_availability_no_focus_subtree_fallback_skips_focus_bound_edit_commands \
  --no-fail-fast
```

Latest result: 1 passed.

Focused inspector direct-entry stable-root coverage:

```bash
cargo nextest run -p fret-ui-gallery --no-fail-fast \
  inspector_scroll_direct_entry_perf_script_starts_on_target_page_without_nav_search \
  inspector_scroll_perf_script_keeps_nav_transition_setup \
  gallery_inspector_torture_uses_fixed_row_text_roles \
  gallery_inspector_torture_stamps_row_root_semantics_and_action_state \
  gallery_inspector_torture_keeps_selected_row_model_on_paint_invalidation \
  gallery_inspector_torture_keeps_row_shell_shrunk \
  gallery_inspector_torture_keeps_tight_virtual_list_overscan \
  gallery_inspector_torture_wraps_the_retained_list_in_a_stable_root_semantics_host
```

Latest result: 8 passed.

Focused migrated parser and registry gates:

```bash
cargo nextest run -p fret-diag \
  registered_perf_key_inventory_doc_is_in_sync \
  unsupported_trace_resolve_and_index_flags_are_rejected_by_migrated_parser \
  bundle_stats_projects_command_availability_hotspots \
  --no-fail-fast
```

Changed packages:

```bash
cargo nextest run -p fret-ui -p fret-bootstrap -p fret-diag --no-fail-fast
```

Latest result: 1855 passed.

Formatting:

```bash
cargo fmt
```

Latest focused checks:

```bash
cargo nextest run -p fret-diag \
  registered_perf_key_inventory_doc_is_in_sync \
  unsupported_trace_resolve_and_index_flags_are_rejected_by_migrated_parser \
  bundle_stats_projects_command_availability_hotspots \
  --no-fail-fast
```

Latest result: 3 passed.

## Source Anchors

- `crates/fret-ui/src/tree/commands.rs`
- `crates/fret-ui/src/tree/tests/window_command_action_availability_snapshot.rs`
- `crates/fret-ui/src/tree/debug/commands.rs`
- `crates/fret-ui/src/tree/ui_tree_debug/record.rs`
- `crates/fret-ui/src/tree/ui_tree_debug/frame.rs`
- `crates/fret-ui/src/tree/ui_tree_debug/query.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_impl.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/layout_paint_hotspot_diagnostics.rs`
- `crates/fret-diag/src/stats/bundle_stats_snapshot.rs`
- `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`
- `crates/fret-diag/src/stats.rs`

## Boundary Notes

- `fret-ui` owns mechanism and snapshot publication.
- `fret-bootstrap` owns diagnostics projection.
- `fret-diag` owns bundle stats parsing and reporting.
- Component enablement policy stays out of this lane.
