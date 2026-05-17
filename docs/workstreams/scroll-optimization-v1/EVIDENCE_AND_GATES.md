# Scroll Optimization Workstream (v1) — Evidence And Gates

Date: 2026-05-16
Status: Active

## Candidate perf slice — Resize-jitter ScrollArea layout root attribution

Seed evidence after the code-editor retained row-fragment prototype:

- Baseline local resize-jitter stats:
  `target/fret-diag/local-next-editor-paint-20260516-retained-row-fragment-resize-jitter-r3/worst.stats.json`
- Profiled local resize-jitter stats:
  `target/fret-diag/local-next-scroll-layout-resize-jitter-20260516-r1/worst.stats.json`
- Profiled triage:
  `target/fret-diag/local-next-scroll-layout-resize-jitter-20260516-r1/triage.json`

Profiled command shape:

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json \
  --repeat 1 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_A11Y_DISABLE=1 \
  --env FRET_UI_GALLERY_BOOTSTRAP_FONTS=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --env FRET_SCROLL_LAYOUT_PROFILE=1 \
  --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=0 \
  --env FRET_SCROLL_LAYOUT_PROFILE_MIN_MEASURE_US=0 \
  --env FRET_LAYOUT_NODE_PROFILE=1 \
  --env FRET_LAYOUT_NODE_PROFILE_TOP=20 \
  --env FRET_LAYOUT_NODE_PROFILE_MIN_US=100 \
  --env FRET_DIAG_MAX_SNAPSHOTS=180 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --sort time \
  --top 15 \
  --json \
  --dir target/fret-diag/local-next-scroll-layout-resize-jitter-20260516-r1 \
  --launch -- cargo run -p fret-ui-gallery --release \
    --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness
```

Result:

- Worst frame: `total=1362us`, `layout=805us`, `layout_roots=574us`,
  `layout_request_build_roots=157us`, `layout_engine_solve=373us`, `prepaint=160us`,
  `paint=397us`.
- View-cache: one content root, `reuse_reason=needs_rerender`, `layout_dependency=contained_when_bounds_known`,
  `layout_outcome=contained_clean`.
- Top layout hotspot: gallery content `ScrollArea` / `Scroll`, `267us` exclusive and `413us`
  inclusive.
- Scroll profile for `ui-gallery-content-viewport`: `total=405us`, `solve_barrier=244us`,
  `layout_children_first_pass=156us`, `measure_children=0us`.
- Dirty state on that profile: `interactive_resize=true`, `direct_children_layout_invalidated=false`,
  `descendant_subtree_layout_dirty=false`, `post_layout_extents_mode=true`.

Decision:

- Do not spend the next slice on renderer text, glyph atlas, or row-fragment replay from this evidence.
- Do not start a broad scroll layout skip: `Scroll` still writes authoritative viewport/content
  state during layout.
- The next narrow question is whether changing-bounds `solve_barrier` and the small child layout pass
  under a `needs_rerender` content view-cache root can use a cheaper resize apply path without
  weakening scroll extent correctness.

Follow-up local no-4090 attribution (2026-05-16):

- Baseline same-command resize-jitter sample:
  `target/fret-diag/local-next-no4090-resize-jitter-20260516-r3/worst.stats.json`
- Paint-only RAF experiment sample:
  `target/fret-diag/local-next-no4090-resize-jitter-paint-only-raf-20260516-r3/worst.stats.json`
- Focused gates:
  - `cargo nextest run -p fret-ui canvas_paint_only_animation_frame_keeps_view_cache_root_reusable --no-fail-fast`
  - `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`
  - `cargo fmt --check`
- Result:
  - `CodeEditor` torture autoscroll now requests paint-only RAF from its paint hook.
  - The resize-jitter worst-bundle p95 moved total/layout/solve/prepaint/paint from
    `1494/903/408/221/632us` to `1479/865/386/183/613us`.
  - The row replay guardrails stayed stable: `rows_scene_replayed=289`, `rows_scene_stored=0`,
    and row-scene replay hit rate `100%`.
  - The view-cache root still reports `reuse_reason=needs_rerender`, but the dirty source moved
    from `notify/animation_frame_request` to `other/scroll_handle_window_update`.
- Decision:
  - Keep the paint-only RAF cleanup as a baseline-neutral correction.
  - Do not classify this as a shadcn `ScrollArea` recipe issue.
  - The remaining owner is the windowed-paint scroll update contract: avoid parent cache-root
    rerender only when a retained/windowed surface can prove the current visible row window is
    already covered by retained row fragments.

Post-merge retained/windowed scroll update proof (2026-05-17):

- Evidence:
  `target/fret-diag/local-next-no4090-windowed-scroll-paint-only-post-merge-20260517-r3/worst.stats.json`
- Worst bundle:
  `target/fret-diag/local-next-no4090-windowed-scroll-paint-only-post-merge-20260517-r3/1778948069413/bundle.schema2.json`
- Command shape:
  - `target/release/fretboard-dev diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`
  - repeat `3`, warmup `5`, standard prewarm/prelude hooks, overlay disabled, view-cache shell enabled,
    code-editor paint perf enabled, scroll/layout profiling enabled.
  - Launch command: `cargo run -p fret-ui-gallery --release --features gallery-full`.
- Focused gates:
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
  - `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`
  - `cargo nextest run -p fret-ui-kit windowed_rows_frame_row_rects_iterates_visible_rows --no-fail-fast`
  - `cargo fmt --check`
- Result:
  - repeat=3 p95 total/layout/solve/prepaint/paint: `1697/1048/346/260/408us`.
  - Worst-bundle p95 layout roots / layout solve / paint widget: `812/346/250us`.
  - View-cache root classification: `top_view_cache_roots_needs_rerender=0`,
    `top_view_cache_roots_reused=1`.
  - Code-editor row guardrails: rows replayed/stored `289/0`, replay hit rate `100%`,
    code-editor p95 total/windowed callback/row paint `116/149/131us`.
  - Renderer text remains bounded at `65us` p95 in the repeat summary and `67us` in the worst-bundle stats.
  - Top layout solves are changing-bounds solves with no measured text/widget time: content `Semantics`
    `169us` (`available_w_delta=-3`, `subtree_nodes=136`, `measure_time_us=0`), root `Stack`
    `137us` (`available_w_delta=-4`, `subtree_nodes=104`, `measure_time_us=0`), and nav `Container`
    `33us` (`available_w_delta=-1`, `subtree_nodes=10`, `measure_time_us=0`).
- Decision:
  - The retained/windowed-paint view-cache rerender escape is resolved and should stay mechanism-layer.
  - The remaining local owner is now changing-bounds layout/root solve under the content `Scroll`, not
    parent view-cache rerender, renderer text, shadcn `ScrollArea` recipe policy, or editor row replay.

Post-merge root-solve attribution refresh (2026-05-17):

- Evidence:
  `target/fret-diag/local-next-root-solve-attrib-20260517-r1/worst.stats.json`
- Bundle:
  `target/fret-diag/local-next-root-solve-attrib-20260517-r1/1778949437059/bundle.schema2.json`
- Command shape:
  - `target/release/fretboard-dev diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`
  - repeat `1`, warmup `5`, standard prewarm/prelude hooks, overlay disabled, view-cache shell enabled,
    code-editor paint perf enabled, scroll/layout profiling enabled.
  - Launch command: `cargo run -p fret-ui-gallery --release --features gallery-full`.
- Verification gates rerun after merge:
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `152/152` passed.
  - `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`
    - Result: `2/2` passed.
- Result:
  - p95 total/layout/layout-roots/solve/prepaint/paint: `1242/682/468/321/264/379us`.
  - Worst frame total/layout/layout-roots/solve/prepaint/paint: `1242/618/404/305/251/373us`.
  - View-cache root classification remains fixed: `top_view_cache_roots_needs_rerender=0`,
    `top_view_cache_roots_reused=1`.
  - Code-editor row guardrails remain stable: rows replayed/stored `289/0`; row-scene replay hit
    rate stays `100%`.
  - Renderer text remains bounded at `64us`.
  - Top solves are still small-width-delta `new_frame_key_changed` roots with `measure_time_us=0`:
    content `Semantics` `172us` (`available_w_delta=-4`, `subtree_nodes=136`), root `Stack`
    `128us` (`available_w_delta=-4`, `subtree_nodes=102`), and editor `PointerRegion` `3us`.
- Decision:
  - Keep Windows RTX4090 closeout separate; this local macOS sample is not formal closeout evidence.
  - The next code slice should not skip `Scroll` layout wholesale. The correct design target is a
    narrower root-solve / geometry-propagation split that preserves side-effectful layout semantics.
  - Do not start renderer text, shadcn `ScrollArea` recipe, or row-fragment replay work from this
    evidence.

Implemented root-solve / clean-geometry propagation slice (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`can_skip_clean_geometry_engine_solve_for_resize`,
    `clean_manual_geometry_subtree_supported`, and `try_propagate_clean_engine_layout`)
  - `crates/fret-ui/src/tree/layout/solve.rs`
    (`solve_barrier_flow_root_if_needed`, `solve_barrier_flow_roots_if_needed`)
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
    (pending viewport/root solve batching)
- Contract:
  - The skip applies only to engine-backed, clean roots during small-step interactive width-only
    resize. Height changes remain on the full solve path because they can affect scroll windows and
    virtual-list visible ranges.
  - `Scroll` is treated as a boundary. Ancestors may propagate its resized bounds, but `Scroll`
    still runs layout and publishes viewport/content handles, deferred-probe state, overflow
    observation, and child scroll transforms.
  - `ViewCache` and `VirtualList` are excluded from the fast path. `ViewCache` needs a retained
    semantics proof before participating; `VirtualList` must keep resize-driven render-window
    updates authoritative.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_barrier_root_engine_solve clean_geometry_small_resize_does_not_skip_view_cache_root_engine_solve clean_parent_geometry_skip_still_runs_scroll_layout_side_effects virtual_list_render_window_range_tracks_viewport_resize --no-fail-fast`
    - Result: `4/4` passed.
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `17/17` passed.
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `153/153` passed.
  - `cargo fmt --check`
    - Result: passed.
- Local perf evidence:
  - Bundle:
    `target/fret-diag/local-next-root-solve-attrib-20260517-r3/1778956535346/bundle.schema2.json`
  - Stats command:
    `target/release/fretboard-dev diag stats target/fret-diag/local-next-root-solve-attrib-20260517-r3/1778956535346/bundle.schema2.json --sort time --top 15 --json`
  - p95/max total/layout/layout-roots/solve/prepaint/paint/text-prepare:
    `1266/716/492/331/254/385/62us`.
  - Top frame: `total=1266us`, `layout=650us`, `layout_roots=422us`,
    `layout_engine_solve=322us`, `layout_engine_solves=4`.
  - View-cache and editor guardrails remain stable: `cache_roots_reused=1`, row replay/store
    `289/0`, rows painted `289`, and code-editor p95 total/row-paint `90/105us`.
  - Remaining top solves are still small-width-delta `new_frame_key_changed` roots with no measured
    widget/text time: content `Semantics` `177us`, root `Stack` `140us`, and editor
    `PointerRegion` `3us`. The next optimization should either widen the proof with dedicated
    side-effect gates or target a different root-solve owner; it should not fold RTX4090 closeout
    into this local slice.

Remaining small-width-delta solve classification (2026-05-17):

- Evidence:
  - Bundle:
    `target/fret-diag/local-next-root-solve-attrib-20260517-r3/1778956535346/bundle.schema2.json`
  - Stats command:
    `target/release/fretboard-dev diag stats target/fret-diag/local-next-root-solve-attrib-20260517-r3/1778956535346/bundle.schema2.json --sort time --top 15 --json`
- `Semantics` solve:
  - Root path: `apps/fret-ui-gallery/src/ui/content.rs:144` (`ui-gallery-page-preview`).
  - Source shape: `apps/fret-ui-gallery/src/ui/doc_layout.rs:302` wraps preview pages in a
    vertical flex, and the code-editor torture page includes gallery-dev control rows that can use
    wrap flex.
  - Bundle profile: `available_w_delta=-4`, `available_h_delta=0`, `subtree_nodes=136`,
    `measure_time_us=0`, `flex_wrap_patch_wrap_nodes=1`, `solve_time_us=177`.
  - Decision: do not add this to clean-geometry proof by name. Wrapped flex changes require a
    line-break stability proof, or a durable contract that says when previous geometry is still
    authoritative under width deltas.
- Root `Stack` solve:
  - Root path: `root[fret-ui-gallery]`; source assembly starts in
    `apps/fret-ui-gallery/src/driver/render_flow.rs:244`.
  - Source shape: app root mixes sidebar/content view-cache boundaries, `WorkspaceFrame`,
    workspace command scope, overlays/toaster/settings/command palette/debug HUD/inspector
    wrappers, plus the shell content wrapper in `apps/fret-ui-gallery/src/driver/shell.rs:134`.
  - Bundle profile: `available_w_delta=-4`, `available_h_delta=0`, `subtree_nodes=102`,
    `measure_time_us=0`, `solve_time_us=140`.
  - Decision: do not keep growing the wrapper whitelist to cover this root. It needs an explicit
    layout side-effect / geometry-propagation contract for app shell and cache-boundary nodes.
- Editor `PointerRegion` solve:
  - Root path: `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs:783`.
  - Source shape: `PointerRegion -> Canvas` under the windowed rows surface; `Canvas` layout is
    currently a leaf clamp in `crates/fret-ui/src/declarative/host_widget/layout.rs:1570`, while
    paint/prepaint hooks can depend on current bounds.
  - Bundle profile: `available_w_delta=-4`, `available_h_delta=0`, `subtree_nodes=2`,
    `measure_time_us=0`, `solve_time_us=3`.
  - Decision: a dedicated `Canvas` leaf geometry proof is plausible, but the measured benefit is
    too small for this slice. If attempted later, add tests for updated canvas bounds,
    prepaint/paint hooks, and windowed-row hit-test bounds.
- Overall decision:
  - Stop expanding clean-geometry proof ad hoc. The next meaningful optimization should first
    introduce a contract or classification surface for layout side effects and geometry-only
    propagation, plus diagnostics that report the first unsupported kind/reason when a clean root
    cannot skip its solve.

Layout side-effect / geometry-propagation contract diagnostics slice (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`CleanGeometryNodeContract`, `CleanGeometrySolveSkipRejectionReason`,
    `can_skip_clean_geometry_engine_solve_for_resize`)
  - `crates/fret-ui/src/tree/debug/frame_stats.rs`
    (`layout_clean_geometry_solve_skip_*`)
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/frame_stats.rs`
    (`UiFrameStatsV1` clean-geometry rejection fields)
- Contract:
  - The current proof is explicit instead of a silent bool/`Option` chain: pure pass-through
    geometry, no-wrap vertical flex, safe leaves, and side-effect boundaries are separate internal
    classes.
  - `Scroll` remains a side-effect boundary. A parent may propagate geometry to it, but a root
    `Scroll` cannot skip its own layout solve body via this proof.
  - Unsupported retained/windowing/line-breaking surfaces continue to reject with a reason and
    optional element kind. This keeps `ViewCache`, `VirtualList`, wrap flex, and future `Canvas`
    participation behind dedicated proofs rather than ad hoc name expansion.
  - Per-frame diagnostics record only a rejection count plus the first reason/kind, avoiding
    high-volume per-node strings while still making top rejected solve owners explainable.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_barrier_root_engine_solve clean_geometry_small_resize_does_not_skip_view_cache_root_engine_solve clean_geometry_small_resize_reports_wrap_flex_rejection_reason clean_parent_geometry_skip_still_runs_scroll_layout_side_effects virtual_list_render_window_range_tracks_viewport_resize --no-fail-fast`
    - Result: `5/5` passed.
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `18/18` passed.
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `153/153` passed.
  - `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`
    - Result: passed.
  - `cargo fmt --check`
    - Result: passed.

Per-solve clean-geometry rejection attribution refresh (2026-05-17):

- Evidence before per-solve attribution:
  `target/fret-diag/local-next-clean-geometry-rejections-20260517-r1/1778978609337/bundle.schema2.json`
- Evidence after per-solve attribution:
  `target/fret-diag/local-next-clean-geometry-rejections-20260517-r2/1778979436452/bundle.schema2.json`
- Command shape:
  - `target/release/fretboard-dev diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`
  - repeat `1`, warmup `5`, standard prewarm/prelude hooks, overlay disabled, view-cache shell
    enabled, code-editor paint perf enabled, scroll/layout profiling enabled.
  - Launch command: `cargo run -p fret-ui-gallery --release --features gallery-full`.
