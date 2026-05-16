---
title: UI Performance: Zed-level Smoothness v1 (TODO)
status: draft
date: 2026-02-02
scope: performance, profiling, data-structures, caching, input, layout, paint
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed
- egui: https://github.com/emilk/egui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# UI Performance: Zed-level Smoothness v1 (TODO)

This file tracks milestones and concrete tasks for:

- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1.md`
- GPUI/Zed gap reference (mechanism-first): `docs/workstreams/standalone/ui-perf-gpui-gap-v1.md`

Conventions:

- “Contract” items should land with an ADR (or an update to an existing ADR).
- “Perf gate” items should land with a runnable `fretboard-dev diag perf` command and a baseline/threshold update.
- “Fearless refactor” items should include: (1) perf evidence, (2) correctness evidence, (3) rollback plan.

## Current local checkpoint (updated 2026-05-16; Windows RTX4090 deferred)

The target-machine Windows RTX4090 editor-paint closeout remains the formal contract gate, but it is not the
current local execution blocker. Continue baseline-neutral local work only when it has its own evidence and does
not update checked-in baselines.

- [ ] Complete the formal Windows RTX4090 editor-paint closeout when the target machine is available.
  - Required shape: run the validation directory without `--allow-non-windows`, run the attribution directory with
    `--with-paint-perf`, then pass artifact verifier and closeout without `--allow-non-windows`.
  - Preferred runner: `tools/perf/diag_editor_paint_contract_windows_handoff.py`
  - This remains a TODO and must not be replaced by local macOS evidence.
- [x] Keep a one-command target-machine handoff runner for the deferred Windows RTX4090 closeout.
  - Runner: `tools/perf/diag_editor_paint_contract_windows_handoff.py`
  - Sequence: release builds, preflight, baseline validation, `--with-paint-perf` attribution validation, artifact
    verifier, and closeout gates. Use `--skip-build` only when the Windows target binaries are already current.
  - Release build and preflight steps are fatal prerequisites; validation does not run if they fail.
  - Dry-run evidence:
    `target/fret-diag/editor-paint-contract-windows-handoff-workstream-gate/handoff-plan.json`
  - Host guard evidence: non-dry-run on this macOS host exits with
    `the editor paint contract handoff must run on the target Windows host`.
- [x] Emit an explicit closeout owner decision from verified attribution artifacts.
  - Surface: `tools/perf/diag_editor_paint_contract_closeout.py` now writes `owner_decision` into the closeout summary.
  - Outcomes: `canvas-paint-replay`, `renderer-text-prepare`, or `no-code-change`; failed/missing artifacts produce
    `status=incomplete` and no owner.
  - Negative evidence:
    `target/fret-diag/editor-paint-contract-validate-20260516-goal-audit/editor-paint-contract-closeout.after-owner-decision.summary.json`
- [x] Run a baseline-neutral local editor-paint contract validation/attribution pass while Windows RTX4090 is deferred.
  - Tooling: `diag_editor_paint_contract_validate.py` and `diag_resize_probes_gate.py` accept a full `--launch-cmd`;
    verifier and closeout accept explicit `--allow-non-windows` for local triage only.
  - Validation evidence:
    `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo/summary.json`
  - Attribution evidence:
    `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo-attrib/summary.json`
  - Verified local closeout:
    `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo/editor-paint-contract-closeout.summary.json`
  - Result: all three probes passed their checked threshold reports on this local macOS M4 run, and local closeout
    selected `owner=canvas-paint-replay` with complex-wheel `paint_widget_p95_us=509`,
    `canvas_exclusive_p95_us=407`, and highest `renderer_prepare_text_p95_us=69`.
  - Decision: do not start another renderer text/glyph residency slice from current evidence. The next local
    optimization owner is Canvas/paint replay, while the formal Windows RTX4090 closeout remains a TODO.
- [x] Re-run the three local editor paint probes after any host-widget, renderer-text, or paint-cache cleanup before
  widening the implementation lane.
  - Probes:
    - `tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-typical.json`
    - `tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.json`
    - `tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`
  - Required local evidence: repeat=3, warmup=5, standard prewarm/prelude hooks, overlay disabled, `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`,
    and `fretboard-dev diag stats --sort time --top 15 --json` for the worst bundle.
  - Do not treat this as Windows closeout or baseline promotion evidence.
  - Latest local pass (2026-05-16, macOS M4; baseline-neutral):
    - typical autoscroll: `target/fret-diag/local-next-editor-paint-20260516-after-no4090-typical-r3/1778928598441/bundle.schema2.json`
    - complex wheel: `target/fret-diag/local-next-editor-paint-20260516-after-no4090-complex-wheel-r3/1778928659800/bundle.schema2.json`
    - resize jitter: `target/fret-diag/local-next-editor-paint-20260516-after-no4090-resize-jitter-r3/1778928717171/bundle.schema2.json`
  - Result:
    - autoscroll p95: total `654 -> 682us`, paint `416 -> 451us`, `renderer_prepare_text_us 338 -> 35us`,
      `renderer_prepare_text_collect_pin_keys_us 325 -> 22us`
    - complex wheel p95: total `886 -> 808us`, paint `687 -> 662us`, `renderer_prepare_text_us 341 -> 47us`,
      `renderer_prepare_text_collect_pin_keys_us 324 -> 29us`
    - resize jitter p95: total `1501 -> 1689us`, layout `851 -> 862us`, `layout_engine_solve_time_us 399 -> 412us`,
      `renderer_prepare_text_collect_pin_keys_us 347 -> 62us`
  - Decision: the renderer text collector is no longer the dominant local owner on these probes. Keep the text
    collector refactor as the landed local slice and move the next optimization discussion to resize layout roots /
    solve batching, not another scene-text scan rewrite.
- [x] Collapse visible row-rect generation into a windowed-row iterator before considering a broader surface rewrite.
  - Change: `WindowedRowsPaintFrame::row_rects(...)` now generates visible row rectangles incrementally and is used by
    `paint_windowed_rows(...)` plus code-editor row-scene prepaint planning.
  - Gate:
    `cargo nextest run -p fret-ui-kit windowed_rows_frame_row_rects_iterates_visible_rows --no-fail-fast`
    and
    `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`.
  - Local no-4090 evidence:
    `target/fret-diag/local-next-editor-paint-20260516-row-rect-iter-typical-r3/1778932552262/bundle.schema2.json`.
  - Result: the local typical bundle's `code_editor_paint_perf.p95.us_total` moved `138 -> 119us`,
    `us_windowed_surface_paint_callback` moved `176 -> 160us`, and `us_windowed_surface_row_paint` moved
    `158 -> 141us`; the row replay shape stayed stable with `289` rows replayed and `0` rows stored.
  - Decision: this is a small fixed-loop cleanup, not evidence for a broad windowed-surface, paint-cache, or
    display-list rewrite. Later local contract attribution superseded the resize-first discussion and selected
    Canvas/paint replay as the next local owner; keep Windows RTX4090 closeout as TODO.
- [x] Attribute the remaining local paint/renderer split before another code change.
  - Latest typical-only local smoke:
    `target/fret-diag/paint-observed-deps-presence-snapshot-typical-r3/1778921262429/stats.json`.
  - Current p95 total/paint/paint-widget is `673/423/263us`.
  - Host-widget observed model/global replay is now `4/4us` p95 after the paint-pass presence snapshot.
  - Remaining host-widget instance lookup is `42us` p95 across `252` calls; top sampled frames show visual-bounds
    record/flush around `9..14us` and paint-cache key construction around `26..28us`.
  - Renderer text/encode/upload remains visible at `324/150/85us` p95 in the same local smoke.
  - Decision: do not start a broad `ElementHostWidget` or Canvas display-list rewrite from this evidence alone.
    The next implementation owner should be chosen only after the three-probe local pass confirms whether the
    residual is renderer text prepare, paint-cache/visual-bounds bookkeeping, or generic paint traversal noise.
- [x] If renderer text prepare stays dominant across the three probes, add a narrow attribution slice before optimizing.
  - Split `TextSystem::prepare_for_scene(...)` into scene pin-key collection, bucket delta, prewarm, and upload flush
    timing.
  - Implementation surface: renderer frame perf snapshots, UI diagnostics frame stats, and `fretboard-dev diag stats`
    now expose renderer text prepare subphase timings and glyph/blob counts.
  - Smoke evidence:
    `target/fret-diag/renderer-text-prepare-subphase-typical-smoke/1778924874759/bundle.schema2.json`.
  - Result: local typical p95 shows `renderer_prepare_text_us=339us`, with
    `renderer_prepare_text_collect_pin_keys_us=326us`; bucket delta is only `13us`, prewarm/pin update/flush upload
    are `0us`, and the top sampled frame reports `text_blobs=341`, `pinned_glyph_keys=322`, `retained_glyph_keys=322`,
    `added_glyph_keys=0`, and `removed_glyph_keys=14`.
  - Candidate follow-up only after attribution: reuse or fingerprint the per-scene text pin set across stable
    row-replay frames, preserving atlas pin lifetime semantics.
  - Do not change renderer payload thresholds from local macOS evidence.
- [x] Optimize renderer text prepare only if the pin-key collection finding reproduces after the attribution slice lands.
  - Candidate owner: `TextSystem::collect_scene_pinned_keys(...)` used to rebuild a `GlyphKeyBuckets` aggregate from
    `Scene::text_blob_ids()` each frame even when row replay made the text blob set mostly stable.
  - Implemented slice: a scene blob-id cache now tracks blob presence and per-blob pin-key refcounts, then updates the
    aggregate incrementally instead of rescanning every blob shape on every frame.
  - Repro/gate: reran the three local editor probes above plus the focused `fret-render-wgpu` atlas/reset regression
    test.
  - Result: the cached collector cut `renderer_prepare_text_collect_pin_keys_us` from `325/324/347us` p95 on the
    original probes to `22/29/62us` p95 on the rerun probes. The resize probe still stays layout-dominant, so it is a
    separate follow-on rather than a renderer-text problem.
  - Keep any checked-in baseline change blocked on the Windows RTX4090 contract pass.
- [ ] If paint-cache / visual-bounds bookkeeping is the local residual, optimize only the measured subphase.
  - Candidate low-risk paths: avoid redundant visual-bounds writes, reuse small paint-cache key inputs, or reduce
    per-node text-style fingerprint lookups when the node kind cannot carry inherited text style.
  - Avoid replacing `WindowFrame.instances` or cloning semantics in this slice unless the instance lookup p95 grows
    materially above the current `~42us` aggregate.
  - [x] Make the residual measurable from root-level `diag stats --json` summaries before changing behavior.
    - Surface: p50/p95/max now include `paint_cache_key_time_us`, `paint_cache_hit_check_time_us`,
      `paint_record_visual_bounds_time_us`, `paint_record_visual_bounds_calls`, and
      `paint_observation_record_time_us`.
    - Local read: the latest no-4090 editor probes keep paint-cache key construction around `25..29us` in the top
      frames and visual-bounds recording around `8..15us`; this is useful attribution, but not yet evidence for a
      broad paint-cache or visual-bounds rewrite.
- [ ] Keep structural refactors as separate follow-ons rather than reopening P1.5.
  - Split a new narrow workstream only for hard structural changes such as true FrameArena/bump allocation,
    `WindowFrame.children` arena/slab storage, explicit view-cache paint-skip semantics, or an editor row-fragment
    replay contract.
  - Do not split a new workstream for small local host-widget or renderer-text attribution slices.
- [x] Start a narrow Canvas/paint replay investigation before any broad renderer or UI-tree rewrite.
  - First evidence target: use the verified local attribution artifacts above and inspect why complex-wheel
    `canvas_exclusive_p95_us` remains near `407us` while renderer text prepare is below `70us`.
  - Candidate surfaces: Canvas command replay, paint widget hotspot accounting, row-scene replay boundaries, visual
    bounds recording, and paint-cache key construction.
  - Finding: the complex-wheel hotspot was not pure renderer/Canvas cost. Inline preedit caused
    `replay_row_scene_plan_candidates_for_frame(...)` to return before planning any visible row, forcing the whole
    editor surface onto paint-time row-scene probes.
  - Fix: keep only the preedit/caret row on the paint-time path and allow unrelated rows to use retained prepaint
    row-scene replay.
  - Local no-4090 evidence:
    `target/fret-diag/local-next-editor-paint-20260516-preedit-row-plan-complex-wheel-r3/worst.stats.json`.
  - Result versus the previous local `prepaint-both-edges` complex-wheel run:
    `rows_scene_prepaint_planned` moved `0 -> 288`, `rows_scene_prepaint_skip_preedit` moved `0 -> 1`,
    code-editor `us_total` p95 moved `383 -> 111us`, windowed-surface paint callback p95 moved `414 -> 151us`,
    Canvas exclusive p95 moved `419 -> 152us`, and frame paint p95 moved `679 -> 427us`.
  - Tradeoff: prepaint p95 moved `122 -> 268us`; the next measured slice should reduce prepaint planning cost or
    introduce a coarser row-fragment replay contract.
  - Keep this as a measured slice; do not change baselines from local macOS evidence.
- [ ] Reduce the residual prepaint row-scene planning cost after the inline-preedit replay recovery.
  - First target: split `us_row_scene_prepaint_plan` into cache probe/key-compare/resource touch/replay-fragment
    preparation so the next change is attributable.
  - Candidate structural direction: a row-fragment replay contract that lets prepaint hand paint a contiguous retained
    fragment plan without per-row cache/probe bookkeeping in the hot paint callback.
  - Do not widen this into a renderer rewrite unless renderer prepare/encode becomes dominant in the local and
    Windows RTX4090 evidence.

## Current priorities (updated 2026-02-08)

- [x] Keep an explicit perf contract matrix for editor-grade probes.
  - Matrix: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md`
  - Scope: representative scripts, checked-in baselines, gate commands, recent evidence, and Zed/GPUI plus egui
    reference pressure.
  - Note: new `diag perf --perf-baseline-out` rows record `measured_p50`; old baselines remain valid and should only
    gain p50 when intentionally re-seeded.

