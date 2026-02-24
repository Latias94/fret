---
title: Diag Fearless Refactor v1 (TODO)
status: draft
date: 2026-02-24
scope: diagnostics, automation, bundle-schema, refactor
---

# Diag Fearless Refactor v1 (TODO)

This file tracks tasks for `docs/workstreams/diag-fearless-refactor-v1.md`.

## Runtime modularization (reduce `ui_diagnostics.rs` blast radius)

- [x] Extract filesystem triggers into a dedicated module:
  - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`
- [x] Extract “bundle writer” responsibilities (schema selection, semantics-mode application, JSON writing) into a module:
  - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump.rs`
- [x] Extract “script runner” responsibilities (step state machine + evidence capture) into a module:
  - [x] Extract pending-script start/bootstrap helper:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - [x] Extract single-active-script migration helper:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - [x] Extract keepalive/heartbeat helper (when current window has no active script):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - [x] Extract active-window heartbeat writer helper:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - [x] Extract progress writer + pending cross-window drag cancel helper:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - [x] Extract per-step evidence scoping + step-state reset helpers:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - [x] Extract window/cursor/mouse/insets effect-only steps:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
  - [x] Extract non-window “effect-only” steps (reset/wait/clipboard/open-inject):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
  - [x] Extract capture steps (bundle + screenshot export requests):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
  - [x] Extract keyboard/text injection steps (press/type/ime):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_input.rs`
  - [x] Extract selector-driven text input step (`type_text_into`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_input.rs`
  - [x] Extract menu selection step (`menu_select`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_menu.rs`
  - [x] Extract scroll-into-view step (`scroll_into_view`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_scroll.rs`
  - [x] Extract ensure-visible step (`ensure_visible`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_visibility.rs`
  - [x] Extract wheel step (`wheel`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer.rs`
  - [x] Extract click step (`click`) with window handoff + fail-fast behavior:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer.rs`
  - [x] Extract move pointer step (`move_pointer`) with fail-fast behavior:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer.rs`
- [x] Extract stable click steps (`click_stable`, `click_selectable_text_span_stable`):
  - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer.rs`
- [x] Extract assert step (`assert`):
  - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_assert.rs`
- [x] Extract pointer session steps (`pointer_down`, `pointer_move`, `pointer_up`):
  - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer_session.rs`
  - [x] Extract drag steps (`drag_pointer`, `drag_pointer_until`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_drag.rs`
  - [x] Extract drag-to step (`drag_to`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_drag.rs`
  - [x] Extract pointer sweep step (`move_pointer_sweep`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer_sweep.rs`
  - [x] Extract wait step (`wait_bounds_stable`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs`
  - [x] Extract menu path step (`menu_select_path`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_menu.rs`
  - [x] Extract slider step (`set_slider_value`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_slider.rs`
- [x] Extract “inspect/pick state machine” into modules (keep UI/UX policy out of `fret-ui`).
  - [x] Inspect-mode state + shortcuts + hover/focus details:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect.rs`
  - [x] Pick flow (run ids + result export + pending resolution):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/pick_flow.rs`
- [x] Keep DevTools WS wiring isolated (already split; ensure minimal coupling).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs` (`drive_devtools_requests_for_window`),
    `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`.

## Bundle size & AI loops (Plan 1, schema2-first)

- [x] Document recommended defaults for AI loops (env presets) and keep them consistent across tooling and runtime:
  - `docs/ui-diagnostics-and-scripted-tests.md` (AI presets)
- [x] Keep shareable zips bounded when `bundle.schema2.json` is available:
  - Preferred (bounded AI handoff): `diag run|suite|repro --pack --ai-only`.
  - Compat (offline viewer-friendly): `diag run|suite|repro --pack --include-all --pack-schema2-only` (or `--schema2-only`).
- [x] Ensure `bundle.index.json` / `test_ids.index.json` generation is part of the “pack/repro” happy path (where appropriate).
  - [x] Runtime writes canonical sidecars on dump (native filesystem):
    - [x] `bundle.index.json`
    - [x] `bundle.meta.json`
    - [x] `test_ids.index.json`
    - [x] Tail `test_id` bloom (`test_id_bloom_hex`) in `bundle.index.json` for fast `--test-id` queries.
    - [x] Script markers in `bundle.index.json` (`script.steps`) when `script.result.json` is present.
    - [x] Bounded `semantics_blooms` in `bundle.index.json` for `--test-id` filtering beyond the tail snapshots.
  - [x] Tooling consumes sidecars for fast queries (avoid reparsing `bundle.json` when possible):
    - [x] `diag meta` reads `bundle.meta.json` when present
    - [x] `diag query test-id` reads `test_ids.index.json` when present
    - [x] `diag query snapshots` / `diag slice` read `bundle.index.json` when present (selection + semantics presence)
  - [x] Packing includes canonical sidecars under `_root/` (even when the bundle dir is relocated):
    - [x] `diag pack --include-all` (and repro multi-pack with `--include-all`).
    - [x] Includes `frames.index.json` to support `triage --lite` without materializing large bundles.
- [x] Reduce CLI entrypoint churn by isolating zip packing logic:
  - `crates/fret-diag/src/pack_zip.rs`
- [x] Reduce CLI entrypoint churn by isolating evidence indexing:
  - `crates/fret-diag/src/evidence_index.rs`
- [x] Reduce CLI entrypoint churn by isolating perf-hint gating helpers:
  - `crates/fret-diag/src/perf_hint_gate.rs`
- [x] Add a short “AI-first” recipe to `docs/ui-diagnostics-and-scripted-tests.md` that links to:
  - `diag meta`, `diag index`, `diag query`, `diag slice`, `diag ai-packet`.
- [x] Finish the “bundle artifact” naming sweep so common failure messages and CLI hints do not assume `bundle.json`.
  - [x] Rename internal helpers away from `*_bundle_json_*` naming:
    - `crates/fret-diag/src/paths.rs` (`resolve_bundle_artifact_path`, `wait_for_bundle_artifact_*`)
  - [x] Ensure `diag doctor` distinguishes raw `bundle.json` from the resolved bundle artifact in `doctor.json`.
  - [x] Add bundle-artifact aliases to `repro.summary.json` (keep older `*_bundle_json` keys for compatibility).
  - [x] Add bundle-artifact aliases to `diag repeat` output (keep older `bundle_json` key for compatibility).
  - [x] Update CLI user-facing hints to prefer “bundle artifact” wording where supported:
    - `crates/fret-diag/src/diag_simple_dispatch.rs` (`diag trace`)
    - `crates/fret-diag/src/diag_perf_baseline.rs` (`perf-baseline-from-bundles`)
    - `crates/fret-diag/src/lib.rs` (`--diff` arg errors)
    - `crates/fret-diag/src/paths.rs` (integrity failure notes mention raw `bundle.json`)
  - [x] Sweep remaining narrow modules and tests that still talk about `bundle.json` when they mean “bundle artifact”:
    - [x] `crates/fret-diag/src/artifacts.rs` (docs/comments + test expectations)
    - [x] `crates/fret-diag/src/api.rs` (test fixture filenames `*.bundle.json`)
    - [x] `apps/fret-devtools-mcp/src/main.rs` (resolve/compare via bundle artifacts, prefer `bundle.schema2.json`)
- [x] Add a convenience `--ai-packet` flag to generate `ai.packet/` alongside common workflows:
  - `diag run <script.json> --ai-packet` (writes `<bundle_dir>/ai.packet/`)
  - `diag pack <bundle_dir> --ai-packet` (best-effort ensure before zipping)
- [x] Add a bounded share zip mode that packs only AI artifacts (no full bundle artifact):
  - `diag pack <bundle_dir> --ai-only` (packs `ai.packet/` + nearby script sources)
- [x] Add a sidecars-only mode to build ai packets without reading the bundle artifact:
  - `diag ai-packet <bundle_dir> --sidecars-only`
  - Goal: allow regenerating `ai.packet/` from a shared bundle dir where only sidecars are present (or the raw bundle is too large).
- [x] Make repro AI-only packing resilient when only sidecars are available for some items:
  - `diag repro ... --ai-only` should be able to generate `ai.packet/` from sidecars per item before packing.
- [x] Remove repeated ai.packet generation logic from pack/run/repro by centralizing in a single helper.
  - Evidence: `crates/fret-diag/src/commands/ai_packet.rs` (`ensure_ai_packet_dir_best_effort`), used by pack/run/repro.
- [x] Deduplicate common “looks like a path” + “resolve bundle artifact or latest” CLI parsing helpers across commands.
  - Evidence: `crates/fret-diag/src/commands/args.rs`, used by `agent`, `ai-packet`, `bundle-v2`, `hotspots`, `slice`.
- [x] Deduplicate “resolve latest bundle dir” helper.
  - Evidence: `crates/fret-diag/src/commands/args.rs` (`resolve_latest_bundle_dir_path`), used by `doctor`.
- [x] Remove remaining direct `read_latest_pointer` usage from `commands/*` (prefer shared helpers).
  - Evidence: `crates/fret-diag/src/commands/args.rs`, `crates/fret-diag/src/commands/session.rs`.
- [x] Deduplicate latest bundle resolution helpers outside `commands/*`.
  - Evidence: `crates/fret-diag/src/latest.rs`, `crates/fret-diag/src/diag_perf.rs`, `crates/fret-diag/src/post_run_checks.rs`,
    `crates/fret-diag/src/paths.rs`.
- [ ] Decide how far to push schema2-first:
  - [ ] Decide the runtime emission policy for `bundle.schema2.json` (tooling-derived today):
    - Proposed policy draft: `docs/workstreams/diag-fearless-refactor-v1/schema2-first-decision.md`.
    - [x] Implement an opt-in runtime companion artifact (`bundle.schema2.json`) emission path.
      - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics/config.rs` (`FRET_DIAG_BUNDLE_WRITE_SCHEMA2`),
        `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump.rs`.
    - [x] Make launched tooling workflows auto-enable runtime schema2 emission for schema2/AI-focused flows.
      - Evidence: `crates/fret-diag/src/lib.rs` (injects `FRET_DIAG_BUNDLE_WRITE_SCHEMA2=1` for `--launch` when
        `--ai-packet` / `--ai-only` / `--pack-schema2-only` are set).
    - [ ] Decide whether scripted runs should default to emitting schema2, and whether raw `bundle.json` can be skipped.
  - [ ] Decide when it is acceptable to stop treating raw `bundle.json` as a required artifact for common flows
    (keep it supported for deep debugging).

## Schema migration hygiene

- [ ] Decide the migration policy for schema v1 → v2:
  - [x] Document a conservative draft policy (phases + exit criteria):
    - `docs/workstreams/diag-fearless-refactor-v1.md`
  - [ ] Decide when to flip manual dump defaults to v2 (owner decision).
  - [ ] Decide deprecation messaging + migration recipes for older v1 repros.
- [x] Add one regression guard that prevents re-introducing forked protocol types in the runtime runner.
  - Evidence: `crates/fret-diag-protocol/src/lib.rs` (`DiagScreenshotRequestV1` / `DiagScreenshotResultFileV1`),
    `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs` (request writer),
    `crates/fret-launch/src/runner/desktop/runner/diag_screenshots.rs` (reader + result writer).
- [x] Consolidate semantics traversal helpers in `crates/fret-diag/src/json_bundle.rs`:
  - [x] Treat explicit inline `null` semantics as "missing" (fall back to schema2 semantics table).
  - [x] Centralize semantics table presence scanning for in-place schema conversion.
  - [x] Centralize the streaming schema2 semantics table reader used by `diag slice`.
  - Evidence: `crates/fret-diag/src/json_bundle.rs` (`SemanticsResolver`, `SemanticsTablePresence`), `crates/fret-diag/src/commands/bundle_v2.rs`.

## Tooling modularization (reduce single-file blast radius)

- [x] Finish modularizing `diag ai-packet` and remove the temporary monolith module once parity is proven:
  - [x] delete `crates/fret-diag/src/commands/ai_packet/monolith.rs`
  - [x] keep module boundaries stable (`budget`, `anchors`, `slices`, `fs`)
- [x] Extract triage JSON generation out of the CLI entrypoint file:
  - [x] Move `triage_json_from_stats` into `crates/fret-diag/src/triage_json.rs` (keep a thin wrapper in `crates/fret-diag/src/lib.rs`).
  - [x] Keep existing call sites unchanged (`crate::triage_json_from_stats`), and keep tests compiling.

## Plan 2 (defer until Plan 1 is solid)

- [ ] Prototype manifest-first chunked bundle layout (snapshots/logs/semantics split).
- [ ] Add a compatibility materializer to emit `bundle.json` from the manifest.
- [ ] Add packing/hashing conventions and a `diag pack` integration path.

## Debt removal (remove the baggage)

Keep this section aligned with:

- `docs/workstreams/diag-fearless-refactor-v1/debt-removal.md`
- `docs/workstreams/diag-fearless-refactor-v1/redundancy-removal-checklist.md`

- [ ] Start removing medium-risk compatibility outputs (after a deprecation window):
  - stop writing legacy JSON alias keys (keep reading them longer),
  - reduce flag alias sprawl where it hurts discoverability.
- [ ] Decide the “raw bundle.json optional” policy for scripted runs (high risk; requires exit criteria).