- Mechanism anchors:
  - `crates/fret-ui/src/tree/debug/layout.rs`
    (`UiDebugLayoutEngineSolve::clean_geometry_solve_skip_rejection`)
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`debug_record_clean_geometry_solve_skip_rejection`)
  - `crates/fret-ui/src/tree/ui_tree_debug/record.rs`
    (`debug_record_layout_engine_solve`)
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/layout_paint_hotspot_diagnostics.rs`
    (`UiLayoutEngineSolveV1::clean_geometry_solve_skip_rejection`)
- Result:
  - r2 top frame: `total=1257us`, `layout=632us`, `layout_engine_solve=306us`,
    `layout_engine_solves=4`.
  - View-cache and editor guardrails stay stable: `top_view_cache_roots_reused=1`,
    `top_view_cache_roots_needs_rerender=0`, row replay/store `289/0`, row replay hit rate `100%`.
  - Per-solve rejection attribution:
    - content `Semantics` solve `180/173/162us`: `unsupported_kind`, blocker `Container`,
      `subtree_nodes=136`, `flex_wrap_patch_wrap_nodes=1`;
    - root `Stack` solve `134/130/128us`: `unsupported_kind`, blocker `Container`,
      `subtree_nodes=102`, `flex_wrap_patch_wrap_nodes=0`;
    - nav `Container` solve `28us`: `unsupported_kind`, blocker `Container`;
    - editor `PointerRegion` solve `3-4us`: `unsupported_kind`, blocker `Canvas`;
    - root `Scroll` solve `0us`: `side_effect_boundary`, blocker `Scroll`.
- Focused gates:
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `18/18` passed.
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `153/153` passed.
  - `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`
    - Result: passed.
  - `cargo fmt --check`
    - Result: passed.
- Decision:
  - Do not use RTX4090 closeout as the completion condition for this local slice.
  - Do not start with `Canvas`; the measured solve is too small for the next primary owner.
  - Do not skip `Scroll`; its side-effect boundary is authoritative and the measured root solve is
    `0us` in this sample.
  - The next candidate is a conservative `Container` geometry contract. Even if that lands,
    `Semantics` may immediately expose the known wrap-flex blocker, so the next patch should be
    proof-first and evidence-driven rather than a broad whitelist expansion.

Conservative Container geometry contract slice (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`CleanGeometryChildBoundsStrategy::ContainerPxInsets`,
    `clean_container_width_delta_child_bounds`,
    `clean_engine_geometry_propagation_requires_manual_child_bounds`)
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_skips_px_container_and_updates_child_bounds`,
    `clean_geometry_small_resize_rejects_container_fraction_padding`)
- Contract:
  - `Container` can participate in clean width-delta geometry propagation only when its child bounds
    are derived manually from px padding/border insets and previous clean geometry.
  - `Container` is manual-bounds-only for this fast path. If its conservative proof fails, it does
    not fall back to stale engine local child rects.
  - Fraction/fill padding and non-static child geometry remain on the full solve path until their
    basis/positioning semantics are proven.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_px_container_and_updates_child_bounds clean_geometry_small_resize_rejects_container_fraction_padding --no-fail-fast`
    - Red before implementation: px `Container` still solved once; fraction padding reported
      `unsupported_kind / Container`.
    - Green after implementation: `2/2` passed.
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `20/20` passed.
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `153/153` passed.
  - `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`
    - Result: passed.
- Local perf evidence:
  - Bundle:
    `target/fret-diag/local-next-container-clean-geometry-20260517-r1/1778981585274/bundle.schema2.json`
  - Stats:
    `target/fret-diag/local-next-container-clean-geometry-20260517-r1/worst.stats.json`
  - Top frame total/layout/layout-roots/solve/prepaint/paint:
    `1242/623/414/297/243/376us`.
  - p95 total/layout/layout-roots/solve/prepaint/paint:
    `1242/706/490/319/347/462us`.
  - Guardrails remain stable: view-cache root reused `1`, needs-rerender `0`, row replay/store
    `289/0`, renderer text prepare `62us`.
  - Per-solve blocker shift:
    - content `Semantics` solve `167us`: `auto_child_height`, blocker `Container`,
      `wrap_nodes=1`;
    - root `Stack` solve `125us`: `auto_child_height`, blocker `Flex`, `wrap_nodes=0`;
    - editor `PointerRegion` solve `4us`: `unsupported_kind`, blocker `Canvas`;
    - root `Scroll` solve `0us`: `side_effect_boundary`, blocker `Scroll`.
- Decision:
  - Keep the Container proof; it is small, tested, and makes the next blocker explicit.
  - Do not widen auto-height next by name. First classify whether each `auto_child_height` blocker is
    genuinely width/reflow-dependent; the content `Semantics` root still contains wrap flex and must
    stay blocked until line-break stability is proven.

Auto-height / size-stability classification slice (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`CleanGeometryWidthDeltaSizeStability::StableComputedBox`,
    `clean_child_height_style_supported_for_width_delta`,
    `clean_child_width_style_supported_for_width_delta`)
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_skips_stable_auto_height_container_wrapper`,
    `clean_geometry_small_resize_skips_stable_auto_height_vertical_flex_child`,
    `clean_geometry_small_resize_rejects_auto_height_text_reflow`)
- Contract:
  - `height: Auto` is not enough by itself to reject a clean width-delta propagation. Stable
    wrappers can continue only if recursive child-bound propagation proves the descendant geometry
    remains stable.
  - Text leaves are stable only when their computed box size is unchanged; otherwise they reject
    with `text_reflow` so line-break and wrap-dependent height changes keep the authoritative solve.
  - Fraction/fill width constraints, non-px margins, aspect-ratio height coupling, wrap flex, side
    effect boundaries, and unsupported layout containers remain on the full solve path.
- Classification audit:
  - The current `CleanGeometryNodeContract` shape is acceptable for this local fast path, but it is
    not the final architecture for all layout nodes. Before extending to `Grid`, horizontal flex,
    retained/cache nodes, canvas/prepaint, layout queries, or transforms, split the model into
    explicit axes: layout side effects, parent-derived child-bounds strategy, and width-delta size
    stability.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_stable_auto_height_container_wrapper clean_geometry_small_resize_skips_stable_auto_height_vertical_flex_child clean_geometry_small_resize_rejects_auto_height_text_reflow --no-fail-fast`
    - Result: `3/3` passed.
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `23/23` passed.
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `153/153` passed.
- Local perf evidence:
  - Bundle:
    `target/fret-diag/local-next-auto-height-classification-20260517-r1/1778984176582/bundle.schema2.json`
  - Stats:
    `target/fret-diag/local-next-auto-height-classification-20260517-r1/worst.stats.json`
  - Top frame total/layout/layout-roots/solve/prepaint/paint:
    `1231/626/405/311/236/369us`.
  - p95 total/layout/layout-roots/solve/prepaint/paint:
    `1231/707/482/325/252/383us`.
  - Guardrails remain stable: view-cache root reused `1`, needs-rerender `0`, row replay/store
    `289/0`, renderer text prepare `62us`.
  - Per-solve blocker shift:
    - content `Semantics` solve `180/174/162us`: `unsupported_kind`, blocker `Grid`,
      `wrap_nodes=1`;
    - root `Stack` solve `139/133/128us`: `flex_direction`, blocker `Flex`, `wrap_nodes=0`;
    - nav `Container` solve `29us`: `flex_direction`, blocker `Flex`;
    - editor `PointerRegion` solve `4us`: `unsupported_kind`, blocker `Canvas`;
    - root `Scroll` solve `0us`: `side_effect_boundary`, blocker `Scroll`.
- Decision:
  - Do not keep pushing `auto_child_height`; the useful classification has landed and the next
    blocker moved.
  - Do not start a broad node-classification rewrite solely from this evidence. Start the next slice
    with either a proof-first `Grid` / horizontal `Flex` geometry contract or a small formal
    classification-model refactor if those proofs cannot stay local.

Clean-geometry classification model refactor (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`CleanGeometryNodeContract`,
    `CleanGeometryLayoutEffect`,
    `CleanGeometryChildBoundsStrategy`,
    `CleanGeometryWidthDeltaSizeStability`,
    `clean_geometry_boundary_layout_node_kind`)
- Contract:
  - The supported node set and rejection strings are unchanged. This is a model-clarity refactor,
    not a fast-path expansion.
  - `CleanGeometryNodeContract` now records three explicit axes:
    layout side effects (`Pure` vs `SideEffectBoundary`), parent-derived child-bound strategy
    (`None`, `PreserveLocalOrigins`, `ContainerPxInsets`, `VerticalNoWrapFlex`), and width-delta
    size stability (`Propagated` vs `StableComputedBox`).
  - Side-effect boundary detection reads the same contract as the recursive clean-geometry proof,
    so future layout-effect boundaries do not need a second element-name table.
  - The next `Grid` or horizontal `Flex` slice should add a new child-bound strategy only after a
    focused proof. `ViewCache`, `VirtualList`, `Canvas`, layout queries, transforms, and retained
    surfaces remain excluded until their side effects and bounds dependencies are proven.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_px_container_and_updates_child_bounds clean_geometry_small_resize_rejects_container_fraction_padding clean_geometry_small_resize_skips_stable_auto_height_container_wrapper clean_geometry_small_resize_skips_stable_auto_height_vertical_flex_child clean_geometry_small_resize_rejects_auto_height_text_reflow --no-fail-fast`
    - Result: `5/5` passed.
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `23/23` passed.
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `153/153` passed.
  - `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`
    - Result: passed.
  - `cargo fmt`
    - Result: passed.
  - `python3 tools/check_layering.py`
    - Result: passed.
- Decision:
  - No new workstream is needed for this refactor; it closes the local M7/M9 architecture concern.
  - The next optimization can proceed as a bounded `Grid` / horizontal `Flex` proof in
    `scroll-optimization-v1`, with RTX4090 validation still tracked as follow-up evidence rather
    than a local completion gate.

Horizontal fixed Flex clean-geometry proof (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`CleanGeometryChildBoundsStrategy::HorizontalFixedFlex`,
    `clean_horizontal_fixed_flex_width_delta_child_bounds`,
    `CleanGeometrySolveSkipRejectionReason::FlexItemSizing`,
    `CleanGeometryNodeContract::propagated_leaf`)
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_skips_fixed_horizontal_flex_children`,
    `clean_geometry_small_resize_skips_center_aligned_fixed_horizontal_flex_children`,
    `clean_geometry_small_resize_rejects_center_aligned_vertical_flex_child`,
    `clean_geometry_small_resize_rejects_horizontal_flex_grow_children`)
- Contract:
  - Horizontal no-wrap `Flex` can participate only when main-axis distribution is fixed:
    `justify=Start`, px padding/gap, static children, px margins, default order, zero grow/shrink,
    `basis=Auto`, no `align_self`, and non-auto/non-fill/non-fraction child main-axis widths.
  - Horizontal cross-axis alignment can be preserved from previous clean geometry because this proof
    only runs for width-only resize and rejects parent height deltas. Stretch-height children still
    update to the stable inner height.
  - Vertical no-wrap `Flex` keeps the older stricter cross-axis rule: non-stretch alignment still
    rejects with `flex_cross_align` because a width delta can move children horizontally.
  - Text leaves remain stable-computed boxes, while pure geometry leaves such as `Spacer`, `Image`,
    `SvgIcon`, `SvgImage`, and `Spinner` are propagated leaves. This keeps text reflow blocked
    without forcing geometry-only leaves to require unchanged boxes.
  - `ViewCache`, `VirtualList`, `Canvas`, layout-query/transform nodes, and root `Scroll` remain
    excluded or boundary-protected.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_stable_auto_height_vertical_flex_child clean_geometry_small_resize_rejects_center_aligned_vertical_flex_child clean_geometry_small_resize_skips_fixed_horizontal_flex_children clean_geometry_small_resize_skips_center_aligned_fixed_horizontal_flex_children clean_geometry_small_resize_rejects_horizontal_flex_grow_children --no-fail-fast`
    - Result: `5/5` passed.
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_fixed_horizontal_flex_children clean_geometry_small_resize_skips_center_aligned_fixed_horizontal_flex_children clean_geometry_small_resize_rejects_horizontal_flex_grow_children --no-fail-fast`
    - Red/green note: before the horizontal cross-axis proof, the center-aligned horizontal case
      still solved once with `flex_cross_align / Flex`; after the proof, `3/3` passed.
- Final gates:
  - `cargo fmt --check`
    - Result: passed.
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_barrier_root_engine_solve clean_geometry_small_resize_does_not_skip_view_cache_root_engine_solve clean_geometry_small_resize_reports_wrap_flex_rejection_reason clean_geometry_small_resize_skips_px_container_and_updates_child_bounds clean_geometry_small_resize_rejects_container_fraction_padding clean_geometry_small_resize_skips_stable_auto_height_container_wrapper clean_geometry_small_resize_skips_stable_auto_height_vertical_flex_child clean_geometry_small_resize_rejects_center_aligned_vertical_flex_child clean_geometry_small_resize_rejects_auto_height_text_reflow clean_geometry_small_resize_skips_fixed_horizontal_flex_children clean_geometry_small_resize_skips_center_aligned_fixed_horizontal_flex_children clean_geometry_small_resize_rejects_horizontal_flex_grow_children --no-fail-fast`
    - Result: `12/12` passed.
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `27/27` passed.
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `153/153` passed.
  - `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`
    - Result: passed.
  - `python3 tools/check_layering.py`
    - Result: passed.
  - `git diff --check`
    - Result: passed.
- Local blocker-shift evidence:
  - Bundle:
    `target/fret-diag/local-next-horizontal-flex-clean-geometry-20260517-r2/1778989269602/bundle.schema2.json`
  - Stats:
    `target/fret-diag/local-next-horizontal-flex-clean-geometry-20260517-r2/worst.stats.json`
  - Command shape:
    - `target/release/fretboard-dev diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`
    - repeat `1`, warmup `5`, standard prewarm/prelude hooks, overlay disabled, view-cache shell
      enabled, code-editor paint perf enabled, scroll/layout/node profiling enabled.
    - Launch command: `cargo run -p fret-ui-gallery --release --features gallery-full`.
  - Result:
    - The sample is useful for blocker classification, not for a perf-win claim: max/p95
      total/layout/layout-roots/solve/prepaint/paint/text-prepare is
      `3234/1630/1113/823/909/1103/94us`, which shows local variance beyond the clean-geometry
      change itself.
    - Guardrails remain stable in the top row: view-cache root reused `1`, needs-rerender `0`, row
      replay/store `289/0`, and row-scene replay hit rate `100%`.
    - Per-solve blocker shift:
      - content `Semantics`: `unsupported_kind=Grid`, `wrap_nodes=1`, solve `190/321/405us`
        across the sampled solve frames;
      - root `Stack`: `flex_item_sizing`, blocker `Flex`, solve `151/496/248us`;
      - nav `Container`: `flex_item_sizing`, blocker `Flex`, solve `48us` on the sampled nav frame;
      - editor `PointerRegion`: `unsupported_kind=Canvas`, solve `5-18us`;
      - root `Scroll`: `side_effect_boundary`, blocker `Scroll`, solve `0us`.
- Decision:
  - Keep the horizontal fixed `Flex` proof; it removes the cross-align false blocker without
    weakening vertical flex or main-axis sizing semantics.
  - Do not broaden to grow/fill horizontal flex in the same patch. The next app-shell/nav slice
    would need a dedicated main-axis distribution proof for `flex_item_sizing`.
  - Do not start `Grid` until the content `Semantics` root's wrap-flex context is handled or the
    Grid proof explicitly accounts for the visible `wrap_nodes=1` blocker.

Horizontal basis-zero grow Flex proof (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`clean_horizontal_fixed_flex_width_delta_child_bounds`,
    `clean_horizontal_flex_basis0_grow_item_next_width`,
    `clean_horizontal_fixed_flex_item_supported`)
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_skips_horizontal_flex_single_basis0_grow_child`,
    `clean_geometry_small_resize_rejects_horizontal_flex_multiple_grow_children`)
- Contract:
  - Horizontal no-wrap `Flex` can derive child bounds during small width-only resize when the row
    has exactly one main-axis flexible child with `basis: 0px`, positive finite `grow`,
    non-negative finite `shrink`, `width: Auto | Fill`, and px/auto min/max constraints that are
    not crossed by the delta.
  - All siblings must still satisfy the fixed main-axis subset from the previous proof. Their
    widths stay unchanged, while siblings after the flexible child shift by the flexible child's
    width delta.
  - Multiple flexible children remain rejected because distribution requires proportional free-space
    allocation and rounding/min/max proof. `basis=Auto`, non-zero basis, `width: Px + grow`,
    fractional/fill max widths, wrapped flex, non-static children, and non-px spacing remain on the
    authoritative solve path.
  - The local Taffy audit confirms this cannot be generalized to default shrink behavior: negative
    free-space distribution scales by `inner_flex_basis * flex_shrink`, so `basis=Auto +
    width=Fill + grow=1 + shrink=1` needs a separate contract.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_horizontal_flex_single_basis0_grow_child clean_geometry_small_resize_rejects_horizontal_flex_multiple_grow_children --no-fail-fast`
    - Result: `2/2` passed.
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `29/29` passed.
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `153/153` passed.
  - `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`
    - Result: passed.
  - `cargo fmt`
    - Result: passed.
  - `cargo fmt --check`
    - Result: passed.
  - `python3 tools/check_layering.py`
    - Result: passed.
  - `git diff --check`
    - Result: passed.