- [ ] Pause checkpoint (2026-02-10): consolidate and avoid new experiments unless a gate regresses.
  - Summary + rollback switches: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1.md` (“Checkpoint (2026-02-10)”).
  - Evidence anchors:
    - small-step max `dw` widening: perf log entry `2026-02-09 22:54:20` (commit `53aa6534a`)
    - wrap-from-unwrapped allocation win: perf log entry `2026-02-09 22:12:02` (commit `7b9a98a8f`)
    - non-landed experiment example: perf log entry `2026-02-10 00:18:40` (sticky small-step)
  - Maintenance tasks (keep this workstream “ready to resume”):
    - [x] Re-run `ui-resize-probes` + `ui-code-editor-resize-probes` gates after any large merge/refactor and record the
      no-code-change evidence in the perf log.
    - [ ] If `ui-resize-probes` becomes flaky again, cut a new baseline via `tools/perf/diag_perf_baseline_select.sh`.

- [ ] Linux editor-grade perf evidence.
  - Status: blocked for formal contract closure until a real Linux runner/profile exists.
    A smoke-only `linux-local` export exists at
    `docs/workstreams/perf-baselines/ui-code-editor-resize-probes.linux-local.v1.json`, but it is
    repeat=1/max-only evidence and not a contract baseline.
  - Goal: add a checked-in Linux baseline for the editor-grade probes that already close on Windows/macOS, and keep
    Linux evidence labeled separately from Windows/macOS contracts until then.
  - Evidence anchors: `docs/code-editor.md`; `docs/workstreams/code-editor-public-api-and-architecture-v1/M5_PERF_CONTRACT_CLOSURE_2026-05-12.md`;
    `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-audit.md`.

- Representative daily smoke set (local, not CI yet):
  - `tools/diag-scripts/ui-gallery/perf/ui-gallery-dialog-escape-focus-restore-steady.json`
  - `tools/diag-scripts/ui-gallery/perf/ui-gallery-context-menu-right-click-steady.json`
  - `tools/diag-scripts/ui-gallery/perf/ui-gallery-material3-tabs-switch-perf-steady.json`
  - Use this trio as the default “is the frame still good?” loop; keep full `ui-gallery-steady` for periodic
    drift evidence or suite membership changes, not as a single formal Windows contract.
  - Do not try another Windows `ui-gallery-steady` promotion by loosening thresholds; treat the broad suite as drift
    evidence unless it is redefined as a suite-of-contracts.
  - `ui-gallery-hover-layout-torture-steady` now has its own Windows v1 baseline plus a `diag stats
    --check-hover-layout-max 0` semantic gate and no longer belongs to the broad-only member list.
  - `ui-gallery-material3-tabs-switch-perf-steady` now has its own Windows v1 baseline and no longer belongs to the
    broad-only member list.
  - `ui-gallery-menubar-keyboard-nav-steady` now has its own Windows v1 baseline and no longer belongs to the
    broad-only member list.
  - `ui-gallery-view-cache-toggle-perf-steady` now has its own Windows v1 baseline and no longer belongs to the
    broad-only member list.
  - `ui-gallery-virtual-list-torture-steady` now has its own Windows v1 baseline and no longer belongs to the
    broad-only member list.
  - Evidence: perf log entries `2026-05-07 13:58` and `2026-05-07 14:01`.

- [x] Stabilize `ui-gallery-overlay-pointer-move-steady` cleanup after pointer sweeps.
  - Change: re-enter `ui-gallery-overlay-underlay` and wait one frame before the outside-press cleanup click, because
    the sweep intentionally traverses past the 1280px test window and can leave the pointer outside the hit-test surface.
  - Gate: `cargo nextest run -p fret-ui-gallery overlay_pointer_move_perf_cleanup_reenters_underlay_before_outside_press`.
  - Evidence: perf log entry `2026-05-08 13:06`; single-script run PASS with bundle
    `target/fret-diag/codex-overlay-pointer-move-reentry-check/1778216164094/bundle.schema2.json`.

- [x] Stabilize `ui-gallery-virtual-list-torture-steady` row-jump setup.
  - Change: seed `ui-gallery-virtual-list-jump-input` with `9000` before clicking `Jump`; keep the steady script setup
    before `reset_diagnostics` so text input frames do not pollute the perf capture window.
  - Gate: `cargo nextest run -p fret-ui-gallery virtual_list_torture_scripts_seed_jump_input_before_waiting_for_row_9000 virtual_list_steady_script_keeps_jump_input_setup_outside_perf_capture_window`.
  - Evidence: perf log entry `2026-05-08 13:14`; full `ui-gallery-steady` repeat=1 passes after the script fix.

- [ ] Keep the Zed/GPUI + egui reference map current and milestone-linked:
  - Reference: `docs/workstreams/standalone/ui-perf-gpui-gap-v1.md`
  - egui adds the pass/repaint/cache accounting counter-reference; keep it updated when a Fret optimization changes
    frame cause accounting, extra-pass behavior, cache eviction, scene diff/replay, or multi-viewport repaint coupling.
  - When a gap is materially improved, add a perf log entry + mark the corresponding milestone tasks here.
  - Latest: refreshed the GPUI gap map on 2026-05-12 so it no longer treats broad `ui-gallery-steady` as the canonical
    formal gate and now points at the dedicated resize/code-editor/payload contracts plus the current no-display-list
    rewrite decision.

- [ ] Dev tooling: keep the “perf investigation loop” crisp for contributors (skills + checklists + attribution playbooks).
  - Workstream: `docs/workstreams/standalone/perf-devtools-skills-v1.md`
  - Deliverable (initial): expand `fret-perf-optimization` attribution recipes + add a single “worked example”.
  - Latest:
    - Added `fret-perf-attribution` skill (tail-hitch playbook): commit `7ea708d2f`.
    - Added `click_stable` diag script step to reduce selector-driven flakiness: commit `75ac42db9`.
    - Stabilized the Dialog steady perf probe after gallery surface drift, fixed `diag perf` `meta.env_defaults`
      launch parity, and recorded Windows RTX4090 evidence: commit `76cd1160c`.
    - Scoped suite env defaults to per-launch-group, moved font bootstrap ownership onto the prewarm script, and
      verified a mixed `dialog` + `context-menu` smoke under `gallery-full`: commit `1776617de`.

- [ ] ADR alignment: document and lock down the “interactive resize perf policy” contracts (what is allowed to be
  bucketed/deferred/cached during live resize, and what must remain exact).
  - Candidates: text wrap width bucketing, measure/shaping caching, released blob retention, and any LOD/deferral.
  - Goal: make future “fearless refactors” safer by pinning what must remain stable.
  - Companion note: `docs/workstreams/standalone/ui-perf-resize-path-v1.md`

- [ ] **P0 Resize-drag smoothness**: reduce `layout/solve` costs and eliminate avoidable secondary probes under
  `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`.
  - Companion probe (width jitter / live-drag approximation):
    `tools/diag-scripts/ui-gallery-window-resize-drag-jitter-steady.json`.
  - [ ] Explain why `top_layout_engine_solves` is typically > 1 in resize probes, and decide which roots should be
    solved separately vs batched.
    - Background: `docs/workstreams/standalone/ui-perf-resize-path-v1.md`
  - [ ] Attribute the current Windows RTX 4090 normalized resize-stress sample where view-cache reuse is active but
    `layout_roots_time_us` / `layout_request_build_roots_time_us` still dominate.
    - Baseline evidence: `target/fret-diag/1778235545947/bundle.schema2.json`
    - Current p50/p95: total `15276/15296us`, layout `11429/11674us`, paint `3649/3732us`,
      `layout.engine_solve` `505/2174us`
    - Contract pressure: GPUI says reuse boundaries should avoid broad subtree churn; egui says any repeated pass/cache
      work must be explicitly accounted and tied to frame cause.
    - [x] Add root-level request-build attribution so `layout_request_build_roots_time_us` can be split by root and
      mode before another optimization is proposed.
      - Bundle field: `debug.layout_request_build_roots[]`
      - Stats/triage surface: `layout_request_build_roots` row output and internal `fret-diag`
        `layout.build_roots_heavy.evidence.examples`
      - Evidence: perf log entry `2026-05-08 19:30`; smoke bundle
        `target/fret-diag/codex-request-build-roots-smoke/1778239301005/bundle.schema2.json`.
    - [x] Re-run the normalized resize-stress script and classify the top request-build roots as `mark_seen`,
      `cached_flow_reuse`, or `build_flow` dominated.
      - Evidence: perf log entry `2026-05-08 19:50`; repeat=3 worst bundle
        `target/fret-diag/codex-request-build-roots-r3/1778239800406/bundle.schema2.json`.
      - Result: heavy resize frames are `build_flow` dominated; `mark_seen` is cheap, and `cached_flow_reuse` frames
        move the remaining cost to `layout_roots_time_us` / barrier solves rather than request-build.
    - [x] Add root dirty-count attribution before considering a self-only root cached-flow reuse
      optimization.
      - Fields: `subtree_layout_dirty_count` and `descendant_layout_dirty_count` on
        `debug.layout_request_build_roots[]`, `fretboard diag stats`, and triage JSON.
      - Evidence: perf log entry `2026-05-08 20:12`; smoke bundle
        `target/fret-diag/codex-request-build-roots-dirty-count-smoke/1778241162991/bundle.schema2.json`.
      - Result: the smoke sample's top heavy root reports `layout_invalidated=false`, `subtree_dirty=true`,
        `subtree_layout_dirty_count=4`, and `descendant_layout_dirty_count=4`, so the next optimization should not
        assume a root-only invalidation.
    - [x] Attribute the dirty descendant sources inside the top request-build roots before proposing another
      cached-flow or dirty-frontier optimization.
      - Fields: `dirty_descendants[]` under each `debug.layout_request_build_roots[]` entry, with element kind/path,
        `subtree_layout_dirty_count`, `source_root_node`, `source`, and `detail`.
      - Evidence: perf log entry `2026-05-08 21:00`; smoke bundle
        `target/fret-diag/codex-request-build-roots-dirty-desc-final-smoke/1778245207520/bundle.schema2.json`.
      - Result: the top root is still descendant-dirty, and the sampled dirty descendants are `Opacity` /
        `Scrollbar` nodes with `source=other` and `detail=unknown`.
    - [x] Refine `unknown` dirty-source details for the sampled `Opacity` / `Scrollbar` descendants.
      - Target: distinguish scroll-handle authored layout, structural child rewrites, view-cache repair, and generic
        local invalidations before changing cached-flow or dirty-frontier behavior.
      - Implementation: added mechanism-layer detail categories for `initial_mount`, `local_invalidation`,
        `structural_children_changed`, `structural_parent_repair`, `barrier_followup_relayout`,
        `view_cache_layout_dirty_expansion`, `subtree_layout_dirty_repair`,
        `interactive_resize_full_rebuild`, and `prepaint_invalidation`.
      - Evidence: perf log entry `2026-05-08 21:29`; smoke bundle
        `target/fret-diag/codex-dirty-source-detail-smoke/1778246942782/bundle.schema2.json`.
      - Result: the same `Opacity` / `Scrollbar` descendants now classify as `detail=initial_mount` instead of
        `unknown`, so the next layout behavior change should focus on whether these scroll-area chrome mounts are
        expected resize churn or avoidable subtree identity churn.
  - [x] Runner no-op resize drop (GPUI parity): track last delivered quantized logical size and skip delivering
    `Event::WindowResized` when unchanged.
    - Rationale: reduce float-noise churn in window-metrics consumers; align with GPUI `set_frame_size` early-return.
    - Reference: `docs/workstreams/standalone/ui-perf-resize-path-v1.md` (runner coalescing + GPUI note).
    - Implementation: commit `d834481b3`.
  - [x] Harden the `ui-resize-probes` gate against rare tail outliers by running multiple attempts and requiring a
    strict majority pass (keeps the gate strict, but reduces single-run flake).
    - Gate runner: `tools/perf/diag_resize_probes_gate.sh --attempts 3`
    - Implementation: commit `4755aa087`
  - [ ] Stabilize `ui-resize-probes` `drag-jitter` tail behavior on macOS M4 (avoid intermittent gate failures).
    - Evidence: perf log entry `2026-02-08 12:20:46` (attempts=3; 0/3 pass; one near-threshold run and one outlier).
    - Candidate actions:
      - Cut a new baseline (v4) with more candidates/validation runs on an idle machine.
      - If it remains flaky, revisit the metric/seed/headroom contract for `drag-jitter` (keep “no hitch” intent).
    - Latest: perf log entry `2026-02-08 13:32:06` shows attempts=3 PASS on the merged head (`828c945d4`).
    - Latest: perf log entry `2026-02-09 19:15:00` shows attempts=3 PASS with a default-on small-step interactive-resize
      wrap-width LRU for prepared text blobs (commit `58db05d7c`).
  - [x] Quantize `LayoutMeasureKey` bits to reduce float-noise in measure caching (commit `94057ffab`).
    - Evidence + numbers: perf log entry `2026-02-07 11:15` in `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`.
  - [x] Record resize-drag worst-frame attribution (ScrollArea + text wrap under width jitter).
    - Evidence: perf log entry `2026-02-07 11:15` (r16 worst bundle + snapshot pointers).
  - [x] Quantize logical window sizes in the runner to reduce float-noise resize churn (commit `74dc38bd9`).
    - Evidence: perf log entry `2026-02-07 11:50`.
  - [x] Post-merge sanity: ensure the P0 resize probes gate still passes after integrating upstream `main` (commit `9bf37cc0b`).
    - Evidence: perf log entry `2026-02-07 20:39` (`target/fret-diag-resize-probes-gate-r21/summary.json`).
  - [x] Re-validate both resize gates on the current head (no-code-change evidence snapshot).
    - Evidence: perf log entry `2026-02-08 12:20:46` in `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`.
  - [x] Track an “interactive resize” window in the UI tree to enable guarded LOD/deferral experiments (commit `34bac1b78`).
    - Evidence: perf log entry `2026-02-07 21:23` (`target/fret-diag-resize-probes-gate-r24/summary.json`).
  - Use `debug.layout_hotspots[]` (exclusive) and `debug.layout_inclusive_hotspots[]` (inclusive) attribution to
    identify dominant layout contributors even when time is distributed across child widgets (commit `69111ebde`).
    - `layout_hotspots[]` includes `element_kind` and best-effort `element_path`, plus
      `layout_engine_child_rect_*` counters (commit `3d6f0870e`).
    - Fix `element_path=null` during cache-hit frames by touching debug-identity ancestor chains (commit `e46b8df08`).
  - [x] Reduce flow layout build allocations (avoid `UiTree::children(...).to_vec()` clones; avoid cloning the
    previous children vec in `TaffyLayoutEngine::set_children`).
    - Implementation: commit `10e30dac1`.
    - Evidence: perf log entry `2026-02-09 09:10:11` in `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`.
  - [x] Narrow command-availability revision bumps so paint-only animation frames do not recompute window gating.
    - Change: only bump `command_availability_revision` for invalidations that can affect command availability /
      semantics; keep paint-only animation and hover churn out of the revision path.
    - Gates:
      - `cargo nextest run -p fret-ui window_command_action_availability_snapshot`
      - `cargo nextest run -p fret-runtime register_bumps_revision`
      - `cargo build -p fret-ui-gallery --release --features gallery-full`
    - Evidence: perf log entry `2026-05-08 11:46:31`; on the Material3 tabs steady probe the aggregated
      `window_runtime_snapshot_command_availability_time_us` sum drops from `4781081us` to `1117167us`
      and the max frame drops from `651032us` to `335809us` after the revision scope is narrowed.
  - [x] Keep command/action availability snapshots dispatch-path scoped and avoid whole-subtree fallback scans.
    - Change: `publish_window_command_action_availability_snapshot` no longer runs
      `command_availability_in_subtree` for each widget command; it keeps focus/default-route availability plus
      explicit focus traversal and menu-bar hooks.
    - Diagnostics: `diag stats` now reports
      `window_runtime_snapshot_widget_command_count`,
      `window_runtime_snapshot_command_registry_collect_time_us`, and
      `window_runtime_snapshot_command_availability_eval_time_us`.
    - Gates:
      - `cargo nextest run -p fret-ui window_command_action_availability_snapshot`
      - `cargo check -p fret-diag -p fret-bootstrap`
      - `cargo build -p fret-ui-gallery --release --features gallery-full`
    - Evidence: perf log entry `2026-05-08 12:42`; Material3 tabs dispatch p95 drops from
      `220550us` to `1095us`, and availability eval on the worst command snapshot drops from
      `322040us` to `911us`.
- [x] **P0.5 Code editor resize drag smoothness**: close the remaining 2–3× gap to the editor resize threshold.
  - Evidence snapshot: perf log entry `2026-02-09 12:34:16` (commit `1778ba563`) showing the gate passing 3/3
    with `top_total_time_us≈15.6–16.0ms` vs `16308us` target.
  - Root cause (confirmed): per-frame syntax/rich cache resets caused by non-idempotent `CodeEditorHandle::set_language(...)`
    being called during render.
  - [x] Cache per-row rich syntax materialization (`AttributedText`) to reduce per-frame span merge churn.
    - Implementation: `perf(fret-code-editor): cache syntax rich text` (commit `26ad57906`) + build fix (commit `a78a5fc76`).
    - Evidence: perf log entry `2026-02-09 11:31:52` (commit `a78a5fc76`) showing a ~5ms reduction in Canvas time in the best attempt
      but still far above threshold.
  - [x] Add code-editor-Canvas internal attribution (no more “Canvas is 30ms”).
    - Implementation: `feat(diag): add code editor paint perf breakdown` (commit `f664ead2d`).
    - Output: `bundle.json` `app_snapshot.code_editor.torture.paint_perf` (opt-in via `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`).
    - Evidence: perf log entry `2026-02-09 12:22:35` (commit `f664ead2d`).
  - [x] Make `CodeEditorHandle::set_language(...)` idempotent (no per-frame cache reset).
    - Implementation: `perf(fret-code-editor): make set_language idempotent` (commit `1778ba563`).
    - Evidence: perf log entry `2026-02-09 12:34:16` (commit `1778ba563`).
  - Latest no-code evidence on Windows RTX 4090: perf log entry `2026-05-09 18:05` shows
    `ui-gallery-code-editor-window-resize-drag-jitter-steady` repeat=3 at
    `total/layout/paint/solve p95=3995/2137/1747/574us`, with the real 20k-line torture surface active and
    `paint_perf.us_total=365us` in the sampled bundle. Do not start the row display-list rewrite from this sample
    alone; either create a stricter editor paint stressor or move to a currently near-threshold gate.
  - Latest macOS M4 contained-layout gate: perf log entry `2026-05-13 16:44:18 +08:00` shows
    `ui-code-editor-resize-probes` repeat=3 passing `docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json`
    with `failures=[]`, non-zero `code_editor.paint_perf`, and p95 total/layout/paint/solve
    `1361/295/1134/116us`. This closes the current macOS layout-solve failure; the next measured
    limiter on this script is paint/widget row replay and content resolution.
  - [x] Contain the code-editor gallery content cache layout boundary during resize.
    - Implementation: `apps/fret-ui-gallery/src/spec.rs` marks only the code-editor MVP/torture pages as contained
      layout roots, and `apps/fret-ui-gallery/src/driver/shell.rs` forwards that policy into `ViewCacheProps`.
    - Evidence: perf log entry `2026-05-13 16:44:18 +08:00`; p95 total drops from the script-fix smoke's
      `2070us` to `1361us` and p95 layout solve from `766us` to `116us`.
  - Latest payload-aware typical-frame contract: perf log entry `2026-05-11` promotes
    `ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v2.json` with
    `threshold_surface=ui-renderer-payload`, measured p50/p95/max top total=`2563/3603/3603us`, hard frame p95
    thresholds total/layout/solve=`3360/368/0us`, and payload thresholds
    `max_renderer_instance_bytes=262416`, `max_renderer_encode_scene_text_ops=406`. This covers the missing
    typical-frame paint/payload surface, but it is still a passing contract; only start a `WindowedRowsSurface`
    display-list rewrite from a future near-threshold or failing stressor.
  - [x] Cache code-editor frame overlay state before row paint.
    - Change: `begin_paint_frame` now prepares normalized selection bytes/display points plus caret byte/row/col once
      per `WindowedRowsSurface` frame, and `paint_row` consumes that snapshot for preedit injection, fallback
      selection geometry, and caret overlay painting.
    - Evidence: perf log entry `2026-05-11` (`complex editor wheel frame overlay cache`); paint-detail
      `ns_row_overlay` p95 drops from `556.0us` to `8.2us`, while frame overlay preparation is `9.2us` p95.
    - Decision: this follows the GPUI/Zed prepaint-derived-state direction and fixes duplicated display-map work. It
      does not by itself justify a row display-list rewrite because row scene replay remains high and renderer payload
      is still bounded.
  - [x] Fix soft-wrap syntax prefetch to use buffer lines, not display rows.
    - Change: syntax prefetch now maps `WindowedRowsPaintFrame` display rows through
      `DisplayMap::display_row_line(...)` before chunk selection, and row/rich cache capacity observes a frame-local
      visible-window floor.
    - Evidence: perf log entry `2026-05-11 20:18` (`complex editor wheel syntax prefetch line mapping`); the
      paint-detail spike drops from `5681us` with `syntax_evict_delta=85`, `row_rich_miss_delta=85`, and
      `rows_scene_stored=86` to `3580us` with syntax/rich evictions gone and row scene misses mostly `1..5`.
    - Decision: this is the correct semantic fix before any row display-list rewrite. The cache window uses display
      rows, while syntax/rich chunking must use physical buffer lines.
  - [ ] Reduce per-row scene op churn in `WindowedRowsSurface` paint.
    - Candidate directions:
      - record per-row scene ops once and replay them with a pure transform/translation boundary,
      - split replay/capture cost from renderer encode/upload cost before changing thresholds,
      - reduce quads/text ops count (batching or fewer per-row background ops),
      - avoid per-frame allocations in the hot loop (scratch vec reuse, pre-sized buffers).
    - Current blocker: latest complex wheel evidence points first at frame-derived overlay work, then at a
      display-row/physical-line syntax prefetch bug, and now at Canvas paint-widget / renderer encode payload cost
      rather than missed row-scene reuse. Start this only from a future near-threshold or failing stressor where the
      measured limiter is replay/capture itself, not syntax/rich cache churn or a stale display-row mapping.
  - [ ] Re-evaluate text blob cache behavior for editor rows under resize jitter.
    - Confirm whether the hitch is dominated by fingerprint comparison, text prepare, atlas upload, or pure CPU list building.
    - If dominated by fingerprint compare, consider pointer-fast-pathing for more content variants (and/or richer cache keys).
  - [ ] Add a smaller, editor-only perf suite to reduce noise while iterating.
    - Goal: a probe that opens only the code editor torture view (or a minimal demo) and runs the same resize jitter steps.
    - Deliverable: `tools/diag-scripts/*` + a `ui-code-editor-*` perf baseline/policy if needed.
  - [ ] GPU side validation (only after CPU attribution is clear).
    - Run RenderDoc/Tracy captures for the worst bundle and confirm whether the hitch is CPU-bound (scene build/text) or GPU-bound (uploads/compositing).
- [ ] **P0.6 Declarative “setter idempotency” contract**: eliminate per-frame cache reset footguns.
  - Motivation: declarative element trees will call `handle.set_*` during render; setters must be no-ops when values are unchanged.
  - Evidence: editor resize drag fix (commit `1778ba563`) was a ~2.6× win by making `set_language` idempotent.
  - [x] Audit + fix code-editor `CodeEditorHandle::set_*` methods that can be called from render and clear caches/epochs.
    - Gate: `ui-code-editor-resize-probes` must remain PASS.
    - Evidence: `set_language` idempotent (commit `1778ba563`), `set_line_folds`/`set_line_inlays` idempotent (commit `007006b28`), and `set_text` idempotent for identical content (perf log entry `2026-05-15 21:34:00`).
    - Latest: render-time view setters `set_soft_wrap_cols`, `set_code_font_feature_policy`, and `set_interaction`
      audited; soft-wrap/font-feature no-op cache behavior is now covered by regression tests (perf log entry
      `2026-05-15 22:05:00`).
  - [x] Audit other “handle-style” surfaces used from render (markdown editor, docking, viewport tooling, and code-view prepared state) for the same pattern.
    - Deliverable: list of audited setters + commit references in the perf log.
    - Evidence: docking viewport layout publication now uses `DockManager::sync_viewport_layouts_for_window(...)`
      instead of clearing and reinserting identical render-frame layouts; `ViewportToolArbitrator::set_tools(...)`
      is audited as a replacement/cancellation command, not a render-safe setter (perf log entry `2026-05-15 21:45:00`).
    - Evidence: markdown preview uses already-audited `CodeEditorHandle` setters and gates fold/inlay fixture
      updates with slot-local last-value checks; `fret-code-view` prepared state idempotency is covered by
      `prepared_state_is_idempotent_for_identical_inputs` (perf log entry `2026-05-15 22:20:00`).
  - [x] Add at least one regression test per high-risk surface.
    - Evidence: `test(fret-code-editor): cover set_language idempotency` (commit `4847d4f13`) + fold/inlay idempotency tests (commit `007006b28`) + `set_text_is_idempotent_for_same_text` + `set_soft_wrap_cols_is_idempotent_for_same_value` + `code_font_feature_policy_is_idempotent_for_same_value` + `prepared_state_is_idempotent_for_identical_inputs`.
  - [x] Add a short guidelines note describing the contract and common pitfalls.
    - Deliverable: `docs/workstreams/standalone/ui-perf-setter-idempotency-v1.md` (commit `420845878`).
  - [x] Extend the audit to retained text-input surfaces where render-time state re-application is likely.
    - Evidence: `perf(fret-ui): make TextArea::set_text idempotent` (commit `fcd1ada2d`) + perf log entry
      `2026-02-09 17:00:00`.
- [ ] **P1 Text under width jitter**: stabilize wrapped-text cache keys (and consider bucketed widths during resize).
  - [x] Reduce Word-wrap cost on long paragraphs by shaping once and slicing per-line layouts (plain LTR only).
    - Implementation: `perf(text): shape-once word wrap` (commit `4f2009408`) + default-on for long wraps (commit `10e7d97fc`).
    - Knob: `FRET_TEXT_WORD_WRAP_SHAPE_ONCE` (`1`/`0`) overrides the default threshold behavior.
    - Evidence: perf log entries appended for the A/B run and the default behavior (2026-02-07, `ui-resize-probes`).
  - [x] Add a default small-step wrap-width bucketing policy during interactive resize to reduce text wrap churn under
    `drag-jitter`-style width jitter.
    - Default: `FRET_UI_TEXT_WRAP_WIDTH_SMALL_STEP_BUCKET_PX=32` (set to `0`/`1` to disable).
    - Applies only when:
      - interactive resize is active, and
      - the window width delta is small (jitter-class, not stress-class).
    - Keep the old knob for global experiments:
      - `FRET_UI_TEXT_WRAP_WIDTH_BUCKET_PX` (still default-off; applies across all interactive resize frames).
  - [x] Treat interactive-resize “small-step” detection as symmetric so back-and-forth drags keep the same
    bucketing/caching policies enabled.
    - Implementation: `perf(fret-ui): treat small-step resize symmetrically` (commit `0de40863f`).
    - Evidence: perf log entry `2026-02-09 16:37:00` (both resize probe gates PASS; `ui-resize-probes` p95 total down
      ~0.3ms on the worst jitter probe).
  - [x] Widen the “small-step” `dw` threshold so bucketing applies under common drag deltas (not only <=16px).
    - Implementation: `perf(fret-ui): widen resize small-step wrap bucketing` (commit `53aa6534a`).
    - Knob: `FRET_UI_TEXT_WRAP_WIDTH_SMALL_STEP_MAX_DW_PX` (default: `64`).
    - Evidence: perf log entry `2026-02-09 22:54:20` (`ui-code-editor-resize-probes` gate passes 3/3; p95 total down
      ~0.95ms vs the prior run).
  - [x] Normalize nowrap text-blob cache keys to ignore `max_width` when `overflow!=Ellipsis` (clip/visible).
    - Implementation: `perf(fret-render): ignore max_width for nowrap blobs` (commit `1ce4693a9`).
    - Evidence: perf log entry `2026-02-08` (editor resize gate delta).
  - [x] Align declarative host-widget paint prepare invalidation with the nowrap text-blob key.
    - Implementation: `crates/fret-ui/src/declarative/host_widget/paint.rs`.
    - Contract: `TextWrap::None + overflow!=Ellipsis + align=Start` is width-insensitive for paint prepare;
      ellipsis and non-start alignment remain width-sensitive.
    - Evidence: perf log entry `2026-05-13 14:39:15 +08:00` (unit guards + text-measure overlay diag).
  - [x] Normalize Canvas hosted/shared text fingerprints to ignore `max_width` for nowrap+non-ellipsis.
    - Implementation: `perf(fret-ui): normalize nowrap canvas text keys` (commit `667d8317b`).
    - Evidence: perf log entry `2026-02-08` (editor resize jitter drops to ~13ms worst-frame).
  - [x] Avoid code editor baseline text measurement churn during resize by making baseline alignment caching
    independent of the row `max_width`.
    - Implementation: `perf(fret-code-editor): avoid baseline measure churn on resize` (commit `dd2da2ada`).
    - Evidence: perf log entry `2026-02-08` (`ui-code-editor-resize-probes` p95 total ~11.8ms).
  - [x] Add an experimental interactive-resize wrapped-text width cache to reduce `Text::prepare` churn when
    dragging back-and-forth across wrap-width buckets.
    - Implementation: `feat(fret-ui): add interactive-resize wrapped text width cache knob` (commit `2e479fc2f`).
    - Knob: `FRET_UI_INTERACTIVE_RESIZE_TEXT_WIDTH_CACHE_ENTRIES` (default: `0`/off; try `4`).
    - Evidence: perf log entries `2026-02-08` (A/B: off vs `ENTRIES=4`).
  - [x] Add a renderer-owned, bounded “released blob” retention policy (LRU / time-based) to avoid thrashing
    `Text::prepare` under interactive resize width jitter.
    - Rationale: `TextSystem::release` currently eagerly evicts blobs when refcount hits zero, which amplifies churn
      when the UI alternates between a small set of wrap widths.
    - Expected impact: reduce `paint_text_prepare.us(time/calls)` spikes on resize frames even without per-widget
      multi-width caches.
    - Implementation: `perf(fret-render): retain released text blobs via LRU` (commit `abf7ce646`).
    - Knob: `FRET_TEXT_RELEASED_BLOB_CACHE_ENTRIES` (default: `0`/off; A/B tested at `256`).
    - Evidence: perf log entry `2026-02-08 15:51:15` (A/B gates + worst-frame attribution).
  - [x] Add a width-independent “unwrapped layout” cache and reuse it for word wrap under width jitter (GPUI-style).
    - Goal: prevent “shape again” work when only wrap widths change during interactive resize (especially in the
      code editor jitter probe).
    - Implementation:
      - `perf(fret-render): reuse unwrapped layouts for word wrap` (commit `2fac17832`)
      - `perf(fret-render): avoid fallback after unwrapped wrap` (commit `06a16f35b`)
    - Knobs:
      - `FRET_TEXT_UNWRAPPED_LAYOUT_CACHE_ENTRIES` (default: `0`/off; A/B tested at `2048`)
      - `FRET_TEXT_UNWRAPPED_LAYOUT_CACHE_MAX_TEXT_LEN_BYTES` (default: `4096`; A/B tested at `16384`)
    - Evidence: perf log entry `2026-02-08 17:38:51` (A/B gates + worst-frame attribution; editor gate flips
      from FAIL to PASS when enabled).
  - [x] Avoid cloning per-line glyph/cluster vectors when wrapping from cached unwrapped layouts (word wrap, LTR).
    - Implementation: `perf(fret-render-wgpu): avoid cloning glyphs during wrap` (commit `7b9a98a8f`).
    - Evidence: perf log entry `2026-02-09 22:12:02` (`ui-resize-probes` gate attempts=3 PASS; commit-bound).
  - [ ] Follow-up: validate memory bounds + eviction behavior on longer editor sessions (ensure the LRU remains
    bounded and does not retain pathological blobs indefinitely).
  - [ ] Follow-up: decide if `FRET_TEXT_UNWRAPPED_LAYOUT_CACHE_ENTRIES` should become a default-on policy for
    native builds (with an opt-out env), and add explicit diagnostics counters for cache hit/miss rates so we can
    validate “global optimum” across the acceptance suite.
  - [x] Bucket wrapped-text **measure** widths during interactive resize in the host-widget layout path to reduce
    measure churn and align layout/paint wrap widths.
    - Implementation: `perf(fret-ui): bucket wrapped text measure width during resize` (commit `b6c4d1094`).
    - Evidence: perf log entries `2026-02-08` (`ui-code-editor-resize-probes` and P0 `ui-resize-probes` sanity).
  - [x] Stabilize `TextService::measure` shaping reuse working-set to reduce `layout_engine_solve` tail outliers
    (avoid occasional “measure reshaping thrash” during interactive resize).
    - Implementation: `perf(fret-render): stabilize measure shaping cache tail` (commit `f2c08b806`).
    - Knobs:
      - `FRET_TEXT_MEASURE_SHAPING_CACHE_ENTRIES` (default: `4096`; clamp: `64..=65536`)
      - `FRET_TEXT_MEASURE_SHAPING_CACHE_MIN_TEXT_LEN_BYTES` (default: `128`; cache only long paragraphs)
    - Evidence: perf log entry `2026-02-08 23:44:01` (`ui-code-editor-resize-probes`, `ui-resize-probes`, `ui-gallery-steady`).
  - [x] Attempt: reuse prepared text blobs across the host-widget layout/paint paths.
    - Implementation: `perf(fret-ui): reuse prepared text across layout/paint` (commit `e337b4299`).
    - Evidence: perf log entry `2026-02-09 21:14:30` (`ui-resize-probes` attempts=5 PASS).
    - Known gap: most UI gallery nodes are sized via the layout engine’s measure callback; follow up by aligning
      `TextService::measure`/`prepare` cache behavior so paint can reuse measurement work without re-shaping.
- [ ] **P1.5 Editor canvas paint replay**: reduce editor-class `Canvas` paint cost (scene-op rebuild), aiming for
  “paint-only” frames under small-step resize/scroll.
  - Primary probes:
    - `tools/diag-scripts/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`
    - `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`
  - Work items (fearless refactor allowed; log every perf-affecting change):
    - [x] Close the stale prepaint-planner follow-on before widening the editor paint lane.
      - Decision: `code-editor-prepaint-planner-cost-v1` is closed by
        `docs/workstreams/code-editor-prepaint-planner-cost-v1/CLOSEOUT_AUDIT_2026-05-16.md`.
        The lane reduced `us_row_scene_prepaint_plan` p95 from `91us` to `67us` and preserved
        `rows_scene_fast_miss_no_entry == 0` / `rows_scene_full_miss_no_entry == 0`, but fresh
        post-merge evidence moved the dominant hotspot to paint/widget and Canvas replay/cache
        attribution.
    - [x] Short-circuit planned row replay paint when no selection/preedit/overlay work remains.
      - Implementation: `perf(code-editor): short-circuit planned row replay paint` (`3086481679`).
      - Evidence:
        `ecosystem/fret-code-editor/src/editor/paint/mod.rs`,
        `ecosystem/fret-code-editor/src/editor/state.rs`, and
        `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`.
      - Gate:
        `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`.
      - Smoke evidence:
        `target/fret-diag/paint-widget-canvas-replay-fast-return-smoke-20260515/1778856015202-ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady/bundle.schema2.json`
        and
        `target/fret-diag/code-editor-resize-replay-fast-return-smoke-20260515/1778856345501-ui-gallery-code-editor-window-resize-drag-jitter-steady/bundle.schema2.json`.
        These show low code-editor paint p95 (`229us` complex wheel, `113us` resize replay) while
        frame-level `paint.widget` and renderer text/encode/upload remain the attribution surfaces
        to watch.
      - Next formal evidence requirement: re-run the editor paint probes with repeat/warmup policy
        on the target machine profile and compare Canvas `paint.widget`, row content resolve,
        row-scene replay/cache replay, and renderer encode/upload payload before tightening any
        baseline or starting a broader display-list rewrite.
    - [x] Run the formal repeat=3 editor Canvas replay evidence pass after the planned row replay
      short-circuit.
      - Evidence: perf log entry `2026-05-16 01:03:00 +08:00` (`editor canvas replay formal
        evidence pass`).
      - Result:
        - typical autoscroll: total p50/p95=`777/887us`, `paint.widget` p50/p95=`384/439us`,
          code-editor paint p50/p95=`105/131us`, row replay hit rate `100%`, stores `0`.
        - complex wheel: total p50/p95=`894/1037us`, `paint.widget` p50/p95=`539/634us`,
          code-editor paint p50/p95=`255/330us`, row replay hit rate `99%`, stored-row p95 `1`.
        - resize jitter: total p50/p95=`883/1145us`, `paint.widget` p50/p95=`432/446us`,
          code-editor paint p50/p95=`123/138us`, row replay hit rate `100%`, stores `0`.
      - Decision: row replay/cache and prepaint planning are not the next mainline bottlenecks.
        Keep the display-list rewrite gated on a future near-threshold/failing stressor where row
        replay/capture itself is measured as the limiter.
    - [x] Split the next owner lane between Canvas wrapper overhead and renderer text/encode payload.
      - Target: explain why `paint.widget` remains roughly `439..634us` p95 while
        `code_editor.paint_perf` is only `131..330us` p95, and why renderer text prepare remains
        roughly `419..435us` p95/max with atlas upload/eviction at `0`.
      - Result: renderer text prepare was the first safe owner slice. `TextSystem::collect_scene_pinned_keys(...)`
        now pre-sizes glyph pin buckets from per-shape pin-key counts before merging scene text blobs.
      - Evidence: perf log entry `2026-05-16 01:20:00 +08:00` (`renderer glyph pin bucket capacity`).
        Renderer text p95/max changed:
        typical `392/422us -> 360/376us`, complex wheel `412/435us -> 381/412us`, and resize
        jitter `419/419us -> 379/379us`.
      - Contract decision: keep checked-in payload baselines unchanged; strict baseline audit still passes.
    - [x] Exclude the diagnostic torture overlay from the formal editor perf probes.
      - Change: `FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0` is now the default env for the
        three formal editor torture scripts that drive the contract matrix.
      - Validation:
        `cargo nextest run -p fret-ui-gallery code_editor_perf_contract_scripts_disable_torture_overlay_by_default --no-fail-fast`
        and
        `cargo nextest run -p fret-ui-gallery --features gallery-dev code_editor_torture_overlay_env --no-fail-fast`.
      - Evidence: perf log entry `2026-05-16 04:54:35 +08:00`
        (`formal editor probes exclude torture overlay`).
      - Result: overlay-disabled repeat=3 evidence now covers typical autoscroll, complex wheel, and resize
        jitter:
        `target/fret-diag/editor-paint-overlay-disabled-20260516-typical-r3/1778878430806/bundle.schema2.json`,
        `target/fret-diag/editor-paint-overlay-disabled-20260516-complex-wheel-r3/1778878778260/bundle.schema2.json`,
        and `target/fret-diag/editor-paint-overlay-disabled-20260516-resize-jitter-r3/1778878807245/bundle.schema2.json`.
        All three report `top_code_editor_torture_overlay_us=0`; row replay remains healthy (`289/0`,
        `287/2`, and `289/0` replay/store p95).
    - [ ] Close the remaining Canvas wrapper / `paint.widget` attribution gap.
      - Current evidence after the renderer slice: `paint.widget` p95 is still `414us` typical,
        `633us` complex wheel, and `421us` resize jitter, while `code_editor.paint_perf` p95 is
        `123us`, `318us`, and `119us` respectively.
      - Target: split the gap between `WindowedRowsSurface` frame/row-loop overhead, per-row closure
        dispatch/state access, Canvas host wrapper work, and any generic ElementHostWidget paint
        bookkeeping before proposing a broader Canvas or display-list refactor.
      - [x] Add machine-readable `WindowedRowsSurface` paint diagnostics to support that split.
        - Fields now include `us_windowed_surface_paint_callback`, `us_windowed_surface_hook`,
          `us_windowed_surface_row_loop`, `us_windowed_surface_row_paint`,
          `us_windowed_surface_non_row`, and `us_windowed_surface_row_callback_gap` in
          `code_editor.paint_perf`.
        - Evidence: perf log entry `2026-05-16 01:30:00 +08:00`
          (`windowed surface paint attribution fields`).
      - [x] Re-run the formal repeat=3 Editor Canvas replay probes with these new fields before
        deciding whether the next reversible optimization belongs to generic Canvas wrapper work,
        code-editor row callback overhead, or another renderer slice.
        - Evidence: perf log entry `2026-05-16 01:45:00 +08:00`
          (`editor canvas wrapper attribution formal evidence`).
        - Result:
          - typical autoscroll worst bundle `target/fret-diag/editor-canvas-wrapper-attribution-20260516-typical-r3/1778865865185/bundle.schema2.json`:
            total p50/p95/max `765/862/1022us`, `paint.widget` p50/p95 `393/431us`,
            `code_editor.paint_perf` p50/p95 `93/106us`, surface callback p50/p95 `238/268us`,
            surface non-row p50/p95 `127/145us`, row callback gap p50/p95 `18/21us`, row replay
            hit rate `100%`, stores `0`.
          - complex wheel worst bundle `target/fret-diag/editor-canvas-wrapper-attribution-20260516-complex-wheel-r3/1778865994148/bundle.schema2.json`:
            total p50/p95/max `881/1156/1170us`, `paint.widget` p50/p95 `553/653us`,
            `code_editor.paint_perf` p50/p95 `253/321us`, surface callback p50/p95 `405/489us`,
            surface non-row p50/p95 `138/154us`, row callback gap p50/p95 `12/14us`, row replay
            hit rate `99.65%`, stored p95 `3`.
          - resize jitter worst bundle `target/fret-diag/editor-canvas-wrapper-attribution-20260516-resize-jitter-r3/1778866025069/bundle.schema2.json`:
            total p50/p95/max `842/1287/1287us`, `paint.widget` p50/p95 `421/465us`,
            `code_editor.paint_perf` p50/p95 `111/133us`, surface callback p50/p95 `258/288us`,
            surface non-row p50/p95 `131/137us`, row callback gap p50/p95 `20/23us`, row replay
            hit rate `100%`, stores `0`.
        - Decision: the new fields split the inner surface cost cleanly, but the remaining outer
          `paint.widget - surface_callback` gap is still about `155..177us` p95. The next owner
          surface is generic Canvas / `ElementHostWidget` paint bookkeeping, not a broad
          `WindowedRowsSurface` display-list rewrite.
      - [x] Add `paint_widget_hotspot_summary` so `fretboard diag stats --json` can split the
        sampled top-N paint-widget hotspots into Canvas and non-Canvas classes before naming the
        next owner.
        - Implementation: `diag stats` now exports top-level
          `paint_widget_hotspot_summary`, with `sampled_top_n_per_frame=16`, per-frame top
          Canvas and non-Canvas hotspot p50/p95/max, sampled top-N class sums, top hotspot
          identity, and p95 gaps versus `code_editor.paint_perf.us_total` /
          `us_windowed_surface_paint_callback`.
        - Gate:
          `cargo nextest run -p fret-diag bundle_stats_summarizes_canvas_paint_widget_hotspots --no-fail-fast`.
        - Evidence on the same formal 2026-05-16 bundles:
          - typical autoscroll: `paint.widget` p95 `431us`, Canvas hotspot p95 `270us`,
            sampled non-Canvas top-N sum p95 `71us`, surface callback p95 `268us`, Canvas
            minus surface callback p95 gap `2us`.
          - complex wheel: `paint.widget` p95 `653us`, Canvas hotspot p95 `491us`,
            sampled non-Canvas top-N sum p95 `67us`, surface callback p95 `489us`, Canvas
            minus surface callback p95 gap `2us`.
          - resize jitter: `paint.widget` p95 `465us`, Canvas hotspot p95 `292us`,
            sampled non-Canvas top-N sum p95 `71us`, surface callback p95 `288us`, Canvas
            minus surface callback p95 gap `4us`.
        - Decision: the single Canvas hotspot is effectively the `WindowedRowsSurface` callback,
          not an additional outer Canvas-wrapper tax. The residual `paint.widget` cost after
          Canvas plus sampled top-N non-Canvas work is roughly `90..102us` p95, so the next
          reversible owner lane should focus on generic `ElementHostWidget` / paint traversal
          aggregate overhead, not code-editor row replay or a broad windowed-surface display-list
          rewrite.
      - [x] Promote existing host-widget paint subphase timers to root-level `diag stats`
        p50/p95/max output before changing `ElementHostWidget`.
        - Fields: `paint_host_widget_observed_models_time_us`,
          `paint_host_widget_observed_globals_time_us`,
          `paint_host_widget_instance_lookup_time_us`, plus the matching item/call counts in
          `p50`, `p95`, and `max`.
        - Gate:
          `cargo nextest run -p fret-diag bundle_stats_summarizes_canvas_paint_widget_hotspots --no-fail-fast`.
        - Evidence on the same formal 2026-05-16 bundles:
          - typical autoscroll p95 host models/globals/lookup `29/28/47us`.
          - complex wheel p95 host models/globals/lookup `29/29/47us`.
          - resize jitter p95 host models/globals/lookup `28/27/45us`.
        - Decision: these existing subphase timers account for roughly the same scale as the
          remaining `paint.widget` residual after Canvas plus sampled non-Canvas top-N work.
          The next optimization should be a narrow `ElementHostWidget::paint_impl` owner slice
          around observed-dependency replay and instance-record lookup, with the same three editor
          probes as formal evidence.
      - [x] Slim `ElementHostWidget::paint_impl` instance-record lookup to avoid cloning the full
        retained element record.
        - Implementation: `crates/fret-ui/src/declarative/host_widget/paint.rs` now extracts only
          inherited foreground, inherited text style, and element instance from the record before
          dispatching paint.
        - Gate:
          `cargo fmt -p fret-ui --check`;
          `cargo check -p fret-ui`;
          `cargo nextest run -p fret-ui -E 'test(~paint)' --no-fail-fast`.
        - Exploratory evidence: perf log entry `2026-05-16 02:23:09 +08:00`. No-reuse repeat=3
          samples report host lookup p95 around `39..43us` versus the earlier same-mouth formal
          `45..47us` range.
        - Contract decision: do not update baselines from this evidence. The no-reuse command
          completed, but the same-command `--reuse-launch` repeat=3 formal run timed out after
          navigation state drift. Treat the slice as a small reversible lookup optimization and
          keep formal contract closure blocked on stable same-mouth evidence.
      - [x] Fix or replace the editor paint `--reuse-launch` formal evidence path before the next
        baseline decision.
        - Failure: `target/fret-diag/editor-host-record-slim-20260516-typical-r3` and
          `target/fret-diag/editor-host-record-slim-20260516-typical-r3-reuse-prelude-each`
          timed out at the nav-item wait because the reused process retained stale nav search
          state (`nav_query_len_bytes=37`, no visible nav items).
        - Fix: `ui-gallery-code-editor-torture-autoscroll-steady` and
          `ui-gallery-code-editor-window-resize-drag-jitter-steady` now use `type_text_into`
          with `clear_before_type=true` for the gallery nav search, matching the already-stable
          complex wheel probe.
        - Gate:
          `python3 -m json.tool ...autoscroll-steady.json`;
          `python3 -m json.tool ...window-resize-drag-jitter-steady.json`;
          `python3 tools/check_diag_scripts_registry.py`;
          `cargo nextest run -p fret-diag-protocol --no-fail-fast`.
        - Evidence: perf log entry `2026-05-16 02:31:15 +08:00`. The three editor paint probes
          now pass same-mouth `--reuse-launch --repeat 3 --warmup-frames 5` evidence again:
          typical `target/fret-diag/editor-paint-contract-formal-20260516-typical-r3`,
          complex wheel `target/fret-diag/editor-paint-contract-formal-20260516-complex-wheel-r3`,
          and resize jitter `target/fret-diag/editor-paint-contract-formal-20260516-resize-jitter-r3`.
      - [x] Continue host-widget paint aggregate attribution only after the formal evidence path is
        stable.
        - Current formal evidence path status: stable enough to continue this lane. The next slice
          should use the same three editor paint probes before and after the change.
        - Candidate owners: observed model/global dependency replay, collapse observation cost,
          generic paint traversal aggregation, and the remaining sampled non-Canvas top-N hotspots.
        - Avoid: broad `WindowedRowsSurface` display-list rewrite, row replay rewrite, or renderer
          payload threshold changes unless new evidence makes one of those surfaces the limiter.
        - Evidence: the typical autoscroll formal bundle now reports
          `paint_host_widget_observed_deps_calls=252`, `paint_host_widget_observed_deps_empty_calls=244`,
          `paint_host_widget_observed_models_non_empty_calls=8`, and
          `paint_host_widget_observed_globals_non_empty_calls=2`, so empty dependency lookups are the
          dominant candidate. The presence-set fast path now short-circuits the empty case before the
          model/global map lookups.
        - Gates:
          `cargo nextest run -p fret-ui observed_deps_presence_tracks_rendered_and_touched_observations --no-fail-fast`;
          `cargo nextest run -p fret-ui -E 'test(~paint)' --no-fail-fast`.
        - Post-fast-path formal evidence: perf log entry `2026-05-16 03:51:42 +08:00`.
          Same-mouth `--reuse-launch --repeat 3 --warmup-frames 5` runs now pass for typical,
          complex wheel, and resize jitter:
          `target/fret-diag/editor-paint-contract-post-observed-deps-fastpath-20260516-typical-r3-cargo`,
          `target/fret-diag/editor-paint-contract-post-observed-deps-fastpath-20260516-complex-wheel-r3-cargo`,
          and
          `target/fret-diag/editor-paint-contract-post-observed-deps-fastpath-20260516-resize-jitter-r3-cargo`.
          Treat this as macOS M4 evidence only; keep baselines unchanged.
      - [ ] Attribute the remaining post-fast-path generic paint-widget aggregate cost before the
        next reversible optimization.
        - Current evidence: typical p95 paint/widget/Canvas/code-editor is `624/428/283/134us`;
          complex wheel is `838/627/481/317us`; resize jitter is `631/613/269/126us`.
        - Current host-widget residuals: observed model/global replay p95 stays around
          `24..25us` / `23..24us`, instance lookup p95 `41..46us`, collapse observations p95
          `51..56us`, with observed-deps empty calls still dominating count shape
          (`244..245` empty of `252..253` calls).
        - Next owner decision should be evidence-first: code-editor row work if the callback
          internals dominate, generic Canvas paint/cache if Canvas callback and wrapper diverge,
          or renderer text/encode/upload if renderer payload becomes the limiter. Do not start a
          broad row replay rewrite from the current passing row replay/cache evidence.
        - Added attribution summary in perf log entry `2026-05-16 03:59:52 +08:00`:
          `paint_widget_hotspot_summary.gap_to_code_editor_p95` now includes
          `windowed_surface_paint_callback_minus_us_total`,
          `windowed_surface_row_paint_minus_us_total`, and
          `windowed_surface_paint_callback_minus_row_paint`, plus
          `code_editor_windowed_surface_p95`.
        - Current decision: Canvas wrapper is not the owner (`2..4us` Canvas-minus-callback p95).
          `WindowedRowsSurface` callback internals remain the next owner candidate:
          callback-minus-row-paint is `118..149us` p95, while row-paint-minus-code-editor-total
          is only `13..21us` p95.
        - Local micro-slice: `ElementHostWidget::paint_impl` now short-circuits the container
          `focus_visible` global lookup unless the container is focused, matching text input/area
          paint behavior. This is a baseline-neutral traversal cleanup, not evidence for a
          threshold update.
      - [x] Inspect `WindowedRowsSurface` callback overhead before changing behavior.
        - Start from `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs`.
        - Preserve the existing code-editor row replay/cache evidence as a guardrail; do not
          optimize by invalidating row replay semantics or weakening renderer payload thresholds.
        - Follow-up evidence: `fret-diag` now reports
          `windowed_surface_paint_callback_minus_row_paint_per_row_ns` and
          `windowed_surface_row_callback_gap_per_row_ns`. On the overlay-disabled typical autoscroll,
          complex wheel, and resize jitter bundles, the p95 values are `65/62/62ns` per row and
          `79/48/72ns` per row respectively. That keeps the remaining gap in aggregate loop overhead
          territory rather than a separate row hot loop.
      - [ ] Continue non-RTX4090 local optimization triage without treating it as contract closeout.
        - Current policy: keep the Windows RTX4090 validation as the target-machine TODO below, but
          allow independent local slices when they have their own evidence and do not update checked-in
          baselines.
        - Current local smoke after the focus-visible short-circuit:
          `target/fret-diag/container-focus-visible-short-circuit-typical-r3/1778912115985/bundle.schema2.json`.
          `fretboard-dev diag stats` reports p95 total/prepaint/paint/paint_widget
          `769/221/524/314us`, Canvas exclusive p95 `176us`, renderer text/encode/upload p95
          `344/142/84us`, host instance/models/globals p95 `41/25/24us`, paint collapse p95
          `52us`, and row replay/store p95 `289/0`.
        - Local slice: `ObservationIndex::record` and `GlobalObservationIndex::record` now remove
          previous node entries when the new observation list is empty instead of retaining empty
          `by_node` entries for later view-cache collapse scans. Same-script local typical evidence
          in `target/fret-diag/empty-observation-record-fastpath-typical-r3` moves
          `paint_collapse_observations_time_us` p95 from `52us` to `17..18us`, while row
          replay/store remains `289/0`. Treat the run-1 total-frame outlier (`1715us`) as local
          scheduler/noise evidence and do not update baselines.
        - Local slice: paint now prepares an observed-deps presence snapshot once per `UiTree`
          paint pass, and `ElementHostWidget::paint_impl` skips the runtime empty-deps lookup
          path when that active snapshot says the element has no declarative model/global
          observations. Same-script local typical evidence in
          `target/fret-diag/paint-observed-deps-presence-snapshot-typical-r3/1778921262429/stats.json`
          moves `paint_host_widget_observed_models_time_us` / `paint_host_widget_observed_globals_time_us`
          p95 from `24/23us` to `4/4us`. This is a baseline-neutral host-widget traversal cleanup,
          not Windows RTX4090 closeout evidence.
        - Formal owner decision from the three-probe editor contract remains unchanged: the
          dominant residual is still `paint.widget` / Canvas aggregate work, with renderer text
          prepare visible but not yet the primary limiter. The next implementation slice should
          inspect remaining host-widget instance lookup plus paint cache / visual-bounds
          bookkeeping before any glyph/text-index residency or broad display-list rewrite.
        - Near-term local owner order:
          1. Re-run the three editor paint probes locally after any host-widget cleanup to confirm
             the typical-only smoke generalizes to complex wheel and resize jitter.
          2. Attribute the remaining outer paint traversal cost before changing behavior, especially
             `paint_host_widget_instance_lookup_time_us`, paint-cache key / replay bookkeeping, and
             visual-bounds flush. Consider scratch-map reuse or recording already-collapsed
             view-cache-root observations only if bundles keep this above noise.
          3. Audit policy/component focus-visible lookups in ecosystem crates and only short-circuit
             the obvious `focused && focus_visible` cases with targeted tests; this is a small
             traversal cleanup, not a core contract change.
          4. Refresh or close the older `ui-perf-paint-pass-breakdown-v1` notes before using them as
             a current execution source; the active editor-paint evidence now lives in this lane.
        - Do not split a new workstream for the above. Split a narrow follow-on only when the next
          slice becomes a hard contract or structural rewrite, such as true FrameArena/bump allocation,
          `WindowFrame.children` arena/slab storage, explicit view-cache paint-skip semantics, or an
          editor Canvas/row-fragment replay contract.
      - [ ] Stabilize the target-machine editor paint contract before closing P1.5.
        - Current execution policy: defer this as the Windows RTX4090 TODO while continuing independent local
          optimization slices. Do not treat local macOS evidence or dry-run plans as substitutes for the required
          target-machine validation and attribution directories.
        - Required artifact: Windows RTX4090 overlay-disabled validation for typical autoscroll, complex wheel, and
          code-editor resize jitter; use a deliberate re-seed path only if the validation evidence justifies it.
        - Latest local handoff audits: perf log entries `2026-05-16 14:46:00 +0800` and
          `2026-05-16 18:26:00 +0800` confirm the current macOS M4 workspace cannot produce the required closeout
          artifacts. The target-machine runner rejects non-dry-run execution on non-Windows hosts, and the verifier /
          closeout tools reject dry-run directories because they lack real validation/attribution `summary.json` files.
        - Clean Windows handoff plans:
          `target/fret-diag/editor-paint-contract-windows-handoff-validation-plan/validation-plan.json`,
          `target/fret-diag/editor-paint-contract-windows-handoff-attribution-plan/validation-plan.json`, and
          `target/fret-diag/editor-paint-contract-windows-handoff-closeout-plan.json`.
        - Target-machine runner:
          `python tools/perf/diag_editor_paint_contract_validate.py --date-tag <date>`.
          Use `--dry-run` on non-target hosts to inspect the command plan. Use a fresh `--date-tag` / `--out-dir` for
          non-dry-run validation; the runner rejects an existing non-empty output directory by default to avoid stale
          dry-run or failed-run artifacts. Run once without
          `--with-paint-perf` for the baseline-validation `failures=[]` artifact, then run
          `--with-paint-perf` only for the follow-up attribution pass. The runner collects the
          worst-bundle `diag stats --sort cpu_cycles --top 15 --json` output for each validation probe and
          checks that the required paint-widget, renderer, and paint-perf field groups are present.
          Non-empty `check.perf_thresholds.json.failures` is treated as a failed validation run.
        - After copying both validation directories back into the workspace, prefer the local closeout gate:
          `python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-<date> --attribution-dir target/fret-diag/editor-paint-contract-validate-<date>-attrib`.
          Use `tools/perf/diag_editor_paint_contract_verify_artifacts.py` only when the artifact-only check is needed
          without the repo-level closeout gates.
        - Post-sync verifier hard requirements:
          - both validation summaries must carry a non-empty `date_tag`;
          - stored commands must match the Windows validation shape: release `fretboard-dev.exe` /
            `fret-ui-gallery.exe`, standard prewarm/prelude hooks, `--reuse-launch` on direct `diag perf`, and the
            overlay-disabled env set;
          - baseline-validation direct `diag perf` commands must not include `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`;
          - the attribution directory must include paint-perf coverage, `code_editor_paint_perf`, and overlay-zero
            stats (`top_code_editor_torture_overlay_us=0` / `code_editor_paint_perf.max.us_torture_overlay=0`).
        - Required evidence: ordinary validation must provide `check.perf_thresholds.json` with
          `failures=[]`, worst-bundle `diag stats` summaries for paint/widget, code-editor paint perf, and
          renderer text/encode/upload. The artifact verifier projects these raw fields as per-probe
          `decision_inputs` so closeout can choose Canvas/paint, renderer text, or no-code-change from the synced
          artifacts. A deliberate re-seed path must additionally provide
          `selection-summary.json` plus no-threshold-loosening evidence unless a policy note intentionally
          justifies the reset.
        - Guardrail: do not update checked-in baselines from macOS M4 evidence and do not mark P1.5
          closed until the contract matrix and this TODO point at the target-machine artifacts.
    - [x] Add a stable “row op count” signal to diag snapshots (or reuse an existing one) so we can gate
      “we are rebuilding 500+ ops/frame” vs “we are replaying”.
      - Field: `code_editor.paint_perf.row_scene_ops_stored` in UI Gallery app snapshots and
        `code_editor_paint_perf.*.row_scene_ops_stored` in `fretboard diag stats --json`.
      - Evidence: perf log entry `2026-05-12` (`code editor row-scene stored-op signal`); gallery-dev typical
        autoscroll smoke bundle `target/fret-diag/codex-row-scene-ops-smoke-gallery-dev/1778538679777/bundle.schema2.json`
        reports frames `180`, sum/p50/p95/max `row_scene_ops_stored=90/0/1/1`.
    - [x] Decide the replay boundary:
      - Option A (component-level): `fret-code-editor` caches per-row paint ops and replays when inputs unchanged.
      - Option B (mechanism-level): add a general `CanvasPainter` op cache (keyed, bounded, frame-aware) that any
        component can use.
      - Decision (2026-05-12): do not start Option B from the current evidence. Keep the current row-scene replay
        boundary, and if a future near-threshold/failing editor stressor proves store/capture churn is the limiter,
        prototype Option A first as an editor-owned row payload boundary.
      - Evidence: complex wheel repeat=3 paint-detail run
        `target/fret-diag/perf-complex-editor-row-store-ops-v1/1778539097606/bundle.schema2.json` reports
        `row_scene_ops_stored` p50/p95/max `2/10/12`, p95 replay rows `288`, and worst top total `2601us`.
    - [ ] Ensure replay is correctness-safe:
      - invalidation keys include font stack, scale factor, wrap width bucket, theme/style, and selection/preedit
        geometry dependencies.
      - replayed ops preserve hit-testing / selection rect correctness (or explicitly opt-out).
    - [x] Add a “canvas replay hit rate” counter to `fretboard-dev diag perf --json` output for the editor probes.
      - Fields:
        `top_code_editor_rows_painted`, `top_code_editor_rows_scene_replayed`,
        `top_code_editor_rows_scene_stored`, `top_code_editor_row_scene_ops_stored`, and
        `top_code_editor_row_scene_replay_hit_rate_pct`.
      - Coverage: single-run `rows[]`, repeat `runs[]`, and repeat `stats{}` JSON output.
      - Evidence: perf log entry `2026-05-12` (`diag perf editor row-scene replay JSON fields`).
    - [ ] Tighten the `ui-code-editor-resize-probes` baseline once replay is real and stable.
  - Acceptance (initial):
    - `ui-code-editor-resize-probes` stays PASS (no regressions in P0 `ui-resize-probes`).
    - In the editor probes, `paint_widget_hotspots` no longer shows the editor `Canvas` dominating p95 paint.
- [ ] **P2 GPU vs CPU attribution**: make “GPU stall vs CPU work” obvious from diag bundles / captures.
  - [x] Deep-run editor resize jitter with `FRET_DIAG_RENDERER_PERF=1` to classify CPU vs renderer costs.
    - Evidence: perf log entry `2026-02-08` (commit `f1292f2f8`).

## Milestones

Execution plan:

- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-execution-plan.md`

### M0: Baseline + suite gates (make perf a contract)

- [ ] Decide Tier A / Tier B thresholds per script (initially “best-effort”, then tighten).
- [x] Decide what `--launch` represents (cold-start gate vs steady-state gate) and codify it.
  - `ui-gallery` + `--launch`: cold-start gate (mount + first interaction).
  - `ui-gallery-steady` + `--reuse-launch` + `--launch`: steady-state gate (post-mount interactions).
- [ ] Finalize the acceptance suite list (see `ui-perf-zed-smoothness-v1.md`) and keep it small.
  - Ensure it includes at least one editor-grade text surface (`ui-gallery-code-editor-torture-autoscroll-steady.json`).
- [x] Record initial baselines (one per machine profile) using `fretboard-dev diag perf --perf-baseline-out`.
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v1.json` (commit `50bfcc54`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v2.json` (see perf log entry).
    - v1 was slightly flaky on `ui-gallery-window-resize-stress-steady` `max_top_solve_us` when checked with repeat=3.
      v2 bumps headroom to 30% to reduce false positives.
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v4.json` (see perf log entry).
    - Includes the new `ui-gallery-hover-layout-torture-steady.json` script in the `ui-gallery-steady` suite.
    - v3 exists but is superseded by v4 (hover script cleanup to reduce cross-script state contamination).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v5.json` (see perf log entry).
    - Switches perf protocol to `FRET_DIAG_SCRIPT_AUTO_DUMP=0` to avoid per-step bundle dumps dominating I/O.
    - Supersedes v4 for perf gating; keep v4 only if you explicitly want “auto dump on” behavior for debugging.
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v6.json` (see perf log entry).
    - Includes pointer-move maxima in the baseline rows (newer perf protocol) and reflects the current steady-state
      costs of the menubar script after recent diagnostics/runtime changes.
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v7.json` (see perf log entry).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v8.json` (post-merge snapshot;
    evidence + drift notes in the perf log entry for commit `72e6c32df`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v9.json` (refresh after the
    post-merge editor regression fix; evidence + drift notes in the perf log entry for commit `0d8ad27ac`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v10.json` (refresh under the
    steady-state protocol: `--reuse-launch` + diagnostics envs pinned; evidence + drift notes in the perf log entry
    for commit `09ecac494`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v11.json` (adds the editor-grade
    autoscroll probe to the suite; evidence + drift notes in the perf log entry for commit `f21a0aa82`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v12.json` (pointer-move threshold slack/quantum stabilization; see perf log entry around 2026-02-06 12:36).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v13.json` (refresh after resize-event coalescing work; see perf log entry for commit `beb2fa315`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v14.json` (schema refresh including run-max hit-test-replay gate fields; evidence + drift notes in perf log entry 2026-02-06 20:12).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v15.json` (adds anti-noise threshold seeding metadata, policy header, and resize-script p95 seeding with interpolated percentile; evidence + drift notes in perf log entries 2026-02-06 21:05 and 2026-02-06 21:35).
- [x] Add a “how to run locally” snippet to the workstream doc (keep it copy/paste friendly).
- [ ] Create a “known-noise sources” section (thermal, background apps, debug vs release, shader compile).
- [x] Pick one canonical view-cache setting for the suite and enforce it via `--env` in scripts.
  - Candidate: `FRET_UI_GALLERY_VIEW_CACHE=1` + `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`.
- [x] Add a dedicated P0 resize probe suite + gate runner (so resize regressions are always caught).
  - Suite: `ui-resize-probes` (`tools/diag-scripts/ui-gallery-window-resize-stress-steady.json` +
    `tools/diag-scripts/ui-gallery-window-resize-drag-jitter-steady.json`).
  - Baseline: `docs/workstreams/perf-baselines/ui-resize-probes.macos-m4.v3.json`.
  - Seed policy preset: `docs/workstreams/perf-baselines/policies/ui-resize-probes.v1.json`.
  - Gate runner: `tools/perf/diag_resize_probes_gate.sh`.
- [x] Add an editor-grade resize jitter probe suite (so resize work stays bounded on text-heavy surfaces).
  - Suite: `ui-code-editor-resize-probes` (`tools/diag-scripts/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`).
  - Baseline: `docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json`.
  - Seed policy preset: `docs/workstreams/perf-baselines/policies/ui-code-editor-resize-probes.v1.json`.
  - Gate runner: `tools/perf/diag_resize_probes_gate.sh --suite ui-code-editor-resize-probes`.
- [x] Ensure `diag perf --json` still emits the JSON payload even when perf thresholds fail (so gate triage can
  resolve worst bundles without rerunning).
- [x] Fix `tools/perf/diag_resize_probes_gate.sh` to record non-zero attempt exit codes correctly (for trustworthy
  gate summaries and downstream triage).
- [x] Fix `fret-perf-workflow` gate triage helper:
  - robust JSON payload extraction (skip leading logs without awk regex pitfalls),
  - support absolute gate out-dirs (worktrees) by resolving attempt paths relative to the summary out-dir.
- [x] Create a commit-addressable perf log:
  - `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`
- [x] Add a helper to append suite results to the log:
  - `tools/perf/perf_log.py`
- [x] Extend `tools/perf/perf_log.py` to include churn signals (top frame p95/max) alongside CPU breakdown.
  - Signals: text atlas uploads/evictions, intermediate pool peak bytes, intermediate pool evictions.
  - Implemented by `feat(perf): include churn signals in perf_log` (commit `76d2dfd6`).
- [x] Record an initial suite run in the log (repeat=7).
- [x] Add a steady-state suite and reuse-launched-process support:
  - `fretboard-dev diag perf ui-gallery-steady --reuse-launch --launch -- cargo run -p fret-ui-gallery --release`
- [x] Record a `ui-gallery-steady` baseline in the perf log (repeat=7, `--reuse-launch`).
  - See `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for commit `686bebe1`.
- [ ] Keep the canonical steady baseline up to date when diagnostics instrumentation changes (avoid "false regressions").
  - Current: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v23.json`.
- [x] Stabilize view-cache key to avoid resize-driven `cache_key_mismatch`.
  - Implemented by `perf(fret-ui): stabilize view-cache key` (commit `b6f1b580`).
- [x] Add a resize-smoothness knob for scroll extents: defer unbounded probes while the viewport is resizing.
  - Implemented by `perf(fret-ui): defer unbounded scroll probe on resize` (commit `05d2d56c`).
  - Env: `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1`
  - Debounce: `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES` (default: 2)
- [x] Add correctness gates for the resize + scroll probe policy:
  - Scroll offset stability gate: `--check-scroll-offset-stable <test_id>` (commit `6c248d9e1`).
  - Scrollbar thumb geometry validity gate: `--check-scrollbar-thumb-valid all` (commit `e20637f92`).
- [ ] Decide whether scroll unbounded-probe deferral should become the default (remove env gating) and
  update the canonical perf suite env set accordingly.
- [x] Export view-cache reuse “miss reasons” as perf-visible counters (so regressions are explainable).
  - Implemented by `feat(diag): export view-cache reuse miss counters` (commit `43f9c73e`).
- [x] Export a coarse layout-phase breakdown (so `layout_time_us` is explainable in bundles and stable-frame fast paths).
  - Add: `layout_collect_roots_time_us`, `layout_invalidate_scroll_handle_bindings_time_us`,
    `layout_expand_view_cache_invalidations_time_us`, `layout_request_build_roots_time_us`,
    `layout_pending_barrier_relayouts_time_us`, `layout_repair_view_cache_bounds_time_us`,
    `layout_contained_view_cache_roots_time_us`, `layout_collapse_layout_observations_time_us`,
    `layout_prepaint_after_layout_time_us`, `layout_skipped_engine_frame`.
  - Follow-up: include `layout_roots_time_us` in `fretboard-dev diag stats` / `diag perf --json` payloads (commit `366efd769`).
  - Wire into: `fretboard-dev diag stats --json` so a worst bundle can be inspected without manual JSON digging.
  - Implemented by `feat(diag): export layout phase breakdown` (commit `b02744a8`).
- [x] Export initial paint-pass breakdown metrics (to disprove/confirm “paint-cache replay is the hotspot”).
  - Adds: `paint_cache_replay_time_us`, `paint_cache_bounds_translate_time_us`,
    `paint_cache_bounds_translated_nodes`, `paint_record_visual_bounds_time_us`,
    `paint_record_visual_bounds_calls`.
  - Implemented by `feat(diag): add paint pass breakdown metrics` (commit `f2bee87a`).
  - Tracking: `docs/workstreams/ui-perf-paint-pass-breakdown-v1/ui-perf-paint-pass-breakdown-v1.md`
- [x] Export top inclusive layout hotspots (to complement exclusive-only `debug.layout_hotspots[]`).
  - Field: `debug.layout_inclusive_hotspots[]`
  - Implemented by `feat(diag): add inclusive layout hotspots` (commit `69111ebde`).
- [x] Export initial paint micro-breakdown timers (paint-all plumbing).
  - Adds: `paint_input_context_time_us`, `paint_scroll_handle_invalidation_time_us`,
    `paint_collect_roots_time_us`, `paint_publish_text_input_snapshot_time_us`,
    `paint_collapse_observations_time_us`.
  - Implemented by `feat(diag): add paint micro-breakdown timers` (commit `b20a1280`).
  - Tracking: `docs/workstreams/ui-perf-paint-pass-breakdown-v1/ui-perf-paint-pass-breakdown-v1.md`
- [x] Export paint node breakdown timers (paint-cache key/hit checks, widget paint, observation recording).
  - Adds: `paint_cache_key_time_us`, `paint_cache_hit_check_time_us`, `paint_widget_time_us`,
    `paint_observation_record_time_us`.
  - Implemented by `feat(diag): add paint node breakdown timers` (commit `c512be81`).
  - Tracking: `docs/workstreams/ui-perf-paint-pass-breakdown-v1/ui-perf-paint-pass-breakdown-v1.md`
- [ ] Keep `diag perf` runs comparable by splitting “gate checks” vs “deep profiling”:
  - Gate check (CPU regressions): keep `FRET_DIAG_RENDERER_PERF` off (avoid instrumentation overhead).
  - Deep profiling (churn / GPU triage): turn `FRET_DIAG_RENDERER_PERF=1` on and record churn tables in the log.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entries on 2026-02-04 show the delta.

### M1: Frame data structures (hashing → dense)

Primary targets (highest leverage):

- [x] Refactor `WindowFrame` stores to avoid per-frame `HashMap` churn:
  - `crates/fret-ui/src/declarative/frame.rs` (`WindowFrame.instances`, `WindowFrame.children`)
  - Landed as `slotmap::SecondaryMap<NodeId, ...>` (commit `448c34ad`).
- [x] Avoid rewriting `WindowFrame.children` when the child list is unchanged (reduce per-frame `Arc<[NodeId]>` allocations).
  - Implemented by `perf(fret-ui): skip unchanged window frame children` (commit `cce827ad`).
- [x] Avoid cloning child lists when calling `UiTree::set_children*` from declarative mount (reduce per-frame heap churn).
  - Implemented by `perf(fret-ui): avoid cloning child lists in mount` (commit `089bac9b`).
- [ ] Replace `Arc<[NodeId]>` for `WindowFrame.children` with a reuse-friendly representation.
  - Candidate: store `Vec<NodeId>` in a slab/arena and reference by index + generation.
- [x] Replace invalidation “visited”/scratch `HashMap<NodeId, u8>` with generation-stamped tables:
  - `crates/fret-ui/src/tree/mod.rs` invalidation propagation caches.
  - Implemented by `perf(fret-ui): generation-stamp invalidation propagation` (commit `a540829e`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entries for commit `a540829e`.
- [x] Avoid per-dispatch `HashMap<NodeId, u8>` churn when deduplicating invalidations during input dispatch.
  - Use the existing generation-stamped `InvalidationDedupTable` for dispatch-time invalidation dedup.
  - Implemented by `perf(fret-ui): reuse invalidation dedup in dispatch` (commit `bcb329e6`).
- [x] Make the layout-engine request/build phase less hashing-heavy (dense tables).
  - Convert `TaffyLayoutEngine::{node_to_layout,styles,children,parent}` to `slotmap::SecondaryMap`.
  - Evidence: perf log entry `2026-02-09 16:10:00` for commit `e9ea4522a` in
    `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` (`layout_request_build_roots_time_us` improved).
- [x] Experiment: memoize wrapper-chain fill scan during request/build.
  - Result: passes the resize gates, but regresses `layout_request_build_roots_time_us` on drag-jitter.
  - Evidence: perf log entry for commit `96661c49c` in `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`.
  - Next: prefer the broader layout-engine M1 refactor (hashing → dense tables) instead of per-frame `HashMap` caches.
- [ ] Ensure deterministic ordering is preserved where diagnostics rely on it (bundle stability).

Perf acceptance:

- [ ] `ui-gallery-overlay-torture.json`: p95 total improves; invalidation nodes/calls do not regress.
- [ ] `ui-gallery-virtual-list-torture.json`: tail latency improves or stays flat.
- [x] Investigate post-`a540829e` suite deltas (noise vs real regression) and decide next step:
  - If real: profile invalidation propagation micro-costs and consider alternative dense map strategy (or env gating).
  - If noise: standardize suite runs on explicit `--dir` and pin a baseline via `--perf-baseline-out`.
  - Result: A/B rerun at `448c34ad` is within noise vs the current baseline (see perf log).

### M2: Allocation model (per-frame scratch arena)

- [ ] Introduce a `FrameArena` (or equivalent) for UI runtime scratch allocations.
  - Reference: `repo-ref/zed/crates/gpui/src/arena.rs`.
- [x] Reuse a small set of per-frame scratch buffers to reduce allocator churn.
  - `perf(fret-ui): reuse frame scratch buffers` (commit `a39e79c4`).
- [x] Reuse view-cache GC “keep-alive” scratch collections (HashSet/Vec) to reduce per-frame allocations.
  - `perf(fret-ui): reuse view-cache keepalive scratch` (commit `cb3ff2d9`).
  - A/B gate: `perf(fret-ui): gate view-cache keepalive scratch` (commit `968305b9`)
    - `FRET_UI_VIEW_CACHE_KEEPALIVE_SCRATCH_DISABLE=1` disables scratch reuse.
  - Status: A/B is within noise on:
    - code editor autoscroll (`tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`)
    - view-cache toggle perf steady (`tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json`)
    - overlay torture steady (`tools/diag-scripts/ui-gallery-overlay-torture-steady.json`)
    (see perf log entries for `968305b9`).
- [x] Convert at least 2 hot scratch paths to arena-backed allocation (scratch reuse, v0):
  - Semantics snapshot traversal scratch (stack + visited).
  - GC reachability scratch sets / traversal stack in mount/GC.
  - Implemented by `perf(fret-ui): reuse GC/semantics scratch via frame arena` (commit `3d6e2431`).
  - Evidence: perf log entry for `1b0364e9` (exports `top_frame_arena_*` counters).
- [x] Export “frame arena scratch” counters into perf-visible diagnostics:
  - Implemented by `feat(diag): export frame arena scratch stats` (commit `fe0ad7c3`).
  - Fix: `fix(fret-ui): restore keepalive scratch after diagnostics` (commit `1b0364e9`).
- [x] Remove per-scope `HashMap` churn during element ID derivation (callsite counters).
  - Implemented by `perf(fret-ui): remove callsite counter HashMap churn` (commit `2dd36fde`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for `2dd36fde`.
- [x] Pool declarative element child buffers (`Vec<AnyElement>`) across frames (arena-adjacent, v0).
  - Implemented by `perf(fret-ui): pool element children vectors` (commit `07a4c252`).
  - Perf-visible counters exported by `feat(diag): export element build pool counters` (commit `cbcd81ed`).
  - Follow-up: `perf(fret-ui): make element children vec pool LIFO` (commit `693a55b0`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for `693a55b0`.
- [x] Validate element children vec pool steady-state behavior on editor-class pages.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entries for:
    - `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json` (0 misses; paint-dominant).
    - `tools/diag-scripts/ui-gallery-chrome-torture-steady.json` (0 misses; very light total).
- [ ] Add an optional allocation counter hook for local profiling (feature-gated).
  - Keep it portable; do not require a global allocator swap for normal builds.

Correctness acceptance:

- [x] Existing `cargo nextest run -p fret-ui` remains green.
  - Evidence: passed locally after `perf(fret-ui): skip layout-engine rebuild on stable frames` (commit `1905de1e`).
- [ ] `fretboard-dev diag repro ui-gallery` smoke suite passes.

### M3: Hit testing (bounds tree / spatial index)

- [x] Implement a bounds tree built during prepaint per hit-testable layer root.
  - Implemented by `perf(fret-ui): add bounds tree hit-test index` (commit `75a9fde3`).
  - Note: current implementation supports axis-aligned transforms only (no rotation/shear).
- [x] Route pointer move/down hit-testing through the bounds tree for large trees.
  - Implemented by `75a9fde3` (hooked via `UiTree::hit_test_layers_cached`).
- [x] Define “fallback” conditions clearly (transforms, clips, non-axis-aligned bounds).
  - Supports `clips_hit_test=false` (overflow-visible hit testing) by propagating the ancestor clip (instead of
    disabling the index for the entire layer).
  - Disabled for a layer if any transform is non-axis-aligned (`b!=0` or `c!=0`).
  - Env toggles:
    - `FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE=1` disables the index.
    - `FRET_UI_HIT_TEST_BOUNDS_TREE_MIN_RECORDS` (default: 256) gates building for small trees.
- [x] Add a pointer-move stress gate that fails on dispatch/hit-test regressions.
  - Use:
    - `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
    - `--max-pointer-move-dispatch-us`, `--max-pointer-move-hit-test-us`,
      `--max-pointer-move-global-changes` (fretboard `diag perf`)
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for commit `6da92d3d`.
  - TODO: Investigate occasional flakiness when running this gate with `--reuse-launch --repeat 7`
    (observed: a run gets stuck early in the script, e.g. `set_window_inner_size`).
    Short-term workaround: use `--repeat 3` for local iteration and keep a stable Tier B gate at repeat=7 once the
    harness is robust.
    - Evidence: a repeat=7 run completed when launching a prebuilt binary
      (`--launch -- target/release/fret-ui-gallery`); see the perf log entry for commit `b83ae7a5`.
- [x] Make pointer-move gate outliers explainable (include snapshot id for pointer-move maxima).
  - Implemented by `feat(diag): include pointer-move max frame ids in triage` (commit `c2ea017b`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for commit `c2ea017b`.
- [x] Eliminate changed-but-unobserved model churn on pointer-move frames.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for commit `dd1a22e8` shows pointer-move
    frames with `changed_models=2` and `propagated_model_change_unobserved_models=2` while remaining paint-only.
  - Fixed in `perf(ui-gallery): avoid per-frame undo/redo model churn` (commit `eb6c6b2e`).
  - Goal: pointer-move frames should have `changed_models=0` unless the interaction explicitly updates observed state.
  - Candidate fix: move per-frame pointer-move bookkeeping out of `Model` updates into a window-scoped scratch store
    (or a “set-if-changed” model update discipline similar to the global churn fix).
- [x] Add a dispatch/hit-test time metric to diagnostics so we can gate pointer-move cost explicitly.
  - Implemented by `perf(diag): expose dispatch and hit-test timing` (commit `4b0be50e`).
  - Adds new `fretboard-dev diag perf --sort dispatch|hit_test` modes and exports:
    - `top_dispatch_time_us`, `top_hit_test_time_us`
    - `top_dispatch_events`, `top_hit_test_queries`
- [x] Add a dedicated hit-test drag stress script (high pointer event density).
  - Script: `tools/diag-scripts/ui-gallery-hit-test-drag-sweep-steady.json`
  - Use with: `fretboard-dev diag perf ... --sort hit_test`
- [x] Add a multi-frame pointer-move sweep step for realistic hover/hit-test measurements.
  - Implemented by `perf(diag): add move_pointer_sweep script step` (commit `4941baa1`).
  - Scripts:
    - `tools/diag-scripts/ui-gallery-hit-test-move-sweep-steady.json`
    - `tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json`
- [x] Find (or construct) a workload where `top_hit_test_time_us` is a meaningful slice of the frame budget.
  - Page: `apps/fret-ui-gallery/src/ui/previews/pages/harness/hit_test_torture.rs` (`hit_test_torture`)
  - Script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
  - Named suite: `perf-ui-gallery-hit-test-torture-steady`
  - Current via-nav script keeps gallery chrome in setup but resets diagnostics before the measured sweep.
  - Evidence + metrics: see `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entries after commit `811101c3`.
- [x] Record baseline numbers for the two “realistic move sweep” probes:
  - Data table sweep: `tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json`
  - Stripes torture (via nav): `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-via-nav-steady.json`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entries on 2026-02-04 (commit `9b2f9fc9`).
- [x] Add a smaller torture script variant to make scaling runs practical (avoid 10GB+ bundles).
  - Script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-mini.json`
  - Implemented by `feat(diag-scripts): add mini hit-test torture sweep` (commit `1b3d2db3`).
  - Use: `FRET_DIAG_SCRIPT_AUTO_DUMP=0` + `FRET_DIAG_SEMANTICS=0` + `FRET_DIAG_MAX_SNAPSHOTS=120`.
- [x] Export cached-path hit-test reuse counters (to measure whether the fast path helps).
  - Counters:
    - `debug.stats.hit_test_path_cache_hits`
    - `debug.stats.hit_test_path_cache_misses`
  - Implemented by `feat(diag): track hit-test path-cache reuse` (commit `55dd923d`).
- [ ] Investigate why the torture workload is still layout/prepaint-dominant on the sampled frames.
  - Goal: create (or tune) a variant where pointer moves are paint-only and hit-test cost is isolated.
  - Hypotheses:
  - TODO: Use the new bounds-tree “work” counters to determine whether `hit_test_time_us` tails are algorithmic or
    wall-time noise:
    - `debug.stats.hit_test_bounds_tree_nodes_visited`
    - `debug.stats.hit_test_bounds_tree_nodes_pushed`
    - Implemented by `feat(fret-ui): track bounds-tree query work in debug stats` (commit `913ee260`).
    - hover policy triggers layout
    - retained tree has a per-frame relayout
    - noise elements invalidate layout
    - diagnostics/script harness accidentally forces expensive work every frame (e.g. semantics refresh)
  - Progress:
    - `1905de1e` reduces this probe's `layout_time_us` max from ~74ms → ~31ms by skipping layout-engine rebuild on stable frames.
    - `prepaint_time_us` remains ~9–10ms and `hit_test_time_us` stays measurable; next isolate remaining ~20ms inside `layout_all_with_pass_kind`.
    - `470708b2` reduces the same probe's top frame max total from ~56ms → ~39ms by gating semantics snapshot refresh
      to only the frames that actually need selector resolution (3/201 frames in the inspected bundle).
    - `ba3fd15d` fixes a diagnostics accounting bug (layout time no longer double-counts prepaint).
    - `6cca2cf1` removes prepaint rebuild work on layout-stable frames by reusing hit-test bounds trees:
      - `top_prepaint_time_us` drops to ~0 for the probe's worst frames.
      - Pointer-move frames become paint-only with `layout_time_us ~ 0` and `prepaint_time_us ~ 0` (see perf log entry).
  - Deliverable: a new/updated script + a log entry demonstrating low `layout_time_us` while `hit_test_time_us` remains measurable.
  - [x] Add hit-test micro timers so tail latency is attributable to concrete work.
    - Exports (per-frame, accumulated across hit-test queries):
      - `hit_test_cached_path_time_us`
      - `hit_test_bounds_tree_query_time_us`
      - `hit_test_candidate_self_only_time_us`
      - `hit_test_fallback_traversal_time_us`
    - Implemented by `feat(diag): break down hit-test timing` (commit `763bf8e7`).
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entries for commits `763bf8e7` and `8bc15eda`.
  - [x] Remove cached-path overhead when bounds-tree is enabled.
    - Implemented by `perf(fret-ui): skip cached-path hit-test under bounds-tree` (commit `8bc15eda`).
    - Result: pointer-move `hit_test_time_us` p50 ~575us → ~3us on the stripes torture probe.
  - [x] Export a coarse dispatch sub-step timing breakdown for pointer-move triage.
    - Exports (per-frame, accumulated across the frame’s dispatch work):
      - `dispatch_hover_update_time_us`
      - `dispatch_scroll_handle_invalidation_time_us`
      - `dispatch_active_layers_time_us`
      - `dispatch_input_context_time_us`
      - `dispatch_event_chain_build_time_us`
      - `dispatch_widget_capture_time_us`
      - `dispatch_widget_bubble_time_us`
      - `dispatch_cursor_query_time_us`
      - `dispatch_pointer_move_layer_observers_time_us`
    - Wired into: `fretboard-dev diag stats --json` (so a worst bundle can be inspected without manual JSON digging).
    - Implemented by `feat(diag): break down dispatch timing` (commit `7fa76fd5`).
    - Evidence: perf log entry for commit `7fa76fd5`.
  - [x] Attribute dispatch time by dispatched event class (pointer vs timer vs other).
    - Exports (per-frame, accumulated across the frame’s dispatch work):
      - `dispatch_pointer_events`, `dispatch_pointer_event_time_us`
      - `dispatch_timer_events`, `dispatch_timer_event_time_us`
      - `dispatch_other_events`, `dispatch_other_event_time_us`
    - Wired into: `fretboard-dev diag stats --json` (bundle triage without manual JSON digging).
    - Implemented by `feat(diag): attribute dispatch time by event class` (commit `5ab4ba71`).
    - Evidence: perf log entry for commit `5ab4ba71`.
  - [x] Reduce timer-driven dispatch work during pointer-move workloads.
    - Why: In the stripes pointer-move probe, the “dispatch gap” was primarily **timer event dispatch** (not pointer
      routing). On the worst pointer-move frame, `dispatch_timer_event_time_us` accounted for ~95%+ of `dispatch_time_us`.
    - Root cause: ui-gallery’s dev-only config polling (`with_config_files_watcher(...)`) installs a repeating global
      timer, and the timer could co-occur with scripted pointer-move frames.
    - Deliverable:
      - Timer routing attribution exported (commit `98ca4fe3`).
      - Harness runs avoid config watcher timer traffic (commit `06feeb41`).
      - Evidence: perf log entries for commits `98ca4fe3` and `06feeb41` (p95 dispatch drops to ~tens of microseconds).
    - Remaining follow-ups (generalizing beyond the ui-gallery harness):
      - [ ] Make “background timers” avoid the UI dispatch hot path by default (or run them out-of-band).
      - [ ] Add a configurable “timer budget / priority” contract so non-UX-critical timers cannot steal time from
        interactive input frames.
  - A/B experiments:
    - [x] Run the pointer-move gate with `FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE=1` and record:
      - `hit_test_time_us` distribution, and
      - `hit_test_path_cache_hits/misses` hit rate.
      - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for commit `8bc15eda` (gate fails expectedly).
    - [ ] Sweep `FRET_UI_HIT_TEST_BOUNDS_TREE_MIN_RECORDS` to find the break-even point (small trees vs index build).

Perf acceptance:

- [ ] Pointer-move heavy cases should stay paint-only (no layout) unless explicitly required.
- [ ] Hit-test CPU time should be bounded as node count scales.
- [x] Ensure the perf log captures pointer-move dispatch/hit-test costs (not just “top frame” totals).
  - Today, `perf_log.py` reports “top frame” metrics for each run, which can show `dispatch=0` for probes
    where the worst total frame is a non-dispatch settle/selector frame.
  - `tools/perf/perf_log.py` now emits a derived “Pointer-move frames” section by scanning the run bundles and
    summarizing per-run maxima over frames where `dispatch_events > 0`.
- [x] Eliminate changed-but-unobserved global churn in hover-only pointer-move probes.
  - Current hotspots reported by `fretboard-dev diag stats`: `WindowInputContextService`,
    `WindowCommandActionAvailabilityService` (often changed but unobserved).
  - Goal: reduce pointer-move dispatch tails by making these globals “notify only on actual value change”
    (or avoid publishing them every frame unless explicitly needed).
  - Implemented by `perf(fret-ui): avoid global churn on hover moves` (commit `d4adf37f`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for `d4adf37f`
    (`dispatch_time_us` run-max p95 drops from ~4.1ms → ~1.2ms; `snapshots_with_global_changes` becomes 0).

### M7: Renderer primitive profiling (bottom-up)

- [x] Add renderer perf logging to UI gallery (primitive-level signals).
  - Enable: `FRET_UI_GALLERY_RENDERER_PERF=1`
  - Optional pipeline breakdown: `FRET_RENDERER_PERF_PIPELINES=1`
  - Goal: provide low-level “are we draw-call/pipeline-switch bound?” signals before deeper refactors.
- [x] Add a short “profiling playbook” that links `diag perf` → renderer perf → Tracy → RenderDoc.
  - `docs/workstreams/standalone/ui-perf-renderer-profiling-v1.md` (commit `22671e06`)
- [x] Export renderer perf snapshots into diagnostics bundles for perf log correlation.
  - Data lands in `bundle.json` under `.windows[].snapshots[].debug.stats.renderer_*` (commit `0e4928fe`).
  - `fretboard-dev diag stats/perf` supports sorting by renderer metrics (commit `cf8975ca`).
- [x] Export renderer churn metrics (text atlas + intermediate pool) into bundles and wire them into `fretboard`.
  - Commits: `feat(render): add text atlas + intermediate churn perf stats` (`d10cac5a`) +
    `feat(fretboard): add renderer churn sort modes` (`c9a8b168`).
  - Text atlas (per-frame signals): `renderer_text_atlas_revision`, `renderer_text_atlas_upload_bytes`,
    `renderer_text_atlas_evicted_pages`, `renderer_text_atlas_resets` (and related counters).
  - Intermediate pool (per-frame signals): `renderer_intermediate_peak_in_use_bytes`,
    `renderer_intermediate_pool_evictions` (and related counters).
  - New sort modes:
    - `atlas_upload_bytes`, `atlas_evicted_pages`, `intermediate_peak_bytes`, `pool_evictions`
- [ ] Add a GPU-time signal (where supported) to separate “CPU is fine” vs “GPU stalls”.
  - Candidate: timestamp queries in the renderer + export `gpu_render_us` (best-effort).
  - If unsupported on a backend, export `None` and keep the field stable in the bundle schema.
- [ ] Establish per-script renderer complexity budgets (to prevent silent GPU regressions).
  - Track at minimum: `renderer_draw_calls`, `renderer_pipeline_switches`, `renderer_bind_group_switches`,
    `renderer_scissor_sets`, and `renderer_text_atlas_upload_bytes`.
  - Add at least one acceptance script that is renderer-heavy (effects/blur, large text surface, SVG churn).
- [ ] Make RenderDoc captures repeatable for the acceptance scripts.
  - Pin marker names and a canonical `--renderdoc-after-frames` per script so “capture the hitch” is low-friction.

### M7.1: Renderer churn correlation (tail latency)

Goal:
- Turn “jank” into a correlation between **slow frames** and a **churn signature** (GPU-side or resource-side),
  and then close that churn.

TODO:

- [x] Add a deterministic workload/script that actually exercises blur/effects so intermediate pool counters become non-zero.
  - Script: `tools/diag-scripts/ui-gallery-effects-blur-torture-steady.json`
  - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` (entry for `effects_blur_torture`).
- [x] Add an eviction stress variant to force intermediate pool churn for correlation work.
  - Script: `tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json`
  - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture`
  - Budget override: `FRET_UI_GALLERY_RENDERER_INTERMEDIATE_BUDGET_BYTES=20971520` (20MB)
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` (pool evictions > 0).
- [x] Add additional churn accounting beyond text atlas (non-text uploads):
  - Bundles now export (best-effort) non-text texture upload counters:
    `renderer_svg_upload_bytes`, `renderer_svg_uploads`,
    `renderer_image_upload_bytes`, `renderer_image_uploads`.
  - Commits: `d01d3190` + `4bade395` + `dfbc02d3` (workload). Evidence:
    `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for commit `dfbc02d3`.
  - Harness/script:
    - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=svg_upload_torture`
    - Script: `tools/diag-scripts/ui-gallery-svg-upload-thrash-steady.json`
    - Budget override: `FRET_UI_GALLERY_SVG_RASTER_BUDGET_BYTES=262144` (256KB)
- [x] Add an eviction stress protocol for intermediate pool churn correlation.
  - Env: `FRET_UI_GALLERY_RENDERER_INTERMEDIATE_BUDGET_BYTES=20971520` (20MB) to force pool evictions.
  - Script: `tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json`
  - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` (entry for `effects_blur_thrash`).
- [ ] Extend churn accounting beyond uploads:
  - [x] SVG raster cache occupancy + eviction counts (to distinguish warmup vs thrash).
    - Commits: `6bd82329` + `5f7e4fd0` + `3d1510a7`
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for commit `3d1510a7`
      (see `svg_cache_misses` / `svg_evictions` columns).
  - [x] Intermediate pool lifecycle churn signals (alloc/reuse/release/free bytes/texture counts + budget/in_use/peak).
    - Commit: `52f555d5`
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for commit `52f555d5`.
  - [ ] Path/MSAA per-pass churn (uploads/resolves/temporary targets) beyond the pooled intermediate counters.
  - [x] Reduce intermediate pool housekeeping overhead by enforcing budget once per frame (instead of per release).
    - Commit: `3b792646`
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for commit `3b792646`.
- [x] Replace keyed repaint forcing with a representative invalidation-driven workload.
  - The legacy `svg_upload_torture` harness keys the Canvas subtree by frame to bypass paint-cache replay.
  - Added an invalidation-driven scroll workload that uses wheel input to shift the VirtualList window:
    - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=svg_scroll_torture` (commit `dd8bc0f8`)
    - Script: `tools/diag-scripts/ui-gallery-svg-scroll-thrash-steady.json`
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for commit `dd8bc0f8`.
- [x] Standardize “churn triage checklist” in the perf log template:
  - `tools/perf/perf_log.py` now emits churn + intermediate pool lifecycle tables and includes captured stdout paths.
  - Commit: `2c40a3fb`
- [x] Keep ADRs and audits in sync with the diagnostics bundle schema.
  - Update ADR 0159 bundle/export notes when schema changes (renderer counters, script steps, screenshot wiring).
  - Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` evidence and gaps when tooling contracts change.

### M4: Windowed surfaces (prepaint-driven visible windows)

- [x] Pick the first “editor-class” migration target: **Option A (VirtualList)**.
  - Rationale: fastest path to validate retained prepaint-window behavior and rerender suppression under wheel traffic.
  - Evidence: `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-crossing-steady.json`,
    `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entries 2026-02-07 00:46 and 2026-02-07 00:56.
- [ ] Reduce editor-class per-frame scene construction when scrolling/animating.
  - Baseline hotspot: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json` can be dominated by
    `paint_widget_hotspots kind=Canvas` (see perf log entry 2026-02-05 15:43:55).
  - Goal: translate/replay cached ranges where possible instead of re-emitting large display lists each frame.
- [ ] Ensure cache-root reuse remains stable under steady scroll/pan.
- [x] Suppress avoidable non-retained prefetch rerenders on steady wheel crossing.
  - Change: `crates/fret-ui/src/tree/prepaint.rs` now disables preemptive/forced prefetch shifts for
    non-retained + view-cache path while visible range remains covered by the rendered overscan envelope.
  - Non-retained sample (`FRET_UI_GALLERY_VLIST_RETAINED=0`, 3 runs):
    - before: `prefetch=1`, `non_retained=1` per run
    - after: `prefetch=0`, `non_retained=0` per run
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-07 01:04.
- [x] Add a “window boundary crossing” probe script for retained VirtualList scrolling.
  - Script: `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-crossing-steady.json`
  - Sampling status: with `FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`,
    `FRET_UI_GALLERY_VLIST_MINIMAL=1`, runs `r3..r6` show `total_shifts=1`, `prefetch=1`, `escape=0`, `non_retained=0`.
- [x] Promote the boundary-crossing probe into a stable acceptance gate recipe (repeat runs + threshold rationale).
  - Gate runner: `tools/perf/diag_vlist_boundary_gate.sh`
  - Validation summary: `target/fret-diag-codex-vlist-boundary-gate-r1/summary.json` (`runs=3`, `run_failures=0`, `pass=true`).

Perf acceptance:

- [ ] `ui-gallery-virtual-list-torture.json`: steady scroll should avoid cache-root rerender in most frames.
- [x] `ui-gallery-virtual-list-window-boundary-crossing-steady.json`:
  - Retained gate target: `prefetch<=3`, `escape<=0`, `non_retained<=0`
  - Command profile: enable view-cache env (`FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`) and run `tools/perf/diag_vlist_boundary_gate.sh --runs 3`.
- [x] `ui-gallery-virtual-list-window-boundary-crossing-steady.json` (non-retained fallback profile):
  - Run profile: add `FRET_UI_GALLERY_VLIST_RETAINED=0`
  - Current sampled expectation (3 runs): `prefetch=0`, `escape=0`, `non_retained=0`
- [x] Add strict non-retained fallback gate and cache-key budgets.
  - Gate runner: `tools/perf/diag_vlist_boundary_gate.sh` now supports
    `--retained`, `--max-cache-key-mismatch`, `--max-needs-rerender`.
  - Validation summary: `target/fret-diag-codex-vlist-boundary-nonretained-gate-r1/summary.json`
    (`runs=3`, `pass=true`, `prefetch=0`, `escape=0`, `non_retained=0`,
    `cache_key_mismatch_max=0`, `needs_rerender_max=0`).
- [x] Add non-retained boundary stress probe and strict gate recipe.
  - Script: `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-nonretained-stress-steady.json`
  - Gate command:
    `tools/perf/diag_vlist_boundary_gate.sh --runs 3 --script tools/diag-scripts/ui-gallery-virtual-list-window-boundary-nonretained-stress-steady.json --retained 0 --prefetch-max 0 --escape-max 0 --non-retained-max 0 --max-cache-key-mismatch 0 --max-needs-rerender 0`
  - Validation summary: `target/fret-diag-codex-vlist-boundary-nonretained-stress-gate-r1/summary.json` (`pass=true`, `run_failures=0`).
- [ ] `ui-gallery-code-view-scroll-refresh-baseline.json`: no hitch spikes after warmup.
- [x] `ui-gallery-code-editor-torture-autoscroll-steady.json`: eliminate the post-merge Canvas paint hotspot.
  - Root cause: accidental per-row `Theme` clone in syntax paint (allocator churn).
  - Fix: `perf(code-editor): avoid per-row Theme clone in syntax paint` (commit `0d8ad27ac`).
  - Evidence + numbers: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for 2026-02-06 (commit `0d8ad27ac`).
  - Follow-up: still track tail outliers (max spikes) and ensure the probe stays within Tier B on high-end HW.

### M5: Text pipeline stabilization (editor-ready)

- [ ] Document stable cache keys for measure/shaping (wrap width, font stack, style).
- [ ] Reduce redundant text measurements under intrinsic probes (layout engine + `TextWrap::None` paths).
- [x] Add a fast path for “min-content probes” (e.g. `wrap=Word` + `max_width=0`) to avoid O(n²) text wrapping.
  - Implemented by `perf(fret-render): fast-path wrapped text measure` (see perf log entry for commit `9440648a`).
- [x] Reduce repeated shaping work when taffy calls `measure()` under multiple intrinsic modes (min/max/definite).
  - Implemented by caching single-line shaping + cluster-based wrap stats (see `ui-perf-zed-smoothness-v1-log.md`).
- [x] Cut code editor syntax paint cost in the “autoscroll torture” probe (p95 paint drops from ~23ms → ~5ms).
  - Implemented by `perf(fret-code-editor): cache syntax rich rows` (commit `81159325`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entries for commit `bd709f88` (baseline) and `81159325`.
- [x] Eliminate allocation churn in editor syntax paint by avoiding per-row `Theme` clones.
  - Implemented by `perf(code-editor): avoid per-row Theme clone in syntax paint` (commit `0d8ad27ac`).
  - Evidence + numbers: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for 2026-02-06 (commit `0d8ad27ac`).
- [x] Add diagnostics hooks to identify text cache misses that correlate with perf hitches.
  - `paint_widget_hotspots` now include `ElementInstance` kind attribution (commit `c80525b9`).
  - `paint_widget_hotspots` now include element debug paths for faster attribution (commit `414974a44`).
  - Paint-phase text prepare counters + reason counts:
    - `paint_text_prepare_time_us`, `paint_text_prepare_calls` (commit `07d2ccf2`)
    - `paint_text_prepare_reason_*` (commit `80a46d49`)
  - Per-frame top-N text prepare hotspots with node/element ids + constraints + reason mask:
    - `paint_text_prepare_hotspots` (commit `77979100`)
- [x] Add a steady-state menubar hover probe to confirm “text prepares happen only on first appearance”.
  - Script: `tools/diag-scripts/ui-gallery-menubar-open-hover-sweep-steady.json` (commit `0a8191eb`)
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry for `ui-gallery-menubar-open-hover-sweep-steady`.
- [ ] Ensure atlas eviction and re-upload events are observable in perf snapshots.

Perf acceptance:

- [ ] Editor-class pages remain within Tier A budgets; Tier B progress is tracked.

### M6: Perf gates in CI (optional, but recommended)

- [ ] Define a reduced suite for CI (fast, stable, platform-agnostic as much as possible).
- [ ] Decide baseline storage approach (per platform, per hardware class).
- [ ] Add a “perf regression triage” template: which bundle artifacts to attach, how to compare.

## Cross-cutting hygiene

- [ ] When a refactor changes a hard-to-change behavior, capture it as an ADR and update
  `docs/adr/IMPLEMENTATION_ALIGNMENT.md` if relevant.
- [ ] Prefer tooling-driven evidence: `bundle.json`, `check.*.json`, and reproducible scripts.
- [ ] Keep `fret-ui` policy-light (mechanisms only; policy stays in ecosystem; see ADR 0066).
- [ ] Track GPUI performance gaps explicitly and close them with measurable gates:
  - `docs/workstreams/standalone/ui-perf-gpui-gap-v1.md`
- [x] Stabilize `ui-gallery-steady` perf baseline gates against microsecond jitter.
  - Adjustment: add slack + quantum rounding for pointer-move thresholds in perf baseline generation.
  - Refresh baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v12.json`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 12:36.
- [x] Refresh steady baseline after perf-threshold schema update (run-max hit-test replay metrics).
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v14.json`
  - Validation: `target/fret-diag-codex-perf-v14-validate2/check.perf_thresholds.json` (failures=0).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 20:12.
- [x] Add anti-noise threshold seeding metadata for steady baselines.
  - Baseline row now records `measured_p90`, `measured_p95`, `threshold_seed`, `threshold_seed_source`.
  - Baseline header records `threshold_seed_policy` (default seed + per-script/metric rules).
  - Script-specific policy: resize steady uses p95 seed for `top_total/layout/solve`; other metrics stay max-seeded.
  - Percentile seeds use linear interpolation so repeat=7 no longer degenerates to max-only seeding.
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v15.json`
  - Validation: `target/fret-diag-codex-perf-v15-validate-seed/check.perf_thresholds.json` (failures=0).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entries 2026-02-06 21:05 and 2026-02-06 21:35.
- [x] Make baseline seed policy configurable from CLI.
  - New flag: `--perf-baseline-seed <scope@metric=max|p90|p95>` (repeatable; scope supports suite names and `this-suite`).
  - Example: `--perf-baseline-seed ui-gallery-steady@top_total_time_us=p90`.
  - Template doc: `docs/workstreams/perf-baselines/seed-policy-template.md`.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 21:35.
- [x] Add versioned JSON presets for baseline seed policies.
  - New flag: `--perf-baseline-seed-preset <path>` (repeatable; merge order follows CLI argument order).
  - Merge precedence: built-in defaults -> preset rules -> explicit `--perf-baseline-seed` overrides.
  - Added preset example: `docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json`.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 22:50.
- [x] Run v16 preset trial and evaluate gate stability.
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v16.json`.
  - Validation sample: `target/fret-diag-codex-perf-v16-validate{,2,3}/check.perf_thresholds.json` (all `failures=1`).
  - Control: `target/fret-diag-codex-perf-v15-validate-recheck/check.perf_thresholds.json` (`failures=0`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 23:20.
- [x] Publish `ui-gallery-steady.v2` preset to remove known false-fail hotspot.
  - Updated: `docs/workstreams/perf-baselines/policies/ui-gallery-steady.v2.json`.
  - Change: `tools/diag-scripts/ui-gallery-overlay-torture-steady.json` now uses `p95` override.
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v17.json`.
  - Validation sample: `target/fret-diag-codex-perf-v17-validate{1,2,3}/check.perf_thresholds.json` (all `failures=0`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 23:55.
- [x] Harden baseline generation workflow against resize outliers (tooling).
  - Added: `tools/perf/diag_perf_baseline_select.sh` (candidate selection + validation sampling + summary JSON).
  - Rule: choose candidate by failures -> resize p90 -> threshold-sum.
  - Template doc updated: `docs/workstreams/perf-baselines/seed-policy-template.md` (`Candidate selection workflow`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-07 00:35.
- [x] Promote payload-aware code-editor autoscroll Windows baseline.
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v4.json`
  - Seed policy: `docs/workstreams/perf-baselines/policies/ui-gallery-code-editor-torture-autoscroll-steady.v2.json`
  - Selector summary: `target/fret-diag-baseline-select-ui-gallery-code-editor-torture-autoscroll-steady-windows-rtx4090-v4c/selection-summary.json`
  - Result: candidate-1 selected with `fail_total=0`; candidate-2 also validated `3/3`.
  - Contract: `threshold_surface=ui-renderer-payload`; `top_total_time_us` uses `p90` + `quantum_us=16`, `top_layout_time_us`
    uses `p90` + `min_slack_us=144` + `quantum_us=8`.
  - Evidence: perf log entry `2026-05-11`.
- [x] Promote payload-aware code-editor autoscroll typical Windows baseline.
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v2.json`
  - Seed policy: `docs/workstreams/perf-baselines/policies/ui-gallery-code-editor-torture-autoscroll-typical.v1.json`
  - Selector summary: `target/fret-diag-baseline-select-ui-gallery-code-editor-torture-autoscroll-typical-windows-rtx4090-v2/selection-summary.json`
  - Result: candidate-1 selected with `fail_total=0`; candidate-2 also validated `3/3`.
  - Contract: `threshold_surface=ui-renderer-payload`; measured p50/p95/max top total=`2563/3603/3603us`;
    hard frame p95 thresholds total/layout/solve=`3360/368/0us`; payload thresholds instance/text_ops=`262416/406`.
  - Evidence: perf log entry `2026-05-11`.
- [x] Promote complex code-editor wheel Windows baseline after dirty aggregation repair.
  - Script:
    `tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.json`.
  - Baseline:
    `docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.windows-rtx4090.v1.json`.
  - Seed policy:
    `docs/workstreams/perf-baselines/policies/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.v1.json`.
  - Selector summary:
    `target/fret-diag-baseline-select-ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady-windows-rtx4090-v1-policy3/selection-summary.json`.
  - Result: candidate-1 and candidate-2 both validated `3/3` with `fail_total=0`; candidate-2 selected on lower
    suite p90.
  - Contract: `threshold_surface=ui-renderer-payload`, `ui_threshold_mode=top_and_frame_p95`; measured p50/p90/max
    top total=`2424/5027/5027us`, frame-p95 total=`2250/2784/2784us`; thresholds top(total/layout/solve)=
    `6033/848/0us`, frame-p95(total/layout/solve)=`3808/592/0us`, payload instance/text_ops=`258663/406`.
  - Evidence: perf log entry `2026-05-11`.
- [x] Make perf baseline UI threshold mode explicit.
  - Seed policy now chooses `top`, `frame_p95`, or `top_and_frame_p95`.
  - The tooling no longer infers typical-frame contracts from suite names; use `frame_p95` for typical contracts and
    `top_and_frame_p95` when a probe intentionally protects tail and typical smoothness together.
  - Evidence: perf log entry `2026-05-11` (`explicit UI threshold mode for perf baselines`).
- [x] Quantize “big-frame” perf baseline thresholds to reduce 1–2us gate flakiness.
  - Change: use `apply_perf_baseline_headroom_with_slack_and_quantum(..., quantum_us=4)` for `top_total/layout/solve`.
  - Commit: `c7ea64bb5`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-07 09:02.
- [x] Promote selected v18 baseline as canonical after candidate-selection run.
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v18.json`.
  - Selection summary: `target/fret-diag-codex-perf-v18-select2/selection-summary.json`.
  - Stability: both candidates validated `3/3` with `failures=0`; winner copied to v18 baseline.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-07 00:35.
- [x] Refresh the canonical steady baseline after diagnostics/perf instrumentation changes.
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v22.json`.
  - Selection summary: `target/fret-diag-baseline-select-ui-gallery-steady-v22/selection-summary.json`.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-07 10:10.
  - Follow-up: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v23.json` bumps micro headroom for
    `ui-gallery-menubar-keyboard-nav-steady` `solve/layout` to avoid 1–30us flake (see 2026-02-08 log).

- [x] Stabilize resize perf scripts and refresh the P0 resize probes baseline + default gate pointer.
  - Scripts:
    - `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json` (insert per-resize waits; settle before capture)
    - `tools/diag-scripts/ui-gallery-window-resize-drag-jitter-steady.json` (insert waits; shrink jitter span)
  - Baseline: `docs/workstreams/perf-baselines/ui-resize-probes.macos-m4.v3.json`
  - Gate default: `tools/perf/diag_resize_probes_gate.sh`
  - Commit: `cad3fef6a`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-07 09:28.

- [x] Avoid redundant scale-factor events during interactive resize.
  - Change: only deliver `Event::WindowScaleFactorChanged` when the scale factor actually changes.
  - Commit: `66b610487`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-07 09:15.


- [x] Coalesce window resizes to once per frame in the desktop runner.
  - Change: apply `WindowEvent::SurfaceResized` at `RedrawRequested` (keep latest pending size).
  - Commit: `beb2fa315`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 13:20.
- [x] Make “deferred unbounded scroll probes on resize” the default behavior (keep an opt-out).
  - Default: enabled (set `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_RESIZE=0` to disable).
  - Invalidation-only gate (separate): `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1`.
  - Debounce: `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES=2` (default).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-07 15:56.
    - [x] Add a correctness probe to ensure resize stress does not clamp scroll offsets incorrectly.
      - Script: `tools/diag-scripts/ui-gallery-window-resize-scroll-offset-stable.json`
      - Gate: `--check-scroll-offset-stable ui-gallery-content-viewport`
      - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 14:26.
    - If acceptable, flip the default for resize-only (keep invalidation deferral opt-in).
- [x] Reuse known post-layout scroll extents for definite vertical scroll surfaces.
  - Scope: final layout only, vertical `probe_unbounded` Scroll nodes with a definite viewport and a previously
    observed non-zero content extent.
  - Rationale: once post-layout overflow observation is the authoritative extent source, steady invalidation frames
    should not repeat a deep child `measure()` walk just to seed the scroll range.
  - Gate: `cargo nextest run -p fret-ui scroll_`.
  - Evidence: perf log entry `2026-05-07 19:58`; the Material3 tabs representative probe drops from
    `p95 total/layout=6565/4440us` to `3716/1663us`, and `ui-gallery-content-viewport` steady
    `measure_children_us` falls to `0`.
- [x] Split and fix horizontal Scroll extent inflation in Material3 tabs.
  - Evidence from the vertical post-layout reuse profiling pass: `ui-gallery-material3-tabs-scrollable` could grow
    `content_w` from `809` to `5663` and then `39648` while using the horizontal unbounded-probe path.
  - Fix: keep scrollable Material3 primary tab labels intrinsic-width, and prevent deferred unbounded-probe frames from
    growing scroll-axis extents from stretched post-layout geometry.
  - Gate: `cargo nextest run -p fret-ui scroll_` plus focused Material3 tabs layout tests.
  - Evidence: perf log entry `2026-05-07 21:36`; the X scroll node stays at `content_w=809.0` through the resized
    deferred frame and no longer emits X-axis `scroll extent grew` lines.
- [x] Make `FRET_UI_GALLERY_VIEW_CACHE=1` drive the runtime model, not only initial `UiTree` state.
  - Discovery: `render_flow::begin_frame` overwrote `UiTree.view_cache_enabled` from a model that still defaulted from
    stale `FRET_UI_GALLERY_VIEW_CACHE_ENABLE_INNER_CONTROL`, so prior perf runs claiming view-cache mode could actually
    run with cache roots reporting `reuse_reason="view_cache_disabled"`.
  - Fix: use one gallery `ViewCacheBootConfig` to initialize both the model and `UiTree`; remove the stale env branch.
  - Gate: focused `fret-ui-gallery` boot-config tests plus release gallery build.
  - Evidence: perf log entry `2026-05-07 22:25`; Material3 tabs steady has `paint.cache_misses=0` and no
    `view_cache_disabled` cache roots under `FRET_UI_GALLERY_VIEW_CACHE=1`.
- [x] Review gallery shell/content view-cache boundary semantics after env fix.
  - Evidence: the env-fixed Material3 tabs steady bundle still shows the content cache root can report
    `reuse_reason="needs_rerender"` during most non-reuse frames, moving the hotspot from paint churn to avoidable
    view-rerender pressure.
  - Rejected experiment: blindly making the content pane `contained_layout` worsened the Material3 tabs steady sample
    and did not remove the non-reuse frames.
  - Decision: make whole-page content caching an explicit gallery page policy. Mostly static pages remain cacheable;
    pages whose primary examples own local interaction state can opt out without coupling component recipes to shell
    cache topology.
  - Implementation: `PAGE_MATERIAL3_TABS` opts out via `PageContentCachePolicy`; the Magic Patterns torture opt-out now
    uses the same metadata hook instead of a shell-only special case.
  - Gate: focused `fret-ui-gallery` policy test plus release gallery build.
  - Evidence: perf log entry `2026-05-08 08:33`; Material3 tabs steady p95 total/layout moves from `3210/2502us` to
    `2696/1956us`, `view_cache_roots_needs_rerender=0`, and tab-switch layout nodes stay at `43`.
- [x] Fix cache-root diagnostics so view-rerender misses are not hidden as generic non-reuse.
  - Change: record `UiDebugCacheRootReuseReason` before clearing `view_cache_needs_rerender` in `mount_element`.
  - Gate: `cargo nextest run -p fret-ui view_cache` (`54/54` passed).
  - Evidence: perf log entry `2026-05-07 23:30`; Material3 tabs content root now reports
    `reuse_reason="needs_rerender"` while paint cache misses remain `0`.
- [x] Keep Material3 indication animation frames paint-only under view-cache reuse.
  - Change: add `CanvasPainter::request_animation_frame_paint_only()` and move Material3 pressable indication progression
    into retained paint-time state; preserve normal RAF for `extra_want_frames` callers that still depend on render-time
    animation state.
  - Gate: `cargo nextest run -p fret-ui view_cache`, focused Material3 indication tests, Material3 tabs tests, and
    `cargo build -p fret-ui-gallery --release --features gallery-full`.
  - Evidence: perf log entry `2026-05-07 23:59`; Material3 tabs steady p95 total/layout/paint moves from
    `5946/4405/1453us` to `3210/2502/620us`, with indication-only RAF walks recorded as
    `source=other detail=animation_frame_request` and no cache-root `needs_rerender`.
- [x] Attribute a11y-active semantics refresh cost under diagnostics.
  - Evidence from the Material3 tabs page-cache pass: `FRET_A11Y_DISABLE=1` removes roughly `0.8-1.1ms` of
    `layout_semantics_refresh_time_us` from the same representative script.
  - Keep this separate from view-cache/page-cache policy work; the decision surface is AccessKit/diagnostics refresh
    cadence and incremental semantics data, not gallery content topology.
  - Implementation: gate accessibility/diagnostics semantics snapshot requests on a `UiTree` semantics-dirty bit.
    Structural, layer, focus, layout, hit-test, model/global, and notify invalidations rearm the snapshot; paint-only
    animation, hover, focus-visible policy, and input-modality policy invalidations keep the previous snapshot.
  - Gate: `cargo nextest run -p fret-ui semantics`, `cargo check -p fret-bootstrap`, and
    `cargo build -p fret-ui-gallery --release --features gallery-full`.
  - Evidence: perf log entry `2026-05-08 09:42`; Material3 tabs steady under diagnostics semantics has
    `p50/p95/max total=1832/1873/1873us`, animation-frame-only frames no longer refresh semantics, and only real tab
    selection changes rebuild the semantics snapshot.
  - Follow-up: if real semantic-change frames become the next bottleneck, design incremental semantics/diffing instead
    of broadening the dirty filter.
- [x] Audit layout side effects for the first engine-solved geometry propagation slice.
  - Scope: `Scroll`, `VirtualList`, text/text input widgets, canvas/viewport surfaces, layout-query regions,
    transforms, and anchored/overlay-related nodes.
  - Rationale: resize profiling shows the remaining `ScrollArea` hotspot is broad `layout_in` recursion after Taffy has
    solved child rects, not unbounded child measurement. Skipping `widget.layout` globally would be incorrect unless the
    affected subtree has no layout-time side effects.
  - Evidence: perf log entry `2026-05-08 15:40`; clean or near-clean resize frames can still visit roughly `962-1044`
    child-layout nodes.
  - Result: the landed slice only allows mechanism-only nodes whose layout can be represented as solved geometry
    propagation (`Container`, `Pressable`, `Semantics`, `ViewCache`, `FocusScope`, `ForegroundScope`, `Opacity`,
    `Stack`, `Grid`, and non-auto-margin `Flex`/`SemanticFlex`/`RovingFlex`). Leaf text stays eligible only when its
    size is unchanged.
  - Explicitly rejected in this slice: `Scroll`, `VirtualList`, text input/area, transforms, anchored overlays,
    layout-query regions, retained/custom widgets, absolute-positioned children, suppressed dirty-child subtrees, and
    flex auto margins.
  - Semantic guard: both translation-only and size-delta fast paths refresh `current_bounds_for_element`, so layout
    queries and overlay/focus geometry do not read stale element bounds.
  - Evidence: perf log entry `2026-05-09 16:38`.
- [x] Land a guarded engine-solved subtree apply path for proven-safe layout nodes.
  - Implementation: clean, final-pass, non-dirty engine-solved nodes can propagate cached child rects without rerunning
    structural `widget.layout`, falling back to normal layout for unsupported or side-effectful nodes.
  - Gate:
    `cargo nextest run -p fret-ui clean_engine_solved_size_delta_propagates_geometry_without_relayouting_structure solve_barrier_flow_root_reuses_solved_root_even_after_other_solves solve_barrier_flow_root_if_needed_skips_translation_only_bounds_changes nested_flow_is_solved_once_per_island`.
  - Release/perf gates: `cargo build -p fretboard --release`, `cargo build -p fret-ui-gallery --release --features
    gallery-full`, plus prewarmed repeat=3 `ui-gallery-window-resize-stress-steady.json` and
    `ui-gallery-window-resize-drag-jitter-steady.json`.
  - Evidence:
    - Stress final repeat=3:
      `target/fret-diag/codex-clean-engine-propagation-stress-final-r3/regression.summary.json`
      (`total/layout/solve p95=9089/4746/2352us`).
    - Drag-jitter final repeat=3:
      `target/fret-diag/codex-clean-engine-propagation-drag-jitter-final-r3/regression.summary.json`
      (`total/layout/solve p95=6495/3947/2328us`).
- [x] Reject the earlier broad guarded engine-solved subtree apply experiment on the current resize-stress sample.
  - Evidence: perf log entry `2026-05-08 16:45`; `ui-gallery-window-resize-stress` p95 total/layout/paint worsened
    from `8234/4505/3494us` to `8659/4692/3629us`.
  - Decision: do not promote the broad fast path; keep the next implementation pass focused on the narrower
    dirty-frontier / scroll post-layout branch.
- [x] Add Scroll child-root bounds delta profiling before attempting the narrower dirty-frontier path.
  - Fields: `layout_child_max_bounds_changed`, `layout_child_max_bounds_size_changed`,
    `layout_child_max_input_matches_before`, `layout_child_max_input_size_matches_before`,
    `layout_child_max_bounds_before`, `layout_child_max_bounds_after`, and `layout_child_max_input_bounds`.
  - Evidence: perf log entry `2026-05-08 23:16`; rebuilt release smoke bundle
    `target/fret-diag/codex-scroll-bounds-delta-profile-r2/1778253370943/bundle.schema2.json`.
  - Result: the first heavy Scroll profile samples are initial/fresh mount frames where the child root changes from
    zero bounds to content bounds. The same profiling payload is now exported into diagnostics bundles as
    `debug.scroll_nodes[].layout_profile`, surfaced in `fretboard diag stats` as `scroll_layout_profiles`, and
    mirrored into triage JSON as `layout.scroll_profile_present`; next attribution should capture stable resize
    frames and separate real geometry deltas from clean subtree state sync.
- [x] Capture stable cached-flow resize-frame scroll profiles and classify whether clean-child-root apply skipping is
  justified.
  - Target: use `layout_child_max_bounds_changed=false` / `layout_child_max_input_matches_before=true` frames from the
    new stats surface to separate genuine geometry changes from pure state sync.
  - Gate: `target\release\fretboard.exe diag stats <bundle.json> --sort time --top 5 --json`
  - Evidence anchor: `target/fret-diag/codex-scroll-layout-profile-stable-fullsnap/1778292518840/bundle.schema2.json`.
  - Result: the full-snapshot low-threshold probe captured 83 scroll profiles across 85 retained snapshots. 78 profiles
    are interactive-resize real bounds deltas (`bounds_changed=true`, `input_matches_before=false`); only 3 profiles
    are clean state-sync candidates, and those are not the live cached-flow resize frames.
  - Decision: do not implement a clean-child-root apply skip from this sample. Keep optimizing the existing resize
    layout path, especially real bounds-delta scroll child relayout and layout-root scheduling.
- [x] Attribute the real bounds-delta scroll resize path before changing layout semantics.
  - Target: split the `layout_child_max_us` cost between unavoidable child bounds application, layout-engine solve
    input churn, and any redundant repeated root scheduling.
  - Start from: `target/fret-diag/codex-scroll-layout-profile-stable-fullsnap/1778292518840/bundle.schema2.json`,
    especially the interactive-resize profiles where `direct_children_layout_invalidated=false`,
    `descendant_subtree_layout_dirty=false`, but `layout_child_max_bounds_changed=true`.
  - Evidence update: perf log entry `2026-05-09 10:55`; new bundle
    `target/fret-diag/codex-scroll-layout-pass-split-smoke/1778294912347/bundle.schema2.json`.
  - Result: live resize cost is first-pass real bounds application / barrier solve, not corrected-content relayout and
    not repeated root scheduling. In the smoke bundle, `ui-gallery-content-viewport` reports `sum_child=83113us`,
    `sum_first=83113us`, `sum_corrected=0us`; `ui-gallery-view-cache-root` reports `sum_barrier=43615us`.
- [x] Add kind-level attribution for scroll child layout before choosing a component-specific optimization.
  - Target: separate text measurement cost from structural layout propagation inside the real bounds-delta child
    layout path.
  - Fields: `layout_child_first_pass_kind_profiles`, `layout_child_corrected_content_kind_profiles`, and
    `layout_child_kind_profiles`.
  - Evidence update: perf log entry `2026-05-09 12:24`; smoke bundle
    `target/fret-diag/codex-scroll-kind-profile-smoke/1778300270796/bundle.schema2.json`.
  - Result: in filtered live-resize real-bounds-delta frames, `ui-gallery-content-viewport` has 27 profiles with max
    `total_us=4377`, max first-pass child layout `3699us`, max traversal `1042` nodes, and no child invalidation or
    subtree dirty flag. The largest filtered content profile records kind self costs of `Scroll=1805us`,
    `Text=645us`, `Flex=458us`, `Container=201us`, and `Pressable=73us`; inclusive totals still stack through
    container/flex wrappers. Do not spend the next pass on a text-specific fast path without fresh evidence.
- [x] Add internal phase attribution for Scroll layout before changing the layout data model.
  - Target: split `Scroll` self/total time into mechanism phases so the next optimization can choose between
    measure/probe policy, handle telemetry, overflow observation, child-layout application, and barrier solve.
  - Fields: `phase_profiles[]` under `debug.scroll_nodes[].layout_profile`, surfaced through diagnostics bundle
    serialization, `fretboard diag stats` text/JSON output, and triage JSON.
  - Evidence update: perf log entry `2026-05-09 13:31`; smoke bundle
    `target/fret-diag/codex-scroll-phase-profile-smoke/1778304701572/bundle.schema2.json`.
  - Result: filtered live-resize real-bounds-delta frames show `measure max=0us`. Content viewport phase cost is
    dominated by `layout_children_first_pass` (`p95=3640us`) with secondary `solve_barrier` (`p95=672us`).
    View-cache root phase cost is dominated by `solve_barrier` (`p95=1674us`) with secondary
    `layout_children_first_pass` (`p95=1296us`). Probe/cache/overflow/handle phases are near-zero.
- [x] Investigate and reduce content scroll real bounds application cost.
  - Target: clean live resize frames where `ui-gallery-content-viewport` visits roughly `1042` child nodes with
    `bounds_changed=true`, `input_matches_before=false`, `layout_child_max_invalidated=false`, and
    `layout_child_max_subtree_dirty=false`.
  - Question: can a clean subtree whose size changed receive final bounds through a narrower geometry-propagation path
    without rerunning layout-time widget side effects?
  - Latest evidence: kind-level attribution points at structural layout application and inclusive propagation
    (`Scroll` / `Flex` / `Container` / `Pressable`) rather than a component-local `Text` hotspot; phase attribution
    shows `layout_children_first_pass` dominates (`p95=3640us`) and measure/probe/overflow phases are negligible.
  - Latest child-rect evidence: `layout_engine_child_rect_queries=1196` costs only `70us` in the latest worst frame,
    so the content lane should stay focused on clean bounds-size application, not child-rect lookup overhead.
  - Result: safe-subset engine-solved geometry propagation removes most structural relayout churn while keeping
    side-effectful widgets on the full `layout_in` path. Final normalized stress evidence reports
    `total/layout/solve p95=9089/4746/2352us`, and drag-jitter reports `6495/3947/2328us`.
  - Guardrails: preserve scroll extents, hit testing, element bounds cache, semantics bounds, focus/overlay geometry,
    text input layout state, virtual-list visible ranges, and layout query semantics.
  - Gate: focused layout-engine regression tests plus prewarmed repeat=3
    `ui-gallery-window-resize-stress-steady.json` and `ui-gallery-window-resize-drag-jitter-steady.json` perf samples.
- [ ] Investigate clean view-cache scroll barrier solve cost.
  - Target: `ui-gallery-view-cache-root` live resize profiles where `solve_barrier_us` dominates while child subtree
    dirty flags are false.
  - Question: can viewport-root override / barrier solve input churn be coalesced or made cheaper for clean real bounds
    deltas without stale layout engine rects?
  - Latest evidence: phase attribution shows `solve_barrier` dominates (`p95=1674us`, max `1712us`) while
    `measure max=0us` and probe/cache/overflow phases are negligible.
  - Latest solve-profile evidence: `diag stats` now emits `top_layout_engine_solves[].solve_profile`; in the latest
    smoke the view-cache root reports `reason=new_frame_same_key` with `available_w=852`, `available_h=8636`, and
    `subtree_nodes=962`, while another sample in the same run reports `reason=new_frame_key_changed` when the root
    width changes. The remaining question is not whether the solve is happening, but whether the root solve can be
    made cheaper or better coalesced without stale rects.
  - Latest child-rect evidence: `layout.perf.summary.v1.json` and `diag stats` now expose
    `layout_engine_child_rect_queries`, `layout_engine_child_rect_time_us`, and
    `layout_engine_widget_fallback_solves`; the latest worst frame reports `1196` queries for `70us` and `0`
    widget-local fallback solves. The current hotspot is not child-rect lookup/replay.
  - Latest root-solve evidence: clean live-resize view-cache frames still show
    `solve_barrier_us=1616..1795us` with view-cache root solve profile
    `reason=new_frame_key_changed`, `subtree_nodes=962`, and `batch_roots=1`.
  - Latest root-solve delta evidence: solve profiles now include previous available size / scale factor and deltas.
    The resize-stress smoke reports the view-cache root solve at `available_w=930`, `previous_available_w=692`,
    `available_w_delta=238`, `available_h_delta=0`, `scale_factor_delta=0`, and `previous_frame_delta=3`.
    Decision: this is a real logical width delta, not float jitter; do not pursue root-solve-key quantization as the
    next optimization. Focus next on reducing the 962-node root's width sensitivity, splitting the solve boundary, or
    optimizing Taffy solve cost with evidence.
  - Latest harness-topology evidence: perf log entry `2026-05-09 17:52` replaces the view-cache torture page's
    artificial 240-row non-virtualized button list with a retained virtual list. The page-local view-cache reuse root
    element count drops from `1104` to `137`; top layout nodes drop from `278` to `34`; and the normalized resize
    smoke drops from `total/layout/solve/paint=8810/4774/2229/3711us` to `3971/1788/784/1988us`.
    Repeat=3 confirmation reports resize-stress p95 `4252/1719/717/2352us` and drag-jitter p95
    `2066/1310/754/643us`, both with `view_cache_roots_reused=2/2`.
    Decision: keep this as a gallery/component-layer correction, not a core-layout shortcut. Remaining core work should
    target legitimate wide or width-sensitive roots after demo pressure sources are removed.
  - Reference direction: compare with GPUI/Zed's per-frame `request_layout` / `compute_layout` / `layout_bounds`
    model; keep Fret's retained state semantics explicit rather than adding broad clean-subtree skips.
- [x] Surface code-editor row-scene paint attribution in `diag stats`.
  - Target: make the existing `app_snapshot.code_editor.torture.paint_perf` counters queryable from normal
    `fretboard diag stats` output so editor paint work can be split between row-scene replay/store, content resolve,
    text draw, rich materialization, syntax work, and renderer encode/upload before a display-list rewrite is attempted.
  - Implementation: `diag stats --json` now emits top-level `code_editor_paint_perf` p50/p95/max/sum summaries and
    per-top-frame `top[].code_editor_paint_perf`; human output prints the same row-scene and text/content breakdown.
    The stats reader now prefers `ns_*` paint counters when present, because summing per-row `as_micros()` values was
    under-reporting editor paint by roughly 15-25% on the current complex wheel bundle.
  - Gate: `cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot`.
  - Evidence: `cargo run -p fretboard -- diag stats target/fret-diag/perf-complex-editor-wheel-tail-syntax-line-prefetch-v1/1778501381582/bundle.json --json --top 1 --sort time`
    now reports `code_editor_paint_perf.frames=34`, top frame `rows_scene_replayed=204`, `rows_scene_stored=1`,
    `us_row_content_resolve=544`, `us_row_scene_fast_path=373`, `us_row_scene_fast_probe=63`,
    `us_row_scene_replay_ops=70`, `us_row_scene_replay_touch=78`, `us_row_scene_capture_ops=0`,
    and `us_text_draw=0`; summary p95 is `us_total=886`, `us_row_content_resolve=636`,
    `us_row_scene_fast_path=347`, `us_row_text=88`, and `us_text_draw=147`.
  - Decision: keep the next optimization evidence-led. The post-fix wheel bundle points at the row-scene fast replay
    path plus Canvas paint-widget and renderer payload, not at row-scene capture/store or syntax materialization. Do
    not start a broad row display-list rewrite until a near-threshold or failing stressor shows replay/capture/store
    as the measured limiter.
- [x] Fix `Scene::replay` text-blob side-index semantics before deeper row-scene rewrites.
  - Discovery: `SceneRecording::push` recorded `SceneOp::Text` ids in `Scene::text_blob_ids()`, but
    `SceneRecording::replay_ops` only copied ops/fingerprint. Replayed row text therefore skipped the renderer text
    prepare side index, even though the text ops remained in the op stream.
  - Implementation: replay now maintains `text_blob_ids`; hot paths can call
    `replay_ops_with_text_blob_ids` / translated / transformed variants with a precomputed text index. Debug builds
    assert that the provided ids match the replayed ops. Code editor row-scene replay uses
    `CanvasHostedResources::text_blob_ids()` to avoid rescanning cached row ops.
  - Gate: `cargo nextest run -p fret-core replay_ops_tracks_text_blob_ids_in_op_order replay_ops_translated_with_text_blob_ids_tracks_precomputed_index`;
    `cargo check -p fret-ui`; `cargo check -p fret-code-editor --features syntax-rust`;
    `cargo nextest run -p fret-ui --lib hosted_resources_from_scene_ops_collects_resource_ids`.
  - Evidence: perf log entry `2026-05-11 23:59`. Complex wheel repeat=3 with paint detail reports worst total
    `3408us`, p95 `us_row_scene_replay_touch=65`, `us_row_scene_replay_ops=77`, and renderer text prepare p95/max
    `1287/1302us` with atlas upload/eviction still `0`. The formal baseline repeat=3 without paint detail passes the
    current Windows v1 contract with worst top total `2859us` and payload text ops / instance bytes `254/192368`.
  - Decision: keep hosted-resource touch for Canvas resource lifetime; treat renderer text prepare / glyph pinning as
    the next evidence target rather than row-scene capture/store or broad display-list replacement.
- [x] Precompute per-shape glyph pin keys for renderer text prepare.
  - Discovery: after replayed text correctly entered `Scene::text_blob_ids()`, renderer text prepare spent real CPU
    walking every `TextShape::glyphs()` entry and inserting each glyph key into per-frame `HashSet`s. The unique pin-key
    set is stable for the prepared shape.
  - Implementation: `TextShape` stores `GlyphPinKeys` built once at shape creation; renderer atlas pinning merges those
    pre-deduplicated key sets. Shape heap-byte diagnostics now include the pin-key arrays.
  - Gate: `cargo fmt -p fret-render-wgpu --check`; `cargo check -p fret-render-wgpu`;
    `cargo nextest run -p fret-render-wgpu --lib glyph_pin_keys_deduplicate_by_bucket`.
    Package-wide `nextest` without `--lib` hit Windows pagefile/mmap pressure while compiling integration tests
    (`os error 1455`), so the focused library gate is the reliable test evidence for this slice.
  - Evidence: perf log entry `2026-05-11 23:59`. Complex wheel repeat=3 paint detail reports renderer text p95/max
    `660/722us` versus the prior replay-index slice's `1287/1302us`; perf row `top_renderer_prepare_text_us`
    p50/p95/max is `441/541/541us`. Formal baseline repeat=3 passes with worst top total `2206us`,
    frame p95 total `2206us`, and payload text ops / instance bytes `254/192368`.
  - Decision: keep the current v1 baseline unchanged; this reduces headroom pressure without changing thresholds.
- [x] Suppress display-none `InteractivityGate` child layout dirty from ancestor cached-flow decisions.
  - Discovery: resize request-build roots that were clean except for descendant dirty samples traced to
    `Opacity` / `Scrollbar` `initial_mount` nodes under absent `ScrollArea` chrome.
  - Fix: keep hidden children mounted and dirty, but exclude them from `subtree_layout_dirty_count` while the gate is
    `present=false`; restore the aggregate when `present=true`.
  - Gate: `cargo nextest run -p fret-ui interactivity_gate interactive_resize_flow_rebuild view_cache`.
  - Evidence: perf log entry `2026-05-08 22:05`; cached-flow resize frames now report `subtree_dirty=false`,
    `dirty_count=0`, while remaining heavy frames are classified as `interactive_resize_full_rebuild`.
- [x] Keep post-resize authoritative rebuild out of live resize frames by widening the quiet-window default.
  - Discovery: after hidden dirty suppression, remaining resize-stress tail came from `interactive_resize_full_rebuild`
    frames being inserted between scripted resize steps, not from a new dirty source.
  - Fix: default `FRET_UI_INTERACTIVE_RESIZE_STABLE_FRAMES` is now `4` instead of `2`; the deferred rebuild still runs
    after the configured quiet window.
  - Gate: `cargo nextest run -p fret-ui interactive_resize_flow_rebuild view_cache`.
  - Evidence: perf log entry `2026-05-08 22:48`; stress default smoke reports top
    total/layout/solve/paint `8756/4329/2238/4156us`, and drag-jitter default smoke reports
    `9049/6447/4283/2336us`.
- [ ] Consider a narrower dirty-frontier scroll relayout path if side-effect audit makes the broad fast path too risky.
  - Target: avoid amplifying a few descendant dirty nodes into a full direct child-root relayout when post-layout extents
    can remain authoritative.
  - Non-negotiable: deferred `scroll_to_item`, overflow observation, scrollbar thumb geometry, and hit-test bounds must
    stay correct.
- [x] Add an experiment gate for paint-cache replay under `HitTestOnly` invalidation.
  - Env: `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1`
  - Commit: `e50173f13`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 16:12.
- [x] Add diagnostics counters for the new gate path before deciding default behavior.
  - Export at least: “paint replay allowed by hit-test-only gate” and “hit-test-only replay attempts rejected by key mismatch”.
  - Implemented by `feat(diag): export hit-test-only paint-cache replay counters` (commit `f38f8c1d5`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 17:32.
  - [x] Add a focused script where `HitTestOnly` dominates and layout stays stable.
    - Added probe page + script: `hit_test_only_paint_cache_probe` + `tools/diag-scripts/ui-gallery-hit-test-only-paint-cache-probe-sweep.json`.
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 18:30.
  - [x] Export per-run counter maxima in `diag perf --json` for gate-path counters.
    - Implemented by `feat(diag): export per-run hit-test-only replay maxima in perf json` (commit `4c88f6696`); new fields `run_paint_cache_hit_test_only_replay_allowed_max` and `run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max`.
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 19:28.
    - `top_*` rows can stay `0` even when bundle-level max counters are non-zero.
  - [x] Wire run-max counters into perf baseline + threshold gates.
    - Implemented by `feat(diag): gate hit-test replay run-max in perf baseline` (commit `f4a6f422b`).
    - Adds CLI thresholds: `--min-run-paint-cache-hit-test-only-replay-allowed-max`, `--max-run-paint-cache-hit-test-only-replay-rejected-key-mismatch-max`.
    - Baseline export now includes both `measured_max` and `thresholds` for these counters.
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 19:56.
- [ ] Decide whether `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` should ever become default.
  - Current status: keep opt-in only; A/B evidence is mixed across repeated resize probes.
- [x] Gate pointer-move thresholds only when pointer-move frames are present for the script.
  - `diag perf` threshold rows now null pointer-move threshold values and sources when the script produced no
    pointer-move frames, even if CLI or baseline pointer-move limits are configured.
  - Baseline rows and `perf-baseline-from-bundles` now omit pointer-move thresholds for no-pointer-move scripts, and
    repeat-mode aggregation ignores stale pointer-move maxima from runs that did not report pointer-move frames.
  - Gate: `cargo nextest run -p fret-diag baseline_rows_omit_pointer_move_thresholds_when_frames_are_absent single_threshold_row_omits_pointer_move_thresholds_when_frames_are_absent repeat_threshold_row_omits_pointer_move_thresholds_when_frames_are_absent perf_threshold_scan_passes_when_under_limits perf_threshold_scan_reports_each_exceeded_metric --no-fail-fast`.
  - Evidence: perf log entry `2026-05-15` in
    `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`.
- [x] Keep diagnostics artifacts bounded for tool-launched perf runs (especially `target/fret-diag*` and
  `target/fret-diag-perf`).
  - `diag perf --launch` now writes a small-by-default tool-launched `diag.config.json`:
    `write_bundle_json=false`, `write_bundle_schema2=true`, `script_dump_max_snapshots=10`,
    `script_auto_dump=false`, and `pick_auto_dump=false`.
  - Manual attach / no-`--launch` runs remain caller-configured. Keep using explicit env/config such as
    `FRET_DIAG_SCRIPT_AUTO_DUMP=0` for those sessions, and clean old run directories periodically.
  - Gate: `cargo nextest run -p fret-diag tool_launch_config_defaults_are_small_by_default --no-fail-fast`.
  - Evidence: `crates/fret-diag/src/compare.rs` (`tool_launched_diag_config` and
    `validate_tool_launched_diag_config`); `docs/workstreams/diag-v2-hardening-and-switches-v1/README.md`.