- Local blocker evidence:
  - Bundle:
    `target/fret-diag/local-next-horizontal-flex-basis0-grow-clean-geometry-20260517-r1/1778992910970/bundle.schema2.json`
  - Command shape:
    - `target/release/fretboard-dev diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`
    - repeat `1`, warmup `5`, `--reuse-launch`, standard prewarm/prelude hooks, overlay disabled,
      view-cache shell enabled, code-editor paint perf enabled, scroll/layout/node profiling enabled.
    - Launch command: `cargo run -p fret-ui-gallery --release --features gallery-full`.
  - Result:
    - Top frame total/layout/layout-engine-solve is `1307/757/350us` with `5` layout-engine solves.
    - Guardrails remain stable: `top_view_cache_roots_reused=1`,
      `top_view_cache_roots_needs_rerender=0`, row replay/store `289/0`.
    - Per-solve blockers:
      - content `Semantics`: `unsupported_kind=Grid`;
      - root `Stack`: `flex_item_sizing / Flex`, solve about `130/128/141us`;
      - nav `Container`: `flex_item_sizing / Flex`, solve about `35us`;
      - editor `PointerRegion`: `unsupported_kind=Canvas`;
      - root `Scroll`: `side_effect_boundary / Scroll`, solve `0us`.
- Decision:
  - Keep the proof because it is a correct, tested mechanism subset.
  - Do not treat it as the app-shell/nav fix. The remaining blockers appear to use `basis=Auto +
    width=Fill + grow=1 + default shrink`, so the next slice should explicitly audit and prove
    that shape or update the ecosystem authoring contract to emit a basis-zero grow row when that is
    the intended policy.

App-shell Flex authoring and fixed auto-width chrome proof (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`clean_horizontal_fixed_flex_item_supported`,
    `clean_child_width_constraints_allow_preserved_width`)
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_skips_horizontal_flex_auto_width_no_shrink_child`,
    `clean_geometry_small_resize_rejects_horizontal_flex_auto_width_child_fractional_max_constraint`)
- Ecosystem/app authoring anchors:
  - `ecosystem/fret-workspace/src/frame.rs`
    (`flex_grow_layout`, `fill_grow_layout`)
  - `ecosystem/fret-workspace/src/tab_strip/layouts.rs`
    (`fill_grow_layout`)
  - `apps/fret-ui-gallery/src/driver/shell.rs`
    (fixed sidebar and content pane layout)
  - `apps/fret-ui-gallery/src/driver/render_flow.rs`
    (workspace content wrappers and center row layout)
- Contract:
  - Grow-driven workspace/content slots use explicit `basis: 0px` plus `grow=1`, `shrink=1`,
    and `min-width: 0` where the authoring intent is Tailwind-like `flex-1`.
  - Fixed-width sidebar/chrome slots opt out of default flex shrink with `shrink=0`.
  - Horizontal no-wrap flex can preserve a fixed auto-width no-shrink child during small width-only
    resize when its previous computed width satisfies auto/px min/max constraints. The child keeps
    its computed bounds, while the already-proven single basis-zero grow child absorbs the width
    delta.
  - Fraction/fill constraints on the preserved auto-width item still reject because they require a
    parent-width basis proof. Multiple grow items, wrapped flex, root `Scroll`, `ViewCache`,
    `VirtualList`, layout-query/transform nodes, and `Canvas` remain excluded.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_horizontal_flex_auto_width_no_shrink_child clean_geometry_small_resize_rejects_horizontal_flex_auto_width_child_fractional_max_constraint clean_geometry_small_resize_skips_horizontal_flex_single_basis0_grow_child clean_geometry_small_resize_rejects_horizontal_flex_multiple_grow_children --no-fail-fast`
    - Result: `4/4` passed.
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `31/31` passed.
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `153/153` passed.
  - `cargo nextest run -p fret-workspace workspace_frame_center_row_does_not_fill_height --no-fail-fast`
    - Result: `1/1` passed.
  - `cargo check -p fret-ui-gallery --features gallery-full`
    - Result: passed.
  - `cargo fmt --check`
    - Result: passed.
- Local blocker evidence:
  - Bundle:
    `target/fret-diag/local-next-horizontal-flex-authoring-and-auto-fixed-clean-geometry-20260517-r1/1778996086098/bundle.schema2.json`
  - Stats:
    `target/fret-diag/local-next-horizontal-flex-authoring-and-auto-fixed-clean-geometry-20260517-r1/worst.stats.json`
  - Result:
    - Top frame total/layout/layout-engine-solve is `1212/583/258us` with `4` layout-engine solves.
    - Guardrails remain stable: `top_view_cache_roots_reused=1`,
      `top_view_cache_roots_needs_rerender=0`, row replay/store `289/0`, row replay hit rate
      `100%`, renderer text prepare `64us`.
    - The previous nav `Container` `flex_item_sizing / Flex` solve no longer appears in the top
      per-solve blockers after the app-shell authoring fix plus fixed auto-width proof.
    - Remaining blockers:
      - content `Semantics`: `unsupported_kind=Grid`, `wrap_nodes=1`, solve about `172-175us`;
      - root `Stack`: `flex_item_sizing / RovingFlex`, solve about `80-88us`;
      - editor `PointerRegion`: `unsupported_kind=Canvas`, solve `3-4us`;
      - root `Scroll`: `side_effect_boundary / Scroll`, solve `0us`.
- Decision:
  - Keep this as a completed app-shell authoring + narrow mechanism proof. It removes the nav
    blocker and improves the root solve cost without broadening into unsupported layout classes.
  - The next slice should not keep expanding generic `Flex` by name. It should either prove the
    remaining root `RovingFlex` item-sizing shape or tackle content `Grid` with an explicit
    wrap-flex/line-break stability story.

RovingFlex trigger authoring closeout (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_skips_horizontal_roving_flex_auto_width_no_shrink_child`)
  - `ecosystem/fret-ui-shadcn/src/context_menu.rs`
    (`ContextMenu::trigger_region_layout`)
  - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
    (`workspace_tab_item_layout`)
  - `ecosystem/fret/src/in_window_menubar.rs`
    (`menubar_trigger_layout`)
- Contract:
  - `RovingFlex` already uses the same clean horizontal flex proof as `Flex` for the proven
    single basis-zero grow child plus fixed auto-width no-shrink item shape.
  - Wrapper components that become the real flex item must preserve the caller's intended item
    semantics. `ContextMenu` therefore exposes a trigger-region layout override instead of making
    callers rely on the visible trigger child's layout after wrapping.
  - Workspace tabs and top-level in-window menubar triggers are fixed auto-width chrome items in
    horizontal roving rows. They opt out of default flex shrink; inner text/chrome owns clipping or
    overflow, and the row/scroll surface owns overflow behavior.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_horizontal_roving_flex_auto_width_no_shrink_child --no-fail-fast`
    - Result: `1/1` passed.
  - `cargo test -p fret --lib menubar_trigger_layout_keeps_trigger_width_out_of_flex_shrink`
    - Result: `1/1` passed.
  - `cargo test -p fret-workspace --lib workspace_tab_item_layout_keeps_tab_width_out_of_flex_shrink`
    - Result: `1/1` passed.
  - `cargo test -p fret-ui-shadcn --lib context_menu_trigger_region_layout_can_forward_outer_flex_item_semantics`
    - Result: `1/1` passed.
  - `cargo fmt`
    - Result: passed.
  - `git diff --check`
    - Result: passed.
- Local blocker evidence:
  - Bundle:
    `target/fret-diag/local-next-menubar-rovingflex-trigger-layout-20260517-r1/1779002779663/bundle.schema2.json`
  - Stats:
    `target/fret-diag/local-next-menubar-rovingflex-trigger-layout-20260517-r1/worst.stats.json`
  - Result:
    - Top frame total/layout/layout-engine-solve/prepaint/paint is `1274/609/269/269/396us` with
      `4` layout-engine solves.
    - Guardrails remain stable: `top_view_cache_roots_reused=1`,
      `top_view_cache_roots_needs_rerender=0`, row replay/store `289/0`, row replay hit rate
      `100%`, renderer text prepare `65us`.
    - The root `Stack` per-solve blocker moved from `flex_item_sizing / RovingFlex` to
      `missing_measured_size`.
    - Remaining blockers:
      - content `Semantics`: `unsupported_kind=Grid`, `wrap_nodes=1`, solve `183us`;
      - root `Stack`: `missing_measured_size`, solve `88us`;
      - editor `PointerRegion`: `unsupported_kind=Canvas`, solve `4us`;
      - root `Scroll`: `side_effect_boundary / Scroll`, solve `0us`.
- Decision:
  - Close the `RovingFlex` item-sizing blocker as an authoring-policy correction, not a broader
    mechanism expansion.
  - Do not fold the next blocker into this patch. The next slice should first classify the root
    `missing_measured_size` path or separately prove the content `Grid` / wrap-flex line-break
    stability story.

Root missing-measured-size attribution and workspace fill-slot closeout (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`CleanGeometrySolveSkipRejection::node`,
    `debug_record_clean_geometry_solve_skip_rejection`)
  - `crates/fret-ui/src/tree/debug/layout.rs`
    (`UiDebugCleanGeometrySolveSkipRejection`)
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/layout_paint_hotspot_diagnostics.rs`
    (`UiCleanGeometrySolveSkipRejectionV1`)
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_rejection_reports_descendant_node_attribution`,
    `clean_geometry_small_resize_skips_horizontal_flex_empty_grow_container_slot`)
  - `ecosystem/fret-workspace/src/frame.rs`
    (`flex_fill_slot`)
- Contract:
  - Clean-geometry rejection diagnostics must identify the actual rejected node, not only the root
    solve that attempted the skip. This keeps root-level blockers actionable when the first
    rejection is a deep descendant.
  - Diagnostics bundles now include `node`, `element`, `element_kind`, and `element_path` on
    per-solve clean-geometry rejections when the data is available.
  - Empty app-shell flex-fill slots should be authored as explicit basis-zero grow slots with no
    hit-test participation, not as a default `Spacer`. A default `Spacer` can legitimately measure
    to `0x0`, which collides with the current `Size::default()` missing-measure sentinel.
- Pre-fix attribution evidence:
  - Bundle:
    `target/fret-diag/local-next-missing-measured-size-attribution-20260517-r1/1779005085651/bundle.schema2.json`
  - Result:
    - The root `Stack` solve reported `missing_measured_size`.
    - The rejected descendant was a `Spacer` with a path through
      `apps/fret-ui-gallery/src/driver/chrome.rs:87` and
      `ecosystem/fret-workspace/src/frame.rs:353` / `frame.rs:387`.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_does_not_skip_view_cache_root_engine_solve clean_geometry_rejection_reports_descendant_node_attribution clean_geometry_small_resize_reports_wrap_flex_rejection_reason clean_geometry_small_resize_skips_horizontal_flex_empty_grow_container_slot clean_geometry_small_resize_skips_horizontal_flex_auto_width_no_shrink_child clean_geometry_small_resize_skips_horizontal_roving_flex_auto_width_no_shrink_child clean_geometry_small_resize_rejects_horizontal_flex_auto_width_child_fractional_max_constraint --no-fail-fast`
    - Result: `7/7` passed.
  - `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`
    - Result: passed.
  - `cargo check -p fret-workspace`
    - Result: passed.
  - `cargo fmt --check`
    - Result: passed.
  - `git diff --check`
    - Result: passed.
- Residual workspace test note:
  - `cargo nextest run -p fret-workspace -E 'test(workspace_root_drop_after_tab_pointer_up_dispatches_split_and_move)' --no-fail-fast`
    currently fails with `expected workspace tab drag to start after threshold`.
  - The failing test does not reference `WorkspaceTopBar`, `WorkspaceStatusBar`, or `frame.rs`; it
    is recorded as residual risk rather than evidence against the empty fill-slot fix.
- Post-fix local blocker evidence:
  - Bundle:
    `target/fret-diag/local-next-workspace-fill-slot-clean-geometry-20260517-r1/1779006799472/bundle.schema2.json`
  - Result:
    - Top frame total/layout/layout-engine-solve is `1184/585/261us` with `4` layout-engine
      solves.
    - Guardrails remain stable: `top_view_cache_roots_reused=1`, row replay/store `289/0`.
    - The root `Stack` per-solve blocker moved from `missing_measured_size / Spacer` to
      `flex_main_align / Flex`.
    - Remaining blockers:
      - content `Semantics`: `unsupported_kind=Grid`, `wrap_nodes=1`;
      - root `Stack`: `flex_main_align / Flex`, path through shadcn Button chrome;
      - editor `PointerRegion`: `unsupported_kind=Canvas`, solve about `4us`;
      - root `Scroll`: `side_effect_boundary / Scroll`, solve `0us`.
- Architecture note:
  - `measured_size: Size` still uses `Size::default()` as a not-measured sentinel, which is
    ambiguous with a legal `0x0` measured box. A future `Option<Size>` refactor is likely the
    cleaner data model, but it should be handled separately because it touches layout
    early-return, viewport batching, paint cache, scroll seed, and test harness behavior.
- Decision:
  - Close the root `missing_measured_size` blocker as a diagnostics + authoring-policy fix.
  - Keep RTX4090 closeout as follow-up evidence, not a local completion gate.
  - The next proof should target either the new `flex_main_align / Flex` blocker or the content
    `Grid` / wrap-flex line-break stability story, with separate evidence and gates.

Centered intrinsic horizontal Flex proof for shadcn Button chrome (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`clean_horizontal_fixed_flex_width_delta_child_bounds`,
    `clean_horizontal_preserved_flex_item_supported`)
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_skips_center_justified_intrinsic_horizontal_flex`,
    `clean_geometry_small_resize_rejects_center_justified_fill_horizontal_flex_width_delta`)
  - `ecosystem/fret-ui-shadcn/src/button.rs`
    (`Button::content_justify`, default `Justify::Center`, and the content `Flex`)
- Contract:
  - shadcn Button's default centered content row is correct recipe policy: it matches upstream
    `inline-flex items-center justify-center`.
  - The clean-geometry fast path may preserve non-start main-axis aligned horizontal flex children
    only when the row's own inner width is unchanged. In that case, center/end/between alignment
    does not produce new positions under the parent width delta.
  - When the row's inner width changes, non-start main-axis alignment still rejects with
    `flex_main_align`; centered fill rows keep their authoritative Taffy solve.
  - For the zero-width-delta row, fixed/intrinsic child widths with default finite shrink may be
    preserved because no new negative/positive free-space distribution is needed. Grow, non-auto
    basis, align-self, fractional/fill constraints, and real width-delta distribution remain
    rejected.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_center_justified_intrinsic_horizontal_flex clean_geometry_small_resize_rejects_center_justified_fill_horizontal_flex_width_delta --no-fail-fast`
    - Result: `2/2` passed.
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_center_justified_intrinsic_horizontal_flex clean_geometry_small_resize_rejects_center_justified_fill_horizontal_flex_width_delta clean_geometry_small_resize_skips_horizontal_flex_single_basis0_grow_child clean_geometry_small_resize_rejects_horizontal_flex_multiple_grow_children clean_geometry_small_resize_skips_horizontal_flex_auto_width_no_shrink_child clean_geometry_small_resize_rejects_horizontal_flex_auto_width_child_fractional_max_constraint clean_geometry_small_resize_skips_horizontal_flex_empty_grow_container_slot clean_geometry_small_resize_skips_horizontal_roving_flex_auto_width_no_shrink_child --no-fail-fast`
    - Result: `8/8` passed.
- Local blocker evidence:
  - Intermediate bundle after the main-align proof:
    `target/fret-diag/local-next-flex-main-align-clean-geometry-20260517-r1/1779008013390/bundle.schema2.json`
    - The shadcn Button content row moved from `flex_main_align / Flex` to
      `flex_item_sizing / Flex`, confirming the main-axis alignment proof was correct but still
      needed the zero-width-delta item-sizing proof.
  - Final bundle:
    `target/fret-diag/local-next-flex-main-align-clean-geometry-20260517-r2/1779008670784/bundle.schema2.json`
  - Stats:
    `target/fret-diag/local-next-flex-main-align-clean-geometry-20260517-r2/worst.stats.json`
  - Result:
    - Top frame total/layout/layout-engine-solve is `1309/683/297us` with `4` layout-engine
      solves. This is recorded as blocker-shift evidence; the sample has local variance and is not
      a perf-win claim.
    - Guardrails remain stable: `top_view_cache_roots_reused=1`,
      `top_view_cache_roots_needs_rerender=0`, row replay/store `289/0`, row replay hit rate
      `100%`, renderer text prepare `62us`.
    - The shadcn Button chrome path no longer appears as `flex_main_align / Flex` or
      `flex_item_sizing / Flex`.
    - Remaining blockers:
      - content `Semantics`: `unsupported_kind=Grid`, `wrap_nodes=1`;
      - root `Stack`: `flex_item_sizing / Flex` at
        `apps/fret-ui-gallery/src/driver/render_flow.rs:336`, solve about `68-95us`;
      - editor `PointerRegion`: `unsupported_kind=Canvas`, solve about `3-5us`;
      - root `Scroll`: `side_effect_boundary / Scroll`, solve `0us`.
- Decision:
  - Keep this as a narrow mechanism proof for intrinsic centered rows. It fixes the Button blocker
    without changing shadcn recipe semantics or widening real width-delta flex distribution.
  - The next slice should target either the new app-shell `render_flow.rs:336` flex-item sizing
    shape or the content `Grid` / wrap-flex line-break stability story, not both.

Gallery sidebar no-shrink authoring closeout (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_rejects_horizontal_flex_fixed_px_default_shrink_child`)
  - `apps/fret-ui-gallery/src/ui/nav.rs`
    (`sidebar_view`)
- Contract:
  - A fixed-width sidebar in a horizontal app-shell row is fixed chrome, so app authoring should
    express it as `shrink=0`.
  - Core clean-geometry propagation must not treat `width: px` plus default `flex-shrink: 1` as a
    fixed main-axis item. Negative free-space distribution is still an authoritative Taffy flex
    solve unless the shrink behavior has an explicit proof.
  - The existing single basis-zero grow proof remains unchanged: the flexible content child absorbs
    the width delta only when the fixed siblings are actually fixed/no-shrink.
- Focused gates:
  - `cargo check -p fret-ui-gallery --features gallery-full`
    - Result: passed.
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_horizontal_flex_fixed_px_default_shrink_child clean_geometry_small_resize_skips_horizontal_flex_single_basis0_grow_child clean_geometry_small_resize_skips_horizontal_flex_auto_width_no_shrink_child clean_geometry_small_resize_rejects_horizontal_flex_multiple_grow_children --no-fail-fast`
    - Result: `4/4` passed.
- Final gates:
  - `cargo fmt --check`
    - Result: passed.
  - `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`
    - Result: passed.
  - `python3 tools/check_layering.py`
    - Result: passed.
  - `git diff --check`
    - Result: passed.
- Local blocker evidence:
  - Bundle:
    `target/fret-diag/local-next-gallery-sidebar-no-shrink-clean-geometry-20260517-r1/1779010876907/bundle.schema2.json`
  - Stats:
    `target/fret-diag/local-next-gallery-sidebar-no-shrink-clean-geometry-20260517-r1/worst.stats.json`
  - Result:
    - Top frame total/layout/layout-engine-solve/prepaint/paint is `1238/602/255/244/392us` with
      `4` layout-engine solves.
    - Guardrails remain stable: `top_view_cache_roots_reused=1`,
      `top_view_cache_roots_needs_rerender=0`, row replay/store `289/0`, row replay hit rate
      `100%`.
    - The previous root `Stack` `flex_item_sizing / Flex` blocker at
      `apps/fret-ui-gallery/src/driver/render_flow.rs:336` is gone from the per-solve blockers.
    - Remaining blockers:
      - content `Semantics`: `unsupported_kind=Grid`, `wrap_nodes=1`, solve about `176-184us`;
      - root `Stack`: `unsupported_kind=TextInput`, path through
        `ecosystem/fret-ui-shadcn/src/input.rs:514`, solve about `74-80us`;
      - editor `PointerRegion`: `unsupported_kind=Canvas`, solve about `4us`;
      - root `Scroll`: `side_effect_boundary / Scroll`, solve `0us`.
- Decision:
  - Close `render_flow.rs:336` as an authoring-policy fix plus core negative guard, not as a new
    core flex distribution proof.
  - Keep RTX4090 closeout as follow-up evidence.
  - The next slice should audit either the `TextInput` blocker or the content `Grid` / wrap-flex
    line-break stability story as separate work.

TextInput side-effect boundary closeout (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`CleanGeometryNodeContract::side_effect_boundary` for `ElementInstance::TextInput`)
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_runs_text_input_layout_as_side_effect_boundary`)
  - `crates/fret-ui/src/text/input/bound.rs`
    (`BoundTextInput::layout`)
  - `crates/fret-ui/src/text/input/widget.rs`
    (`TextInput::layout`)
- Contract:
  - `TextInput` is not a pure clean-geometry leaf. Its layout observes the bound text model and
    font-stack globals, syncs model text, updates text metrics, and writes state consumed by IME /
    selection / platform text-input snapshot behavior.
  - Ancestors may treat `TextInput` as a side-effect boundary when proving a clean width-only
    geometry skip. The ancestor can skip its own Taffy root solve, but `TextInput` itself must still
    run layout when the propagated bounds change.
  - `TextArea` and `TextInputRegion` remain outside this slice. They are still side-effectful text
    surfaces and should get their own boundary proof before they participate in clean-geometry
    ancestor skips.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_runs_text_input_layout_as_side_effect_boundary --no-fail-fast`
    - Result: `1/1` passed.
- Final gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_runs_text_input_layout_as_side_effect_boundary clean_geometry_small_resize_skips_barrier_root_engine_solve clean_geometry_small_resize_rejects_horizontal_flex_fixed_px_default_shrink_child clean_geometry_small_resize_skips_horizontal_flex_single_basis0_grow_child clean_geometry_small_resize_skips_horizontal_flex_auto_width_no_shrink_child --no-fail-fast`
    - Result: `5/5` passed.
  - `cargo fmt --check`
    - Result: passed.
  - `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`
    - Result: passed.
  - `python3 tools/check_layering.py`
    - Result: passed.
  - `git diff --check`
    - Result: passed.
- Local blocker evidence:
  - Bundle:
    `target/fret-diag/local-next-text-input-boundary-clean-geometry-20260517-r1/1779012516551/bundle.schema2.json`
  - Stats:
    `target/fret-diag/local-next-text-input-boundary-clean-geometry-20260517-r1/worst.stats.json`
  - Result:
    - Top frame total/layout/layout-engine-solve/prepaint/paint is `1275/618/267/252/405us` with
      `4` layout-engine solves.
    - Guardrails remain stable: `top_view_cache_roots_reused=1`,
      `top_view_cache_roots_needs_rerender=0`, row replay/store `289/0`, row replay hit rate
      `100%`.
    - The previous root `Stack` `unsupported_kind=TextInput` blocker through
      `ecosystem/fret-ui-shadcn/src/input.rs:514` is gone from the per-solve blockers.
    - Remaining blockers:
      - content `Semantics`: `unsupported_kind=Grid`, `wrap_nodes=1`, solve about `169-183us`;
      - root `Stack`: `positioned_child / Stack`, path through the sidebar nav `ScrollArea`
        wrapper at `ecosystem/fret-ui-shadcn/src/scroll_area.rs:340`, solve about `75-79us`;
      - editor `PointerRegion`: `unsupported_kind=Canvas`, solve about `3-4us`;
      - root `Scroll`: `side_effect_boundary / Scroll`, solve `0us`.
- Decision:
  - Close the `TextInput` blocker as a side-effect boundary contract, not as a pure leaf
    propagation proof.
  - Keep RTX4090 closeout as follow-up evidence.
  - The next slice should either audit the sidebar `positioned_child / Stack` blocker or separately
    tackle content `Grid` / wrap-flex line-break stability.

Sidebar `ScrollArea` absolute overlay closeout (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`clean_absolute_px_inset_child_bounds`, `clean_inset_edge_px_or_auto`, and
    `ElementInstance::Scrollbar` as a childless propagated leaf)
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_skips_px_absolute_stack_overlay_child` and
    `clean_geometry_small_resize_rejects_fraction_absolute_stack_overlay_inset`)
  - `ecosystem/fret-ui-shadcn/src/scroll_area.rs`
    (`ScrollAreaRoot::into_element`, root `Stack` at line 340, absolute scrollbar gates and corner)
- Contract:
  - The proof is for overlay chrome only: zero-margin absolute children whose inset edges are px or
    auto, and whose axis sizes are either px or derivable from both px inset edges.
  - Absolute children with fraction/fill inset or sizing remain on the authoritative solve path
    because their percent basis is parent-size dependent and not proven by this slice.
  - The `Scroll` viewport remains a side-effect boundary and still runs layout. This slice only lets
    clean ancestors propagate the surrounding `Stack` / overlay geometry without a root Taffy solve.
  - `Scrollbar` may participate only as a childless propagated leaf. Scrollbar runtime state,
    events, and painting remain mechanism-owned by their existing host-widget paths.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_px_absolute_stack_overlay_child clean_geometry_small_resize_rejects_fraction_absolute_stack_overlay_inset --no-fail-fast`
    - Result: `2/2` passed after the mechanism change. Before the change both tests failed: the
      px-inset positive case still solved once, and the fraction-inset negative case reported the
      broader `positioned_child` rejection.
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_px_absolute_stack_overlay_child clean_geometry_small_resize_rejects_fraction_absolute_stack_overlay_inset clean_geometry_small_resize_runs_text_input_layout_as_side_effect_boundary clean_geometry_small_resize_skips_barrier_root_engine_solve clean_geometry_small_resize_does_not_skip_view_cache_root_engine_solve clean_geometry_small_resize_reports_wrap_flex_rejection_reason clean_geometry_small_resize_skips_px_container_and_updates_child_bounds clean_geometry_small_resize_skips_fixed_horizontal_flex_children --no-fail-fast`
    - Result: `8/8` passed.
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `40/40` passed.
  - `cargo fmt --check`
    - Result: passed.
  - `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`
    - Result: passed.
  - `python3 tools/check_layering.py`
    - Result: passed.
- Local blocker evidence:
  - Bundle:
    `target/fret-diag/local-next-absolute-scrollarea-clean-geometry-20260517-r1/1779014851068/bundle.schema2.json`
  - Stats:
    `target/fret-diag/local-next-absolute-scrollarea-clean-geometry-20260517-r1/worst.stats.json`
  - Result:
    - Top frame total/layout/layout-engine-solve/prepaint/paint is `1219/598/265/246/375us` with
      `4` layout-engine solves.
    - Guardrails remain stable: view-cache reused `1`, view-cache needs-rerender `0`, row
      replay/store `289/0`, row replay hit rate `100%`.
    - The previous root `Stack` `positioned_child / Stack` blocker through
      `ecosystem/fret-ui-shadcn/src/scroll_area.rs:340` is gone from the per-solve blockers.
    - Remaining blockers:
      - content `Semantics`: `Grid` with `wrap_nodes=1`, solve about `169-176us`;
      - root `Stack`: generic app-shell root solve about `88-92us`;
      - editor `PointerRegion`: `Canvas`, solve about `3-4us`;
      - root `Scroll`: side-effect boundary.
- Decision:
  - Close the sidebar absolute chrome blocker as a narrow core layout contract. No shadcn authoring
    change is needed for this blocker.
  - Keep RTX4090 closeout as follow-up evidence.
  - The next primary optimization should tackle content `Grid` / wrap-flex line-break stability as a
    separate proof. `Canvas`, root `Scroll`, and the `measured_size: Size` sentinel refactor remain
    separate follow-ups.

Card-header-like `Grid` clean-geometry closeout (2026-05-17):

- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`CleanGeometryChildBoundsStrategy::SingleColumnAutoRowsGrid`,
    `clean_single_column_auto_rows_grid_width_delta_child_bounds`,
    `clean_grid_explicit_auto_or_px_track_count`, and
    `CleanGeometrySolveSkipRejectionReason::{GridTrackSizing,GridItemSizing}`)
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_skips_card_header_like_auto_grid` and
    `clean_geometry_small_resize_rejects_flexible_grid_track`)
- Contract:
  - This is a one-column explicit-row proof, not a general grid fast path.
  - Accepted grids must have `cols == 1`, no explicit column template, explicit non-empty
    `template_rows` containing only `Auto` / `Px` tracks, a matching optional `rows` count,
    child count within the explicit rows, px padding/gaps, start alignment, static children, px
    margins, simple grid lines, and stable child width/height styles.
  - `Grid` is manual-bounds-only for clean propagation. Unsupported variants do not reuse stale
    engine local child rects.
  - Flexible tracks (`Fr` / `Flex`), item self-alignment, non-px spacing, positioned children,
    text reflow, and width-dependent height changes remain on the authoritative solve path.
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_card_header_like_auto_grid clean_geometry_small_resize_rejects_flexible_grid_track --no-fail-fast`
    - Result: `2/2` passed.
- Final gates:
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `42/42` passed.
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `153/153` passed.
  - `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`
    - Result: passed.
  - `cargo fmt --check`
    - Result: passed.
  - `python3 tools/check_layering.py`
    - Result: passed.
  - `git diff --check`
    - Result: passed.
- Local blocker evidence:
  - Bundle:
    `target/fret-diag/local-next-card-header-grid-clean-geometry-20260517-r1/1779032450806/bundle.schema2.json`
  - Stats:
    `target/fret-diag/local-next-card-header-grid-clean-geometry-20260517-r1/worst.stats.json`
  - Result:
    - Top frame total/layout/layout-roots/layout-engine-solve/prepaint/paint is
      `1214/602/399/263/234/378us` with `4` layout-engine solves.
    - Guardrails remain stable: the content view-cache reuse root is present, row replay/store is
      `289/0`, and renderer text prepare is `64us`.
    - The previous content `Semantics` `unsupported_kind=Grid` blocker is gone from the per-solve
      blockers.
    - Remaining blockers:
      - content `Semantics`: `text_reflow / Text` at
        `apps/fret-ui-gallery/src/ui/content.rs:742`, solve about `169-174us`;
      - root `Stack`: `missing_measured_size / Stack` through
        `apps/fret-ui-gallery/src/ui/nav.rs:245` and
        `ecosystem/fret-ui-shadcn/src/scroll_area.rs:340`, solve about `84-87us`;
      - editor `PointerRegion`: `unsupported_kind=Canvas`, solve about `4us`;
      - root `Scroll`: `side_effect_boundary / Scroll`, solve `0us`.
- Decision:
  - Close the `Grid` blocker as a narrow core layout contract backed by focused tests and local
    no-4090 evidence.
  - Do not treat this as permission to add a general grid skip or a text skip. The next slice should
    classify the `text_reflow / Text` blocker and the remaining root `missing_measured_size / Stack`
    blocker before implementing another fast path or authoring cleanup.

Sidebar absent-overlay `missing_measured_size` closeout (2026-05-18):

- Source and contract conclusion:
  - The content `text_reflow / Text` blocker is an intentional stop condition. The computed text
    box changes under the width delta, so the content `Semantics` solve must stay authoritative
    until a separate text/line-break stability proof exists.
  - The sidebar nav `ScrollArea` should still be authored as an explicit flex-fill slot inside the
    vertical sidebar column. It now uses
    `w_full().h_full().flex_1().min_w_0().min_h_0()` and has a gallery source guard.
  - Authoring alone was insufficient: the first local rerun still reported
    `missing_measured_size / Stack` through the sidebar `ScrollArea`.
  - The actual mechanism gap was narrower than an `Option<Size>` data-model refactor: hidden
    `ScrollArea` scrollbar/corner gates are absent `InteractivityGate` nodes with legal explicit
    `0x0` absolute geometry. The clean-geometry preflight was treating that legal zero measured
    size as if the child had never been measured.
- Mechanism anchors:
  - `crates/fret-ui/src/tree/layout/node.rs`
    (`clean_geometry_absent_interactivity_gate_leaf`,
    `clean_absolute_px_inset_child_bounds`)
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
    (`clean_geometry_small_resize_skips_absent_zero_absolute_overlay_child`)
  - `apps/fret-ui-gallery/src/ui/nav.rs`
    (`sidebar_view` nav `ScrollArea` authoring)
- Focused gates:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_absent_zero_absolute_overlay_child --no-fail-fast`
    - Result: `1/1` passed.
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_absent_zero_absolute_overlay_child clean_geometry_small_resize_skips_px_absolute_stack_overlay_child clean_geometry_small_resize_rejects_fraction_absolute_stack_overlay_inset clean_parent_geometry_skip_still_runs_scroll_layout_side_effects clean_geometry_small_resize_runs_text_input_layout_as_side_effect_boundary --no-fail-fast`
    - Result: `5/5` passed.
  - `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
    - Result: `43/43` passed.
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: `153/153` passed.
  - `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app gallery_sidebar_nav_scroll_is_explicit_flex_fill_slot -- --exact`
    - Result: `1/1` passed.
  - `cargo check -p fret-ui-gallery --features gallery-full`
    - Result: passed.
  - `cargo fmt --check`
    - Result: passed.
  - `python3 tools/check_layering.py`
    - Result: passed.
  - `git diff --check`
    - Result: passed.
- Local blocker evidence:
  - Authoring-only bundle:
    `target/fret-diag/local-next-sidebar-nav-flex-fill-clean-geometry-20260518-r1/1779034426572/bundle.schema2.json`
    - Result: top frame total/layout/solve/prepaint/paint was `1229/607/267/245/377us`, but the
      root `Stack` blocker still reported `missing_measured_size / Stack` through
      `apps/fret-ui-gallery/src/ui/nav.rs:252` and `ecosystem/fret-ui-shadcn/src/scroll_area.rs:340`.
  - Mechanism-fix bundle:
    `target/fret-diag/local-next-absent-zero-overlay-clean-geometry-20260518-r1/1779035222170/bundle.schema2.json`
    - Result: top frame total/layout/layout-engine-solve/prepaint/paint is
      `1343/671/287/270/402us` with `4` layout-engine solves.
    - Guardrails remain stable: `top_view_cache_roots_reused=1`,
      `top_view_cache_roots_needs_rerender=0`, row replay/store is `289/0`, row-scene replay hit
      rate is `100%`, and renderer text prepare is `71us`.
    - The previous sidebar `missing_measured_size / Stack` blocker is gone from per-solve
      attribution.
    - Remaining blockers:
      - content `Semantics`: `text_reflow / Text` at
        `apps/fret-ui-gallery/src/ui/content.rs:742`, solve about `179us`;
      - root `Stack`: `unsupported_kind / ViewCache` at
        `apps/fret-ui-gallery/src/driver/shell.rs:164`, solve about `91us`;
      - editor `PointerRegion`: `unsupported_kind / Canvas`, solve about `4us`;
      - root `Scroll`: `side_effect_boundary / Scroll`, solve `0us`.
- Decision:
  - Close the sidebar `missing_measured_size` blocker as a narrow absent-overlay clean-geometry
    contract plus app authoring guard.
  - Keep the broad `measured_size: Option<Size>` refactor as a future architecture lane only if
    more evidence shows the sentinel ambiguity recurring outside absent `0x0` overlays.
  - The next optimization candidate is `ViewCache` clean-geometry participation, not direct text
    reflow skipping and not root `Scroll` layout skipping.

## Current slice — Deferred probe seed vs authoritative extent

This slice locks the contract that:

- deferred probing can only happen when a retained seed extent exists,
- retained caches are seeds, not authoritative truth,
- pending probes clear only after an explicit probe or authoritative post-layout observation,
- authoritative extent commits end deferred probe state entirely instead of leaving deferred-mode
  stability bookkeeping armed for later frames.

Verified gates (2026-04-05):

- `cargo nextest run -p fret-ui authoritative_extent_commit_clears_deferred_probe_state`
  - Result: passed.
  - Contract: once a probe/observation path produces an authoritative extent, deferred invalidation
    state is fully cleared rather than leaving `kind/stable_frames` armed behind a cleared pending
    bit.
- `cargo nextest run -p fret-ui scroll_authoritative_observation_same_extent_clears_resize_deferred_state`
  - Result: passed.
  - Contract: an unchanged authoritative observation on a resize-deferred frame clears the resize
    defer state immediately, so the first stable frame does not schedule a redundant follow-up
    barrier relayout/redraw.

## Current perf slice — Engine-solved apply-path side-effect audit

This audit narrows the contract for any future apply-only fast path under scroll resize stress.

Profiling context (2026-05-08):

- Baseline prewarm bundle:
  `target/fret-diag/codex-resize-stress-scroll-child-profile-prewarm/1778225557208/bundle.schema2.json`
  - p50/p95 total: `2327/8234us`
  - p50/p95 layout: `1871/4505us`
  - p50/p95 paint: `353/3494us`
- `ui-gallery-content-viewport` can still visit roughly `1020-1044` child-layout nodes and perform
  roughly `776-1035` of them while `layout_child_max_subtree_dirty_count` is only `0-3`.
- The rejected guarded clean-subtree apply experiment worsened the current resize-stress p95
  (`total/layout/paint` from `8234/4505/3494us` to `8659/4692/3629us`), so the lane must not
  promote a broad `widget.layout` skip from that experiment.

Layout-side blacklist for apply skipping:

- `Scroll`: updates scroll handles, viewport/content size, deferred probe state, post-layout
  overflow observation, and child scroll transform during layout.
- `VirtualList`: updates virtualizer metrics, scroll handle viewport/content state, deferred
  scroll-to-item requests, visible/render ranges, measured item updates, and child scroll
  transform during layout.
- `Text`, `StyledText`, and `SelectableText`: observe font-stack globals and refresh text metrics /
  blobs / selectable text state from layout.
- `TextInput`: is a side-effect boundary for clean-geometry ancestor skips; its own layout must
  still run because it participates in focusable text-input, IME, selection, accessibility, and
  platform text-input snapshot semantics.
- `TextArea` and `TextInputRegion`: remain excluded from clean-geometry ancestor skips until they
  get dedicated side-effect boundary proofs.
- `LayoutQueryRegion`: is a queryable-bounds primitive, so skipping layout cannot be assumed safe
  without a dedicated query-snapshot proof.
- `RenderTransform` and `FractionalRenderTransform`: update retained `render_transform` state
  during layout.
- `Anchored`: resolves `anchor_element` to the live node, computes placement, writes
  `render_transform`, and optionally updates `layout_out` during layout.

Provisional safe subset candidates:

- Pure geometry / passthrough wrappers whose layout body does not write app/model/runtime state and
  only propagates already-solved child geometry, for example `Container`, `Stack`, `Flex`, `Grid`,
  `Semantics`, `FocusScope`, `ViewCache`, `ForegroundScope`, `Opacity`, `InteractivityGate`,
  `HitTestGate`, `FocusTraversalGate`, `DismissibleLayer`, `MaskLayer`, `CompositeGroup`,
  `EffectLayer`, `BackdropSourceGroup`, `Pressable`, `PointerRegion`, `HoverRegion`, `WheelRegion`,
  `InternalDragRegion`, and `ExternalDragRegion`.
- Pure leaf geometry nodes such as `Image`, `SvgIcon`, `SvgImage`, `Spinner`, `Spacer`, and
  `Scrollbar`.
- `Canvas` and `ViewportSurface` currently look leaf-like in layout, but stay provisional until
  the first apply-path proof explicitly checks renderer/resource coupling and hit-test bounds.

Decision:

- Do not implement another broad apply-only whitelist from wrapper names alone.
- Prefer a narrower dirty-frontier / scroll post-layout branch that keeps all blacklisted nodes on
  the full layout path and proves identical scroll extents, hit-test bounds, focus/IME state, and
  virtual-list visible ranges against the full `layout_in` path.

Implemented dirty-frontier slice (2026-05-08):

- Change: `Scroll` no longer promotes a clean direct child root to `Invalidation::Layout` when the
  child root's dirty work is fully covered by contained view-cache roots. The contained relayout
  pass remains responsible for consuming the cache-root dirty work, and the existing nearest-scroll
  follow-up remains responsible for observing the reconciled extent.
- Mechanism evidence:
  - `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs`
    (`node_subtree_layout_dirty_covered_by_contained_view_cache_roots`)
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
    (`forced_barrier_child_roots` filter)
  - `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
    (`scroll_contained_view_cache_dirty_does_not_force_direct_child_root_invalidation`)
- Verified gates:
  - `cargo nextest run -p fret-ui scroll_contained_view_cache_dirty_does_not_force_direct_child_root_invalidation`
    - Result: passed.
  - `cargo nextest run -p fret-ui scroll`
    - Result: passed (`147` tests).
  - `cargo nextest run -p fret-ui interactive_resize_flow_rebuild`
    - Result: passed (`4` tests).
- Perf smoke:
  - Command:
    `target\debug\fretboard-dev.exe diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-window-resize-stress-steady.json --repeat 3 --warmup-frames 5 --reuse-launch --env FRET_UI_GALLERY_BOOTSTRAP_FONTS=1 --env FRET_SCROLL_LAYOUT_PROFILE=1 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=300 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --sort time --top 15 --json --launch -- target\release\fret-ui-gallery.exe`
  - Result: passed.
  - Worst bundle:
    `target/fret-diag/1778234888082/bundle.schema2.json`
  - `diag stats --sort cpu_cycles --top 15` summary for the worst bundle:
    p50/p95 total `2875/17716us`, layout `2309/14325us`, paint `282/3103us`,
    `layout.engine_solve` p50/p95 `75/6080us`, and `contained_relayouts=0` on the top frames.
  - Interpretation: this no-prewarm smoke is not a replacement for the earlier prewarm baseline.
    It shows the representative resize-stress sample is still dominated by direct-child
    invalidation / measure / solve work, so the next slice should profile that path separately.
- View-cache perf smoke:
  - Command:
    `target\debug\fretboard-dev.exe diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-window-resize-stress-steady.json --repeat 3 --warmup-frames 5 --reuse-launch --env FRET_UI_GALLERY_BOOTSTRAP_FONTS=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_SCROLL_LAYOUT_PROFILE=1 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=300 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --sort time --top 15 --json --launch -- target\release\fret-ui-gallery.exe`
  - Result: passed.
  - Worst bundle:
    `target/fret-diag/1778235451027/bundle.schema2.json`
  - Repeat=3 p50/p95 summary: total `15359/15970us`, layout `11449/12277us`, paint
    `3765/4052us`, `layout.engine_solve` `533/2127us`.
  - Interpretation: view-cache reuse is active (`top_view_cache_roots_reused=2`), but the top
    frames still report `top_view_cache_contained_relayouts=0`. This dirty-frontier slice is
    correct but not sufficient for the representative resize-stress hot frames; the next
    optimization target is the direct-child-invalidated / resize-measure path.
- Normalized view-cache perf smoke:
  - Command:
    `target\debug\fretboard-dev.exe diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-window-resize-stress-steady.json --repeat 3 --warmup-frames 5 --reuse-launch --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json --env FRET_UI_GALLERY_BOOTSTRAP_FONTS=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_SCROLL_LAYOUT_PROFILE=1 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=300 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --sort time --top 15 --json --launch -- target\release\fret-ui-gallery.exe`
  - Result: passed.
  - Worst bundle:
    `target/fret-diag/1778235545947/bundle.schema2.json`
  - Repeat=3 p50/p95 summary: total `15276/15296us`, layout `11429/11674us`, paint
    `3649/3732us`, `layout.engine_solve` `505/2174us`.
  - Interpretation: this restores the current local normalized command form. View-cache reuse is
    active (`top_view_cache_roots_reused=2`), but top frames still report
    `top_view_cache_contained_relayouts=0`, confirming the next slice should target
    direct-child-invalidated resize churn rather than contained-relayout escalation.
- Local command drift:
  - The documented `--suite-prewarm` / `--suite-prelude` form is stale for the current local
    `fretboard-dev diag perf`; the current CLI accepts `--prewarm-script` and `--prelude-script`.
  - An initial prewarm-script run without explicit gallery bootstrap env timed out at step 0 on
    `tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json`; the normalized retry above
    passed once `FRET_UI_GALLERY_BOOTSTRAP_FONTS=1` and the view-cache envs were explicit launch
    overrides.

Implemented post-layout observation / virtual-list rerender correctness slice (2026-05-15):

- Change: scroll overflow post-layout observation now distinguishes an observed root whose bounds
  are the synthetic scroll content box from a real child-supported non-leaf extent. When that
  synthetic root is pinned to the previous content extent, validation trusts the child frontier
  instead of allowing the stale synthetic content box to keep a contracted extent authoritative.
- Change: non-retained `VirtualList` wheel-scroll visible-range escapes now notify the view-cache
  root for rerender after marking self invalidated. Retained virtual lists keep using the retained
  reconcile marker and do not notify the cache root.
- Mechanism evidence:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
    (`ScrollOverflowObservedNode::synthetic_content_extent_*`,
    `trust_scroll_overflow_nonleaf_axis(...)`)
  - `crates/fret-ui/src/declarative/host_widget/event/scroll.rs`
    (`handle_virtual_list(...)`)
  - `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
    (`scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative`,
    `scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative_at_edge`)
  - `crates/fret-ui/src/declarative/tests/virtual_list/caching.rs`
    (`virtual_list_triggers_visible_range_rerender_on_wheel_scroll_when_cached`)
  - `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs`
    (`retained_virtual_list_updates_visible_range_on_wheel_scroll_without_notifying_view_cache`)
- Verified gates:
  - `cargo nextest run -p fret-ui scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative virtual_list_triggers_visible_range_rerender_on_wheel_scroll_when_cached retained_virtual_list_updates_visible_range_on_wheel_scroll_without_notifying_view_cache --no-fail-fast`
    - Result: passed (`4` tests; includes the at-edge shrink variant).
  - `cargo nextest run -p fret-ui scroll --no-fail-fast`
    - Result: passed (`151` tests).
- Interpretation: this slice is a correctness prerequisite for the next perf attribution loop. It
  does not close the broader resize-measure objective by itself; the remaining work is still to
  run normalized `ui-resize-probes` / resize-stress attribution and decide whether another
  direct-child-invalidated / resize-measure narrowing is justified.

Normalized resize attribution sample (2026-05-15):

- Command:
  `target/release/fretboard-dev diag perf ui-resize-probes --repeat 3 --warmup-frames 5 --reuse-launch --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json --env FRET_UI_GALLERY_BOOTSTRAP_FONTS=1 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_SCROLL_LAYOUT_PROFILE=1 --env FRET_SCROLL_LAYOUT_PROFILE_MIN_US=300 --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=20 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=300 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --sort time --top 15 --json --dir target/fret-diag/scroll-optimization-v1-ui-resize-probes-20260515 --launch -- target/release/fret-ui-gallery`
- Result: passed.
- Artifacts:
  - Worst bundle:
    `target/fret-diag/scroll-optimization-v1-ui-resize-probes-20260515/1778819328814/bundle.schema2.json`
  - Layout attribution summary:
    `target/fret-diag/scroll-optimization-v1-ui-resize-probes-20260515/layout.perf.summary.v1.json`
  - Regression summary:
    `target/fret-diag/scroll-optimization-v1-ui-resize-probes-20260515/regression.summary.json`
- Suite scripts:
  - `ui-gallery-window-resize-drag-jitter-steady`: total p50/p95/max `989/1002/1002us`,
    layout `599/616/616us`, paint `343/347/347us`, layout engine solve `329/344/344us`;
    barrier relayouts `0`, contained relayouts `0`, reused cache roots `2`, and visible-range
    refresh p95 `1`.
  - `ui-gallery-window-resize-stress-steady`: total p50/p95/max `2041/2220/2220us`,
    layout `732/812/812us`, paint `1210/1270/1270us`, layout engine solve `330/377/377us`;
    barrier relayouts `0`, contained relayouts `0`, reused cache roots `2`, and visible-range
    refresh p95 `0`.
- Worst-bundle `diag stats --sort time --top 15`:
  - considered frames: `10`
  - time p50/p95 total: `236/2220us`
  - time p50/p95 layout: `91/812us`
  - time p50/p95 paint: `107/1270us`
  - hot p50/p95: `layout.engine_solve=0/377us`, `paint.widget=27/649us`,
    `paint.text_prepare=0/0us`
  - top frames: `inv.calls=0`, `barrier(set_children/scheduled/performed)=0/0/0`,
    `contained_relayouts=0`, `cache.reused=2`
- Layout attribution summary for the worst frame:
  - `layout_time_us=812`, `layout_engine_solve_time_us=377`, `layout_engine_solves=2`
  - top solves are `new_frame_key_changed` on bounded subtrees (`subtree_nodes=100` at `273us`,
    and `subtree_nodes=107` at `103us`), with `measure_time_us=0`.
- Interpretation: the normalized steady-frame sample no longer supports another
  direct-child-invalidated / resize-measure narrowing as the next high-value change. The current
  proof surface is paint-dominant at the tail, with layout solve bounded under 0.4ms and no
  invalidation-walk breadth in the considered frames. Any further resize optimization should either
  switch to a hotter proof surface or target paint/cache replay rather than widening the scroll
  apply-skip surface.

## Follow-on slice — Contained relayout dirty vs rerender semantics

This follow-on slice locks the contract that:

- contained relayout is a layout-only repair path, not an implicit “rerender next frame” request,
- `view_cache_needs_rerender` remains authoritative for declarative rerender pressure,
- scheduling-only dirty markers clear once layout invalidation and rerender pressure are both gone.

## Follow-on slice — Detached roots must not keep layout follow-up state alive

This follow-on slice locks the contract that:

- detached/unreachable cache roots must be pruned before contained-relayout candidate selection,
- detached/unreachable barrier roots must be pruned before pending barrier relayout execution,
- detached layout follow-up state must not block later stable frames from taking a layout-skip path.

## Follow-on slice — Barrier same-children fast path must still reach authoritative relayout

This follow-on slice locks the contract that:

- re-applying an unchanged barrier child list is still a no-op when the barrier subtree is clean,
- re-applying an unchanged barrier child list must schedule a contained barrier relayout when the
  barrier subtree still has pending layout work,
- descendant layout invalidations under a clean ancestor must not remain pinned just because the
  barrier child list was structurally unchanged.

## Follow-on slice — Contained cache-root dirty tracking must match authoritative layout state

This follow-on slice locks the contract that:

- a descendant layout invalidation truncated at a contained view-cache root must still make that
  root discoverable to the contained-relayout pass,
- layout-only descendant invalidations must not escalate a contained cache root into declarative
  rerender pressure,
- `dirty_cache_roots` must clear when authoritative main-pass layout already consumed the cache
  root's scheduling-only layout invalidation.

## Follow-on slice — Same-children parent repair must reconnect authoritative layout

This follow-on slice locks the contract that:

- `set_children(...same_children...)` remains a true no-op when parent pointers are already valid,
- `set_children(...same_children...)` must reconnect the parent into the authoritative layout
  invalidation walk when it repaired stale child parent pointers under pending descendant layout
  work,
- `set_children_in_mount(...same_children...)` must honor the same reconnect contract,
- `add_child(...)` must not bypass the same structural consistency contract: reparenting a child
  must sever the old parent's child edge, avoid duplicate child edges on the new parent, and route
  the resulting structural change through the authoritative layout invalidation path.

## Follow-on slice — Reparent cleanup must respect the old parent's structural policy

This follow-on slice locks the contract that:

- reparent cleanup cannot infer the old parent's detach semantics from the new write path,
- an old barrier parent must keep contained-relayout semantics when a child moves elsewhere,
- barrier-to-barrier reparent cleanup must remove stale child edges without forcing ancestor
  relayout through the old barrier,
- runtime/view-cache wrapper transitions must not leave stale parent-to-child edges that later GC
  can interpret as still-live membership.

## Follow-on slice — Layer root replacement must prune detached interaction state

This follow-on slice locks the contract that:

- replacing a layer root must immediately clear `focus` / pointer captures that are no longer
  reachable from the current active input/focus roots,
- root replacement must preserve interaction state that remains reachable from another active layer
  root (for example, an overlay that stays mounted across base-root replacement),
- input-arbitration snapshots must reflect the pruned interaction state immediately instead of
  waiting for a later dispatch/command entry point to clean it up lazily.

## Follow-on slice — Pending shortcut continuation must revalidate authoritative key contexts

This follow-on slice locks the contract that:

- multi-stroke shortcut continuation must not rely on `focus` / `barrier_root` alone as a proxy
  for routing authority,
- if root replacement or another retained-tree repair changes the current key-context stack, the
  pending shortcut must be cleared before the next chord is matched,
- stale pending shortcut key contexts must not dispatch commands after the authoritative routing
  context changed.

## Follow-on slice — Cross-surface command gating snapshots must refresh key contexts

This follow-on slice locks the contract that:

- publishing command/action availability snapshots must also refresh the current
  `WindowKeyContextStackService` snapshot,
- cross-surface command gating must not keep stale `keyctx.*` values alive after rebuild/root
  replacement or other retained-tree reconfiguration,
- app/window-scope command gating must observe the same authoritative key-context stack as the
  current UI tree rather than the last input-event snapshot.

## Follow-on slice — Declarative rebuild commits must republish authoritative window snapshots

This follow-on slice locks the contract that:

- `render_root(...)` / `render_dismissible_root_with_hooks(...)` are authoritative declarative
  rebuild commit points for window-level snapshot consumers,
- detached declarative roots may defer that commit until the returned root is actually attached to
  a parent/layer, but they must still finish the same-frame authoritative publish once attachment
  completes,
- once rebuild GC/root reuse has committed, later same-frame consumers must see refreshed
  `WindowInputContextService`, `WindowKeyContextStackService`, and
  `WindowCommandActionAvailabilityService` state,
- rebuild-time focus/key-context changes must revalidate pending shortcut state before stale
  overlay or gating consumers can observe it.

## Follow-on slice — Imperative tree mutations require explicit window snapshot commit

This follow-on slice locks the contract that:

- raw `UiTree` mutation APIs update retained tree state only; they do not silently republish
  window-level services,
- imperative mutation flows can make same-frame cross-surface consumers authoritative by calling
  `UiTree::publish_window_runtime_snapshots(...)`,
- the explicit commit surface must revalidate focus and pending shortcut/key-context state before
  writing `WindowInputContextService`, `WindowKeyContextStackService`, and
  `WindowCommandActionAvailabilityService`.

Verified gates (2026-04-05):

- `cargo nextest run -p fret-ui -E 'test(layout_all_after_imperative_tree_mutation_still_requires_explicit_window_snapshot_commit)'`
  - Result: passed.
  - Contract: `layout_all()` after a raw retained rebuild is still not an implicit authoritative
    window-snapshot commit boundary; `WindowInputContextService`,
    `WindowKeyContextStackService`, and `WindowCommandActionAvailabilityService` remain stale until
    `UiTree::publish_window_runtime_snapshots(...)` is called explicitly.

## Follow-on slice — Internal-drag target promotion ignores detached stale frame records

This follow-on slice locks the contract that:

- raw rebuilds can temporarily leave prior declarative `WindowFrame` records intact until the next
  declarative refresh,
- `Event::InternalDrag(...)` target promotion may consult retained declarative instance metadata,
  but it must only do so along the current live hit/parent chain,
- detached stale `InternalDragRegion` frame records must not hijack routing after a raw rebuild +
  `layout_all()` frame.

Verified gates (2026-04-05):

- `cargo nextest run -p fret-ui -E 'test(internal_drag_after_raw_rebuild_does_not_route_to_detached_stale_frame_region)'`
  - Result: passed.
  - Contract: even while the prior `WindowFrame` still retains an `InternalDragRegion` record after
    a raw rebuild, `dispatch/window.rs` only promotes the target through the current
    `pointer_chain_snapshot.parent` ancestry and therefore does not dispatch the drag event back
    into the detached stale region.

## Follow-on slice — Diagnostics inspect overlay remains non-authoritative for window snapshots

This follow-on slice locks the contract that:

- the diagnostics inspect overlay is input-transparent and non-authoritative for window runtime
  snapshots,
- attaching or hiding the overlay may mutate layer policy, but it must not change the final
  authoritative `WindowInputContextService`, `WindowKeyContextStackService`, or
  `WindowCommandActionAvailabilityService` state once the base UI frame republishes,
- diagnostics overlay policy should stay centralized so future changes do not accidentally widen
  it into an input barrier or snapshot owner.

Verified gates (2026-04-05):

- `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics -E 'test(inspect_overlay_render_keeps_window_runtime_snapshots_authoritative_to_base_ui)'`
  - Result: passed.
  - Contract: under a real `render_root(...) -> render_diag_inspect_overlay(...) -> layout_all() ->
    publish_window_runtime_snapshots(...)` frame sequence, the inspect overlay leaves the base UI
    authoritative for input-context, key-context, and command-availability snapshots.

## Follow-on slice — Published input-context consumers must overlay authoritative command availability

This follow-on slice locks the contract that:

- `WindowInputContextService` remains a best-effort window snapshot transport for cross-surface
  consumers,
- when consumers need `edit.can_*` / `router.can_*` semantics, `WindowCommandAvailabilityService`
  stays authoritative and must overlay the published `InputContext`,
- stale published input snapshots must not suppress command gating or shortcut lookup once the
  authoritative command-availability service has changed.

## Follow-on slice — Scroll-handle revision-only bumps must preserve baseline vs window-update semantics

This follow-on slice locks the contract that:

- runtime-driven internal scroll-handle updates still commit offset/value baselines even when they
  do not bump the public revision,
- a later revision-only bump must remain a revision-only delta at the frame registry layer rather
  than being reclassified as a fresh offset change,
- final invalidation must still downgrade revision-only bumps to `HitTestOnly` by default,
- windowed-paint scroll surfaces stay reusable on revision-only bumps, while `VirtualList`
  surfaces can still escalate to cache-root window updates when the visible window escaped the
  rendered overscan window.

## Follow-on slice — Scroll-handle invalidation must ignore detached same-frame stale bindings

This follow-on slice locks the contract that:

- scroll-handle invalidation operates on the current live attached binding set, not a multiset of
  every same-frame registration that ever happened,
- detached/dead declarative nodes that are still present in retained same-frame bookkeeping must
  not receive scroll-handle invalidations,
- detached cache roots must not be dirtied by stale same-frame bindings,
- debug scroll-handle binding samples/counts must reflect the authoritative live attached nodes
  rather than stale or duplicate registrations.

## Follow-on slice — Scroll-handle registry writes must dedupe same-frame duplicate elements

This follow-on slice locks the contract that:

- repeated same-frame declarative registrations for the same `handle_key + element` pair must not
  accumulate duplicate registry entries,
- same-frame rebuilds may append new bindings for other elements, but repeated registrations of the
  same element must keep the registry set-like for that element,
- raw registry reads used by diagnostics/tests remain stable and do not grow with duplicate
  same-frame rebuilds.

## Follow-on slice — Event-time scroll-handle invalidation resolves authoritative live bindings

This follow-on slice locks the contract that:

- widget event handlers do not treat the raw scroll-handle registry as the authoritative invalidation
  target set,
- event-time scroll-handle invalidation requests are resolved by the dispatch/runtime layer after it
  regains access to `UiTree`,
- event-time invalidation still reaches live attached bindings across active layers,
- detached stale same-frame bindings remain ignored on the event path as well as the final
  invalidation path.

## Follow-on slice — Explicit scroll-target invalidation resolves authoritative live target nodes

This follow-on slice locks the contract that:

- mechanism widgets do not resolve explicit `scroll_target` elements by directly trusting
  `window_frame.instances.find_map(...)` during event handling,
- event-time `scroll_target` invalidation is deferred until dispatch regains access to `UiTree`,
- explicit scroll-target invalidation resolves live attached target nodes only,
- detached stale same-frame target entries do not win explicit scroll-target resolution.

## Follow-on slice — Command and event focus targets resolve authoritative live attached nodes

This follow-on slice locks the contract that:

- command dispatch must not treat `window_state.node_entry(element)` as the authoritative source
  node when pending command metadata only carries an element id,
- command hooks and event-side focus hooks may request focus by element, but the live attached node
  resolution must happen in `UiTree` / dispatch after runtime regains access to the authoritative
  retained tree,
- stale detached same-frame `node_entry` seeds must not win over a still-live attached node for the
  same element.

## Follow-on slice — Declarative rebuild and invalidation element paths resolve authoritative live nodes

This follow-on slice locks the contract that:

- declarative model/global/notify invalidation paths must not treat
  `window_state.node_entry(element)` as authoritative when `UiTree` is available,
- declarative mount/root reuse must prefer the live attached node for an element and only reuse a
  retained seed when no live attached node exists,
- view-cache GC / retained virtual-list reconcile roots must ignore detached stale `node_entry`
  seeds instead of keeping them alive as authoritative rebuild roots.

## Follow-on slice — GC liveness must ignore parent-pointer-derived layer membership

This follow-on slice locks the contract that:

- declarative GC must not treat `UiTree::node_layer(...)` as an authoritative keepalive signal,
  because stale parent pointers can keep detached nodes appearing to belong to a layer,
- declarative GC liveness must be derived from explicit liveness roots plus authoritative child
  reachability (`UiTree` children and `WindowFrame` children union),
- dead retained keep-alive root `NodeId`s must be pruned before they participate in GC liveness,
  and dead root ids must not be treated as reachable by the GC walk itself,
- view-cache reuse memberships and retained keep-alive roots remain explicit GC inputs, but they do
  not widen parent-pointer-derived layer membership into truth.

Verified gates (2026-04-04):

- `cargo test -p fret-ui gc_ -- --nocapture`
  - Result: passed.

## Follow-on slice — Overlay owner pruning must use authoring-identity liveness

This follow-on slice locks the contract that:

- declarative owner ids used by ecosystem policy caches may be scope-only authoring identities that
  do not map to mounted nodes,
- current-frame owner liveness therefore cannot be inferred from `live_attached_node_for_element`
  or `node_entry(...).last_seen_frame` alone,
- view-cache reuse must restore recorded owner authoring identities for cache-hit frames where the
  producer subtree skips rerender,
- owned cached overlay requests/layers must prune only when the owner identity disappears from the
  current frame's declarative authoring pass.

Verified gates (2026-04-05):

- `cargo test -p fret-ui scope_only_authoring_identity_is_live_for_current_frame -- --nocapture`
  - Result: passed.
  - Contract: `scope` / `keyed` identities remain current-frame-live even when they do not mount a
    node themselves.
- `cargo test -p fret-ui view_cache_reuse_preserves_scope_only_authoring_identity_liveness -- --nocapture`
  - Result: passed.
  - Contract: cache-hit view-cache reuse restores recorded scope-only authoring identities without
    rerendering the producer subtree.
- `cargo test -p fret-ui-kit owned_cached_ -- --nocapture`
  - Result: passed.
  - Contract: owned cached modal / hover overlay requests prune only after the producer owner
    disappears from the current frame's authoring pass.
- `cargo test -p fret-ui-kit owned_cached_modal_request_stays_visible_during_view_cache_reuse -- --nocapture`
  - Result: passed.
  - Contract: owned cached overlay requests remain visible across cache-hit frames when the
    producer subtree is still live via view-cache reuse.
- `cargo test -p fret-ui-kit cached_ -- --nocapture`
  - Result: passed.
  - Contract: the owned-owner prune fix preserves the existing cached overlay synthesis behavior
    for modal / popover / hover / tooltip requests.
- `cargo test -p fret-ui touch_existing_subtree_can_walk_window_frame_children -- --nocapture`
  - Result: passed.

Audit note (2026-04-04):

- `crates/fret-ui/src/declarative/mount.rs` now routes both declarative GC retain callbacks through
  `gc_node_retention_decision(...)`, which no longer trusts `UiTree::node_layer(...)` as an
  authoritative keepalive shortcut.
- `crates/fret-ui/src/declarative/mount.rs` now prunes dead retained virtual-list keep-alive roots
  before computing GC liveness and makes `collect_reachable_nodes_for_gc_in_place(...)` ignore dead
  root ids, so structural removal cannot leave a stale keep-alive root widening reachability.
- `crates/fret-ui/src/declarative/mount.rs::gc_retention_ignores_stale_parent_pointer_layer_membership`
  proves the regression directly: a stale parent path can keep `node_layer(...)` non-`None` while
  `collect_reachable_nodes_for_gc(...)` still excludes the node and GC drops it once layer-root
  reachability is evaluated.
- `crates/fret-ui/src/declarative/mount.rs::gc_prunes_removed_retained_keep_alive_roots_before_reachability`
  proves the keep-alive regression directly: a removed retained keep-alive root no longer survives
  as a liveness root, and the raw GC walk ignores dead root ids.

## Follow-on slice — Runtime-owned subtree updates must refresh cache-root membership

This follow-on slice locks the contract that:

- `view_cache_elements_for_root(...)` remains an authoritative keep-alive source only if runtime-owned
  subtree mutations refresh the membership list after they update the retained subtree,
- retained virtual-list reconcile under a reused cache root must refresh ancestor cache-root
  membership lists even though the cache-root closure itself did not rerender,
- later cache-hit frames must keep touching the new visible retained rows rather than an older
  window snapshot.

Verified gates (2026-04-04):

- `cargo test -p fret-ui retained_virtual_list_host_updates_window_without_rerendering_view_cache_root -- --nocapture`
  - Result: passed.
- `cargo test -p fret-ui retained_virtual_list_ -- --nocapture`
  - Result: passed.

Audit note (2026-04-04):

- `crates/fret-ui/src/declarative/mount.rs` now refreshes ancestor cache-root membership lists
  after retained virtual-list reconcile updates the live subtree under a reused cache root.
- `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs::retained_virtual_list_host_updates_window_without_rerendering_view_cache_root`
  now asserts the cache-root membership list includes the newly visible retained rows after the
  runtime-owned window update.

## Follow-on slice — Recursive cache-root keep-alive closure must ignore stale nested roots

This follow-on slice locks the contract that:

- `view_cache_reuse_roots -> view_cache_elements_for_root` recursive keep-alive closure is only
  authoritative for elements that still resolve to a live attached node,
- a stale nested cache-root membership list must not recursively widen keep-alive closure after the
  nested root is structurally detached or otherwise no longer attached to the layer tree,
- valid live nested cache roots still remain part of the closure, so the existing nested cache-root
  membership contract stays intact.

Verified gates (2026-04-04):

- `cargo test -p fret-ui keep_alive_view_cache_membership_ignores_stale_nested_cache_roots -- --nocapture`
  - Result: passed.
- `cargo test -p fret-ui view_cache_subtree_membership_includes_nested_cache_roots -- --nocapture`
  - Result: passed.
- `cargo test -p fret-ui gc_ -- --nocapture`
  - Result: passed.

Audit note (2026-04-04):

- `crates/fret-ui/src/declarative/mount.rs` now centralizes recursive keep-alive closure
  construction and filters both top-level reuse roots and recursively discovered nested cache roots
  through live attached node resolution before treating their membership lists as authoritative.
- `crates/fret-ui/src/declarative/mount.rs::keep_alive_view_cache_membership_ignores_stale_nested_cache_roots`
  proves the regression directly: recorded membership alone is no longer sufficient to recurse into
  a stale nested cache root or keep its detached descendants alive.

## Follow-on slice — Reuse-frame membership touch must revalidate recorded members

This follow-on slice locks the contract that:

- recorded `view_cache_elements_for_root(...)` membership is only a reuse seed until every member
  revalidates against the current live attached tree,
- reuse frames must not refresh `last_seen_frame` or carry forward the recorded membership list if
  any recorded member has become stale/detached,
- invalid recorded membership must force the reuse path back onto an authoritative retained-subtree
  walk plus membership re-record, including explicit `view_cache_keep_alive(...)` reuse.

Verified gates (2026-04-05):

- `cargo test -p fret-ui view_cache_keep_alive_revalidates_recorded_membership_before_touching_stale_detached_elements -- --nocapture`
  - Result: passed.
- `cargo test -p fret-ui keep_alive_view_cache_ -- --nocapture`
  - Result: passed.
- `cargo test -p fret-ui retained_virtual_list_host_updates_window_without_rerendering_view_cache_root -- --nocapture`
  - Result: passed.
- `cargo test -p fret-ui viewport_resize_after_cache_enable_ -- --nocapture`
  - Result: passed.
- `cargo test -p fret-ui gc_ -- --nocapture`
  - Result: passed.

Audit note (2026-04-05):

- `crates/fret-ui/src/elements/runtime.rs::touch_view_cache_subtree_elements_if_recorded(...)`
  now validates every recorded member through authoritative live attached node resolution before it
  refreshes `last_seen_frame` or carries the recorded membership forward into the next frame.
- `crates/fret-ui/src/declarative/mount.rs` now passes
  `UiTree::resolve_live_attached_node_for_element_seeded(...)` into the reuse touch path, so any
  stale/detached recorded member invalidates the whole list and forces the existing retained-subtree
  walk plus membership re-record fallback.
- `crates/fret-ui/src/declarative/tests/view_cache.rs::view_cache_keep_alive_revalidates_recorded_membership_before_touching_stale_detached_elements`
  proves the regression directly: explicit keep-alive reuse no longer touches or retains a stale
  detached recorded member.

## Follow-on slice — Interaction targets resolve authoritative live attached nodes

This follow-on slice locks the contract that:

- hover/pressed/timer/selection runtime state may retain element identity across frames, but
  authoritative node resolution must happen against the live attached `UiTree` rather than by
  directly trusting a stale detached `node_entry(element)`,
- retained interaction target nodes are cache-like seeds that must be refreshed at final
  layout-frame commit, so a same-element rebuild/remount cannot keep clearing or dispatching to the
  old detached node,
- event helpers may carry `(element, node)` pairs when they already have a live dispatch target,
  but clearing or later dispatch must still consume the authoritative live node snapshot,
- selectable-text active-selection routing must keep targeting the live attached node even when
  retained selection state or `node_entry` was seeded with a stale detached node.

## Follow-on slice — Final-layout / dispatch / anchored queries resolve authoritative live attached nodes

This follow-on slice locks the contract that:

- render-time `focus-within` containment and focused-node-to-element sync are authoritative
  relation queries and must prefer the live declarative window frame before falling back to
  retained mappings,
- final-layout focus repair, touch-drag dispatch, wheel scroll-dismiss lookup, and anchored layout
  anchor-element lookup must not treat `elements::node_for_element(...)` as authoritative truth
  when `UiTree` or the declarative window frame is available,
- `elements::node_for_element(...)` remains a last-known post-frame / component-policy query
  surface; it is not the mechanism-layer source of truth for live attached nodes.

Audit note (2026-04-04):

- `rg -n "crate::elements::node_for_element\\(|elements::node_for_element\\(" crates/fret-ui/src --glob '!**/tests/**'`
  now returns no hits, so non-test mechanism paths in `crates/fret-ui` no longer resolve
  authoritative live nodes through the raw last-known query surface.

## Follow-on slice — Ecosystem runtime paths use explicit live-node query surfaces

This follow-on slice locks the contract that:

- `elements::node_for_element(...)` and `ElementContext::node_for_element(...)` remain last-known
  retained query surfaces rather than becoming implicit authoritative correctness APIs,
- `elements::live_node_for_element(...)`, `ElementContext::live_node_for_element(...)`, and
  `UiTree::live_attached_node_for_element(...)` are the public authoritative query surfaces for
  ecosystem correctness paths that genuinely need a current-frame live node,
- current-frame liveness comes from `WindowElementState::node_entry(...).last_seen_frame`, not from
  `ElementFrame::window_frame.instances`, because the declarative frame cache retains stale records
  until subtree GC,
- ecosystem overlay / focus / active-descendant helpers must resolve authoritative runtime targets
  through the explicit live query surface instead of trusting stale last-known mappings,
- render-time semantics authoring surfaces that describe parent/child relationships before the
  current frame's children mount must keep those relationships declarative (for example via
  `SemanticsDecoration::active_descendant_element(...)`) and let the semantics pass resolve the
  final mounted node after commit,
- semantics-time declarative relation resolution must prefer the local mounted element map and
  fall back to the authoritative current-frame live mapping for lazy / virtualized child subtrees.

Verified gates (2026-04-04):

- `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui-kit apply_initial_focus_prefers_explicit_element resolve_restore_focus_prefers_trigger_and_falls_back_to_live_node resolve_restore_focus_skips_non_focusable_trigger resolve_branch_nodes_dedupes_and_preserves_order active_descendant_is_set_when_active_item_is_present_and_cleared_when_missing apply_initial_focus_ignores_removed_element_with_only_last_known_mapping resolve_branch_nodes_ignores_removed_trigger_with_only_last_known_mapping active_descendant_helper_ignores_removed_element_with_only_last_known_mapping table_active_descendant_semantics_resolves_from_declarative_active_row_relation --status-level fail`
  - Result: `9 passed`.
- `CARGO_TARGET_DIR=target-codex-shadcn-lib cargo test -p fret-ui-shadcn --lib select::tests::select_mouse_hover_leave_clears_active_descendant -- --exact`
  - Result: `1 passed`.
- `CARGO_TARGET_DIR=target-codex-shadcn-lib cargo test -p fret-ui-shadcn --lib select::tests::select_label_and_separator_do_not_affect_positions_or_initial_focus -- --exact`
  - Result: `1 passed`.

Audit note (2026-04-04):

- The ecosystem authoritative runtime call sites in
  `ecosystem/fret-ui-kit/src/window_overlays/render.rs`,
  `ecosystem/fret-ui-kit/src/primitives/focus_scope.rs`,
  `ecosystem/fret-ui-kit/src/primitives/dismissable_layer.rs`,
  and `ecosystem/fret-ui-kit/src/declarative/active_descendant.rs`
  now use the explicit live query surface instead of the raw last-known query surface.
- The render-time semantics surfaces in `ecosystem/fret-ui-kit/src/declarative/table.rs` and
  `ecosystem/fret-ui-shadcn/src/select.rs` now keep `active_descendant` as a declarative
  element-to-element relationship until the semantics pass resolves the final mounted node.
- `crates/fret-ui/src/widget.rs` now lets semantics-time declarative relation resolution fall back
  from the local mounted element map to the authoritative current-frame live node mapping when a
  lazy / virtualized subtree is not represented in the local semantics element map yet.

## Canonical gates

- Seed contract regression:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_deferred_invalidation_uses_intrinsic_cache_seed_before_measure`
- Authoritative observation clears deferred invalidation pending state:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_authoritative_observation_same_extent_clears_deferred_invalidation_pending_state`
- Budget-hit recovery (growth):
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_post_layout_budget_hit_growth_converges_via_pending_probe_next_frame`
- Budget-hit recovery (shrink):
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui scroll_post_layout_budget_hit_shrink_converges_via_pending_probe_next_frame`
- Contained relayout must not force next-frame rerender:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui view_cache_contained_relayout_does_not_force_next_frame_rerender`
- Layout-invalidated definite contained roots still allow reuse:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui view_cache_layout_invalidations_allow_reuse_for_definite_contained_roots`
- Explicit scroll-handle layout invalidations still force rerender:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui view_cache_scroll_handle_layout_invalidations_mark_cache_root_needs_rerender`
- Revision-only scroll-handle bumps after internal offset sync stay classified correctly:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui scroll_handle_revision_only_bumps_after_internal_offset_updates_classify_as_layout view_cache_scroll_windowed_paint_revision_only_bump_after_internal_offset_update_stays_hit_test_only view_cache_virtual_list_revision_only_bump_after_internal_offset_update_marks_window_update`
- Detached same-frame stale scroll bindings are ignored:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui view_cache_scroll_handle_ignores_detached_same_frame_stale_bindings`
- Scroll-handle registry dedupes same-frame duplicate elements:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui scroll_handle_registry_dedupes_same_frame_duplicate_element_bindings`
- Event-time scroll-handle invalidation resolves authoritative live bindings across layers:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui event_scroll_handle_invalidation_targets_live_bindings_across_layers_only`
- Event-time explicit scroll-target invalidation resolves the live attached target:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui event_scroll_target_invalidation_prefers_live_attached_node_over_stale_same_frame_entry`
- Pending command source elements resolve the live attached node instead of a stale detached seed:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui dispatch_command_source_element_ignores_stale_detached_node_entry`
- Command hook focus requests resolve the live attached node instead of a stale detached seed:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui command_hooks_focus_request_ignores_stale_detached_node_entry`
- Key hook focus requests resolve the live attached node instead of a stale detached seed:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui key_hook_focus_request_ignores_stale_detached_node_entry`
- Pointer-region focus requests resolve the live attached node instead of a stale detached seed:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui declarative_pointer_region_focus_request_ignores_stale_detached_node_entry`
- Declarative model/global invalidation and rebuild seed resolution prefer live attached nodes:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui model_observation_invalidation_ignores_stale_detached_node_entry global_observation_invalidation_ignores_stale_detached_node_entry seeded_live_node_resolution_ignores_stale_detached_node_entry seeded_reusable_node_resolution_reuses_detached_seed_when_no_live_attached_node_exists`
- GC liveness ignores parent-pointer-derived layer membership:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui gc_prunes_removed_retained_keep_alive_roots_before_reachability gc_retention_ignores_stale_parent_pointer_layer_membership gc_reachability_unions_ui_and_window_frame_children touch_existing_subtree_can_walk_window_frame_children --status-level fail`
- Retained virtual-list window updates refresh cache-root membership without rerendering the cache root:
  - `cargo test -p fret-ui retained_virtual_list_host_updates_window_without_rerendering_view_cache_root -- --nocapture`
- Recursive keep-alive closure ignores stale nested cache roots while preserving valid nested membership:
  - `cargo test -p fret-ui keep_alive_view_cache_membership_ignores_stale_nested_cache_roots -- --nocapture`
  - `cargo test -p fret-ui view_cache_subtree_membership_includes_nested_cache_roots -- --nocapture`
- Hover/pressed/timer/selection interaction targets prefer live attached nodes over stale
  detached seeds:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui hovered_pressable_clear_uses_latest_node_for_same_element pressed_pressable_clear_uses_latest_node_for_same_element timer_dispatch_resolves_live_attached_element_target_over_stale_detached_seed final_layout_frame_syncs_hovered_pressable_node_to_live_attached_element selectable_text_set_text_selection_ignores_stale_detached_node_entry selectable_text_sets_active_text_selection`
- Render-time focus containment and focused-element sync prefer live window-frame nodes over stale
  detached seeds:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui element_context_reports_focus_within_for_focused_descendant element_context_focus_within_ignores_stale_detached_node_entries`
- Final-layout / dispatch / anchored live-node queries ignore stale detached seeds:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui focus_repair_prefers_live_attached_node_over_stale_detached_node_entry anchored_anchor_element_ignores_stale_detached_node_entry touch_pan_scroll_live_target_resolution_ignores_stale_detached_node_entry`
- Wheel scroll-dismiss lookup resolves the live attached element instead of a stale detached seed:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui dismissible_scroll_dismiss_ignores_stale_detached_node_entry`
- Detached dirty cache roots are pruned before contained relayout:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui detached_dirty_view_cache_root_is_pruned_before_layout_followups`
- Detached pending barrier relayouts are pruned before execution:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui detached_pending_barrier_relayout_is_pruned_before_layout`
- Clean barrier same-children remounts stay no-op:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui set_children_barrier_same_children_clean_subtree_stays_noop`
- Dirty barrier same-children remounts still converge via authoritative relayout:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui set_children_barrier_same_children_with_dirty_descendant_reaches_authoritative_relayout`
- Descendant layout invalidations still schedule contained relayout without rerender:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui descendant_layout_invalidation_marks_contained_view_cache_root_dirty`
- Same-children parent repair reconnects detached descendant layout:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::children::set_children_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
- Mount-time same-children parent repair reconnects detached descendant layout:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::children::set_children_in_mount_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
- `add_child(...)` reparents without stale child edges and no-ops when already attached once:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui add_child_reparents_from_old_parent_without_leaving_stale_child_edges add_child_noops_when_child_is_already_attached_once_to_same_parent`
- Barrier-parent reparent cleanup keeps contained-relayout semantics on the old parent:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui set_children_reparents_from_old_barrier_using_barrier_detach_semantics`
- Barrier-to-barrier reparent cleanup removes stale edges without bubbling ancestor relayout:
  - `CARGO_TARGET_DIR=target-codex-ui cargo nextest run -p fret-ui set_children_barrier_reparents_from_old_barrier_without_leaving_stale_child_edges`
- Runtime/cache wrapper transition keeps authoritative footer membership after compact resize:
  - `cargo test -p fret view_runtime_cache_enable_transition_keeps_toggle_group_footer_semantics_after_compact_resize -- --nocapture`
- High-fidelity todo repro keeps footer filters after compact resize:
  - `cargo test -p fret-examples todo_demo_view_runtime_cache_enable_transition_keeps_footer_filters_after_compact_resize -- --nocapture`
- Root replacement clears detached base-layer interaction state:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::layer_root_replacement::set_root_replacement_clears_detached_base_layer_interaction_state`
- Root replacement preserves still-active overlay interaction state:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::layer_root_replacement::set_root_replacement_preserves_overlay_interaction_state`
- Pending shortcut is cleared when root replacement changes the key-context stack:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::shortcuts::tests::pending_sequence_is_cleared_when_root_replacement_changes_key_contexts`
- Publishing action availability refreshes key-context snapshots for cross-surface gating:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::window_command_action_availability_snapshot::publish_snapshot_refreshes_key_context_stack_for_cross_surface_gating`
- Declarative rebuild refreshes window input snapshots before paint:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::render_root_rebuild_refreshes_window_input_context_snapshot_before_paint`
- Paint refreshes window input context after programmatic focus changes:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui tree::tests::window_input_context_snapshot::paint_all_publishes_programmatic_input_context_snapshot`
- Declarative rebuild refreshes window key-context snapshots before the next explicit publish:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::render_root_rebuild_refreshes_window_key_context_snapshot_before_next_publish`
- Declarative rebuild refreshes widget command availability before the next explicit publish:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::render_root_rebuild_refreshes_command_action_availability_before_next_publish`
- Imperative tree mutation refreshes window input context only after explicit snapshot commit:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_input_context`
- Imperative tree mutation refreshes key-context snapshots only after explicit snapshot commit:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_key_contexts`
- Imperative tree mutation refreshes widget command availability only after explicit snapshot commit:
  - `CARGO_TARGET_DIR=target-codex-check cargo nextest run -p fret-ui declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_command_availability`
- Best-effort input context overlays authoritative command availability:
  - `cargo nextest run -p fret-runtime best_effort_input_context_overlays_authoritative_command_availability`
- Best-effort input-context fallback inherits authoritative command availability:
  - `cargo nextest run -p fret-runtime best_effort_input_context_fallback_inherits_command_availability`
- Window command-gating fallback overlays authoritative command availability over stale input snapshots:
  - `cargo nextest run -p fret-runtime snapshot_for_window_overlays_authoritative_command_availability_over_stale_input_context`
- shadcn shortcut display prefers authoritative command availability over stale published input snapshots:
  - `cargo nextest run -p fret-ui-shadcn shortcut_display_input_context_prefers_authoritative_command_availability`
- Source-policy gate forbids raw window input snapshots from bypassing command-availability helpers:
  - `python3 tools/check_window_input_context_command_availability_usage.py`

## Evidence anchors

- Seed / deferred policy / authoritative commit helpers:
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
- Scroll-handle baseline commit / revision classification:
  - `crates/fret-ui/src/declarative/frame.rs`
- Live attached scroll-handle binding resolution:
  - `crates/fret-ui/src/tree/layout/state.rs`
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
- Immediate event/paint scroll-handle binding consumers:
  - `crates/fret-ui/src/declarative/host_widget/event/mod.rs`
  - `crates/fret-ui/src/declarative/host_widget/paint.rs`
- Mechanism regression coverage:
  - `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
- Final scroll-handle invalidation / window-update escalation coverage:
  - `crates/fret-ui/src/tree/layout/state.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
- Child-list mutation helper coverage:
  - `crates/fret-ui/src/tree/ui_tree_mutation/core.rs`
  - `crates/fret-ui/src/tree/tests/children.rs`
- Declarative GC liveness reachability / retain-time decisions:
  - `crates/fret-ui/src/declarative/mount.rs`
- Retained virtual-list membership refresh under cache-hit runtime-owned updates:
  - `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs`
- Recursive cache-root keep-alive closure filtering:
  - `crates/fret-ui/src/declarative/mount.rs`
- Reuse-frame recorded membership revalidation:
  - `crates/fret-ui/src/elements/runtime.rs`
  - `crates/fret-ui/src/declarative/tests/view_cache.rs`
- Best-effort window snapshot / command-availability overlay helpers:
  - `crates/fret-runtime/src/window_input_context.rs`
  - `crates/fret-runtime/src/window_command_gating/helpers.rs`
- Cross-surface consumer regression coverage:
  - `crates/fret-runtime/src/window_command_gating/tests.rs`
  - `ecosystem/fret-ui-shadcn/src/shortcut_display.rs`
- Source-policy guardrail:
  - `tools/check_window_input_context_command_availability_usage.py`
  - `.github/workflows/consistency-checks.yml`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
- Contained relayout dirty/rerender bookkeeping:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/ui_tree_view_cache.rs`
- Detached follow-up pruning + regression coverage:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
  - `crates/fret-ui/src/tree/tests/barrier_subtree_layout_dirty_aggregation.rs`
- Barrier same-children follow-up scheduling:
  - `crates/fret-ui/src/tree/ui_tree_mutation/barrier.rs`
  - `crates/fret-ui/src/tree/tests/barrier_subtree_layout_dirty_aggregation.rs`
- Contained cache-root dirty-marker lifecycle:
  - `crates/fret-ui/src/tree/layout/node.rs`
  - `crates/fret-ui/src/tree/ui_tree_invalidation_walk/mark.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
- Same-children parent repair reconnect path:
  - `crates/fret-ui/src/tree/ui_tree_mutation/core.rs`
  - `crates/fret-ui/src/tree/ui_tree_mutation/mount.rs`
  - `crates/fret-ui/src/tree/tests/children.rs`
- Old-parent structural policy for reparent cleanup:
  - `crates/fret-ui/src/tree/node_storage.rs`
  - `crates/fret-ui/src/tree/ui_tree_mutation/core.rs`
  - `crates/fret-ui/src/tree/ui_tree_mutation/barrier.rs`
  - `crates/fret-ui/src/tree/tests/children.rs`
- Runtime/high-fidelity wrapper-transition coverage:
  - `ecosystem/fret/src/view.rs`
  - `apps/fret-examples/src/todo_demo.rs`
- Layer-root replacement interaction pruning:
  - `crates/fret-ui/src/tree/layers/impls.rs`
  - `crates/fret-ui/src/tree/tests/layer_root_replacement.rs`
- Pending shortcut authoritative-context revalidation:
  - `crates/fret-ui/src/tree/dispatch/window.rs`
  - `crates/fret-ui/src/tree/shortcuts.rs`
- Cross-surface command gating key-context snapshot refresh:
  - `crates/fret-ui/src/tree/commands.rs`
  - `crates/fret-ui/src/tree/tests/window_command_action_availability_snapshot.rs`
- Declarative rebuild window-snapshot republish:
  - `crates/fret-ui/src/declarative/mount.rs`
  - `crates/fret-ui/src/tree/commands.rs`
  - `crates/fret-ui/src/declarative/tests/core.rs`
- Imperative window-snapshot commit surface:
  - `crates/fret-ui/src/tree/commands.rs`
  - `crates/fret-ui/src/tree/dispatch/window.rs`
  - `crates/fret-ui/src/tree/paint/entry.rs`
  - `crates/fret-ui/src/tree/tests/window_input_context_snapshot.rs`
  - `crates/fret-ui/src/declarative/tests/core.rs`
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Lane positioning:
  - `docs/workstreams/scroll-optimization-v1/DESIGN.md`
  - `docs/workstreams/scroll-optimization-v1/TODO.md`

## Verification notes

- 2026-04-03: compile gate confirmed with
  `CARGO_TARGET_DIR=target-codex-verify3 cargo check -p fret-ui --tests`.
- 2026-04-03: dedicated test binary linked successfully with
  `CARGO_TARGET_DIR=target-codex-verify3 cargo test -p fret-ui --lib --no-run`.
- 2026-04-03: targeted execution gates confirmed via
  `target-codex-verify3/debug/deps/fret_ui-c0a3056b7a68a9e7 --exact ...`:
  - `declarative::tests::layout::scroll::scroll_deferred_invalidation_uses_intrinsic_cache_seed_before_measure`
  - `declarative::tests::layout::scroll::scroll_authoritative_observation_same_extent_clears_deferred_invalidation_pending_state`
  - `declarative::tests::layout::scroll::scroll_post_layout_budget_hit_growth_converges_via_pending_probe_next_frame`
  - `declarative::tests::layout::scroll::scroll_post_layout_budget_hit_shrink_converges_via_pending_probe_next_frame`
- 2026-04-03: follow-on contained-relayout gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-verify4`:
  - `tree::tests::view_cache::view_cache_contained_relayout_does_not_force_next_frame_rerender`
  - `tree::tests::view_cache::view_cache_runs_contained_relayout_for_invalidated_boundaries`
  - `tree::tests::view_cache::view_cache_layout_invalidations_allow_reuse_for_definite_contained_roots`
  - `tree::tests::view_cache::view_cache_scroll_handle_layout_invalidations_mark_cache_root_needs_rerender`
- 2026-04-03: revision-only scroll-handle classification gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-ui`:
  - `declarative::frame::tests::scroll_handle_revision_only_bumps_after_internal_offset_updates_classify_as_layout`
  - `tree::tests::view_cache::view_cache_scroll_windowed_paint_revision_only_bump_after_internal_offset_update_stays_hit_test_only`
  - `tree::tests::view_cache::view_cache_virtual_list_revision_only_bump_after_internal_offset_update_marks_window_update`
  - `tree::tests::view_cache::view_cache_scroll_handle_window_update_marks_cache_root_needs_rerender`
  - `tree::tests::view_cache::view_cache_scroll_windowed_paint_marks_cache_root_needs_rerender`
- 2026-04-04: live-binding filtering for same-frame stale scroll registrations confirmed via
  `cargo nextest` with `CARGO_TARGET_DIR=target-codex-ui`:
  - `tree::tests::view_cache::view_cache_scroll_handle_ignores_detached_same_frame_stale_bindings`
  - `tree::tests::view_cache::view_cache_scroll_windowed_paint_revision_only_bump_after_internal_offset_update_stays_hit_test_only`
  - `tree::tests::view_cache::view_cache_virtual_list_revision_only_bump_after_internal_offset_update_marks_window_update`
  - `tree::tests::view_cache::view_cache_scroll_handle_window_update_marks_cache_root_needs_rerender`
  - `tree::tests::view_cache::view_cache_scroll_windowed_paint_marks_cache_root_needs_rerender`
  - `declarative::frame::tests::scroll_handle_revision_only_bumps_after_internal_offset_updates_classify_as_layout`
- 2026-04-04: same-frame duplicate scroll binding registrations dedupe correctly via
  `cargo nextest` with `CARGO_TARGET_DIR=target-codex-ui`:
  - `declarative::frame::tests::scroll_handle_registry_dedupes_same_frame_duplicate_element_bindings`
  - `tree::tests::view_cache::view_cache_scroll_handle_ignores_detached_same_frame_stale_bindings`
  - `tree::tests::view_cache::view_cache_scroll_windowed_paint_revision_only_bump_after_internal_offset_update_stays_hit_test_only`
  - `tree::tests::view_cache::view_cache_virtual_list_revision_only_bump_after_internal_offset_update_marks_window_update`
- 2026-04-03: detached-root follow-up pruning gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-verify5`:
  - `tree::tests::view_cache::detached_dirty_view_cache_root_is_pruned_before_layout_followups`
  - `tree::tests::barrier_subtree_layout_dirty_aggregation::detached_pending_barrier_relayout_is_pruned_before_layout`
  - `tree::tests::view_cache::view_cache_runs_contained_relayout_for_invalidated_boundaries`
- 2026-04-03: barrier same-children follow-up gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-verify6`:
  - `tree::tests::barrier_subtree_layout_dirty_aggregation::set_children_barrier_same_children_clean_subtree_stays_noop`
  - `tree::tests::barrier_subtree_layout_dirty_aggregation::set_children_barrier_same_children_with_dirty_descendant_schedules_barrier_relayout`
  - `tree::tests::barrier_subtree_layout_dirty_aggregation::set_children_barrier_same_children_with_dirty_descendant_reaches_authoritative_relayout`
  - `tree::tests::view_cache::view_cache_contained_relayout_does_not_force_next_frame_rerender`
  - `tree::tests::view_cache::view_cache_runs_contained_relayout_for_invalidated_boundaries`
  - `tree::tests::view_cache::view_cache_layout_invalidations_allow_reuse_for_definite_contained_roots`
  - `tree::tests::view_cache::view_cache_scroll_handle_layout_invalidations_mark_cache_root_needs_rerender`
- 2026-04-03: contained cache-root dirty-marker lifecycle gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-verify7`:
  - `tree::tests::view_cache::view_cache_invalidation_stops_at_boundary_for_paint`
  - `tree::tests::view_cache::descendant_layout_invalidation_marks_contained_view_cache_root_dirty`
  - `tree::tests::view_cache::view_cache_runs_contained_relayout_for_invalidated_boundaries`
  - `tree::tests::view_cache::view_cache_contained_relayout_does_not_force_next_frame_rerender`
  - `tree::tests::view_cache::view_cache_layout_invalidations_allow_reuse_for_definite_contained_roots`
  - `tree::tests::view_cache::view_cache_scroll_handle_layout_invalidations_mark_cache_root_needs_rerender`
  - `tree::tests::view_cache::detached_dirty_view_cache_root_is_pruned_before_layout_followups`
- 2026-04-04: old-parent structural policy gates confirmed via targeted tree `cargo nextest`
  plus runtime/high-fidelity `cargo test`:
  - `tree::tests::children::set_children_reparents_from_old_barrier_using_barrier_detach_semantics`
  - `tree::tests::children::set_children_barrier_reparents_from_old_barrier_without_leaving_stale_child_edges`
  - `view::tests::view_runtime_cache_enable_transition_keeps_toggle_group_footer_semantics_after_compact_resize`
  - `todo_demo::tests::todo_demo_view_runtime_cache_enable_transition_keeps_footer_filters_after_compact_resize`
- 2026-04-03: same-children parent-repair reconnect gates confirmed via `cargo nextest` with
  `CARGO_TARGET_DIR=target-codex-verify8`:
  - `tree::tests::children::set_children_noops_when_unchanged`
  - `tree::tests::children::set_children_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
  - `tree::tests::children::set_children_in_mount_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
  - `tree::tests::barrier_subtree_layout_dirty_aggregation::*`
  - `tree::tests::view_cache::*` targeted contained-relayout gates
- 2026-04-03: remaining child-list mutation helper audit closed with `add_child(...)` now routed
  through the same authoritative child-list contract via `cargo nextest`:
  - `tree::tests::children::add_child_reparents_from_old_parent_without_leaving_stale_child_edges`
  - `tree::tests::children::add_child_noops_when_child_is_already_attached_once_to_same_parent`
  - `tree::tests::children::set_children_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
  - `tree::tests::children::set_children_in_mount_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout`
- 2026-04-03: layer-root replacement interaction-pruning gates confirmed via `cargo nextest`:
  - `tree::tests::layer_root_replacement::set_root_replacement_clears_detached_base_layer_interaction_state`
  - `tree::tests::layer_root_replacement::set_root_replacement_preserves_overlay_interaction_state`
  - `tree::tests::window_input_arbitration_snapshot::dispatch_event_publishes_post_dispatch_input_arbitration_snapshot`
  - `tree::tests::window_input_arbitration_snapshot::dispatch_command_publishes_post_dispatch_input_arbitration_snapshot`
  - `tree::tests::window_input_arbitration_snapshot::modal_barrier_scopes_pointer_capture_to_active_roots`
  - `tree::tests::semantics_focus_shortcuts::remove_layer_uninstalls_overlay_and_removes_subtree`
- 2026-04-03: pending-shortcut authoritative-context revalidation gates confirmed via `cargo nextest`:
  - `tree::shortcuts::tests::pending_sequence_is_cleared_when_root_replacement_changes_key_contexts`
  - `tree::shortcuts::tests::pending_sequence_matches_reserved_second_chord_before_text_input_consumes`
  - `tree::tests::command_enabled_service::shortcut_dispatch_respects_window_command_enabled_service`
  - `tree::tests::command_enabled_service::shortcut_dispatch_respects_window_command_action_availability_snapshot`
  - `tree::tests::command_enabled_service::focus_menu_bar_shortcut_dispatches_when_menu_bar_focus_service_is_present`
- 2026-04-03: cross-surface command-gating key-context refresh gates confirmed via `cargo nextest`:
  - `tree::tests::window_command_action_availability_snapshot::publish_snapshot_refreshes_key_context_stack_for_cross_surface_gating`
  - `tree::tests::window_command_action_availability_snapshot::action_availability_snapshot_marks_unhandled_commands_unavailable`
  - `tree::tests::window_command_action_availability_snapshot::action_availability_snapshot_publishes_focus_traversal_gating`
  - `tree::tests::window_command_action_availability_snapshot::action_availability_snapshot_publishes_focus_menu_bar_gating`
  - `tree::tests::window_command_action_availability_snapshot::dispatch_event_publishes_action_availability_snapshot`
- 2026-04-03: declarative rebuild window-snapshot republish gates confirmed via `cargo nextest`:
  - `declarative::tests::core::render_root_rebuild_refreshes_window_input_context_snapshot_before_paint`
  - `declarative::tests::core::render_root_rebuild_refreshes_window_key_context_snapshot_before_next_publish`
  - `declarative::tests::core::render_root_rebuild_refreshes_command_action_availability_before_next_publish`
  - `tree::shortcuts::tests::pending_sequence_is_cleared_when_root_replacement_changes_key_contexts`
  - `tree::tests::window_command_action_availability_snapshot::publish_snapshot_refreshes_key_context_stack_for_cross_surface_gating`
  - `tree::tests::window_input_context_snapshot::dispatch_event_publishes_post_dispatch_input_context_snapshot`
  - `tree::tests::window_input_context_snapshot::dispatch_command_publishes_post_dispatch_input_context_snapshot`
  - `tree::tests::window_input_arbitration_snapshot::dispatch_event_publishes_post_dispatch_input_arbitration_snapshot`
  - `tree::tests::window_input_arbitration_snapshot::dispatch_command_publishes_post_dispatch_input_arbitration_snapshot`
- 2026-04-03: imperative window-snapshot commit gates confirmed via `cargo nextest`:
  - `declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_input_context`
  - `declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_key_contexts`
  - `declarative::tests::core::imperative_tree_mutation_requires_explicit_window_snapshot_commit_for_command_availability`
  - `declarative::tests::core::render_root_rebuild_refreshes_window_input_context_snapshot_before_paint`
  - `declarative::tests::core::render_root_rebuild_refreshes_window_key_context_snapshot_before_next_publish`
  - `declarative::tests::core::render_root_rebuild_refreshes_command_action_availability_before_next_publish`
  - `tree::shortcuts::tests::pending_sequence_is_cleared_when_root_replacement_changes_key_contexts`
  - `tree::tests::window_command_action_availability_snapshot::publish_snapshot_refreshes_key_context_stack_for_cross_surface_gating`
  - `tree::tests::window_command_action_availability_snapshot::dispatch_event_publishes_action_availability_snapshot`
  - `tree::tests::window_input_context_snapshot::dispatch_event_publishes_post_dispatch_input_context_snapshot`
  - `tree::tests::window_input_context_snapshot::dispatch_command_publishes_post_dispatch_input_context_snapshot`
  - `tree::tests::window_input_context_snapshot::paint_all_publishes_programmatic_input_context_snapshot`
- 2026-04-03: best-effort input-context authoritative-overlay runtime gates confirmed via `cargo nextest`:
  - `best_effort_input_context_overlays_authoritative_command_availability`
  - `best_effort_input_context_fallback_inherits_command_availability`
  - `snapshot_for_window_overlays_authoritative_command_availability_over_stale_input_context`
- 2026-04-03: source-policy guardrail added so raw `WindowInputContextService` reads cannot feed
  command/shortcut consumers or own `edit.can_*` / `router.can_*` truth outside the runtime
  publisher/helper allowlist:
  - `python3 tools/check_window_input_context_command_availability_usage.py`
  - `.github/workflows/consistency-checks.yml`
- 2026-04-03: remaining raw `WindowInputContextService` consumers audited after the
  command-availability overlay refactor:
  - runtime owner/publisher sites remain in `crates/fret-ui/src/tree/commands.rs` and
    `crates/fret-runtime/src/window_input_context.rs`,
  - diagnostics readers in `ecosystem/fret-bootstrap/src/ui_diagnostics/{service.rs,script_steps_wait.rs,script_steps_assert.rs,script_steps_visibility.rs,script_steps_drag.rs}`
    use the snapshot only for window liveness, `focus_is_text_input`, and platform capability
    predicates, not command-availability truth,
  - text/IME readers in `ecosystem/fret-code-editor/src/editor/mod.rs` and
    `apps/fret-ui-gallery/src/ui/previews/pages/editors/web_ime.rs` use the snapshot only for
    `text_boundary_mode` / `focus_is_text_input`,
  - no remaining first-party command/shortcut consumers bypass the runtime helper overlay; the
    source-policy gate now enforces that boundary.
