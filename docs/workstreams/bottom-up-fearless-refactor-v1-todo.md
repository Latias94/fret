# Bottom-Up Fearless Refactor v1 — TODO Tracker

Status: Active (workstream tracker)

This document tracks cross-cutting TODOs for:

- `docs/workstreams/bottom-up-fearless-refactor-v1.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `BU-FR-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `fretboard diag` script/suite name

## M0 — Guardrails first

- [ ] BU-FR-guard-001 Define the canonical “refactor safety” command set and keep it stable in one place.
  - Candidates: `pwsh -NoProfile -File tools/check_layering.ps1`, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run`.
  - Prefer documenting the minimal subsets that catch most regressions early (e.g. `-p fret-ui`, `-p fret-ui-shadcn`).
  - Prefer defining “Fast vs Full” gate tiers so contributors know what to run in the inner loop vs before merge.

- [ ] BU-FR-guard-002 Add a short “how to add a regression gate” appendix to this workstream (unit test vs `fretboard diag`).
  - Link: `docs/ui-diagnostics-and-scripted-tests.md`

- [ ] BU-FR-guard-003 Inventory the current scripted diagnostics suites and map them to program milestones.
  - Start from: `docs/ui-diagnostics-and-scripted-tests.md` and `tools/diag-scripts/`

- [ ] BU-FR-guard-004 Convert “huge Rust conformance sources” into data-driven harnesses where possible.
  - Target examples: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`, `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`.
  - Goal: move scenario matrices/expected values to `goldens/*.json` (or a dedicated fixtures directory) and keep a thin Rust harness.

- [x] BU-FR-guard-005 Add a lightweight “largest files report” to keep module size drift visible.
  - Goal: prevent new god files from appearing unnoticed during refactors.
  - Evidence:
    - `tools/report_largest_files.ps1`

## M0.5 — Code-quality audit program (make best-practice reviews repeatable)

- [ ] BU-FR-audit-006 Add a stable per-crate audit template and an audits index.
  - Goal: make “read each crate and review best practices” actionable and trackable.
  - Evidence:
    - `docs/workstreams/bottom-up-fearless-refactor-v1-crate-audit-template.md`
    - `docs/workstreams/bottom-up-fearless-refactor-v1-crate-audits.md`
    - `docs/workstreams/crate-audits/fret-core.l0.md`
    - `docs/workstreams/crate-audits/fret-runtime.l0.md`
    - `docs/workstreams/crate-audits/fret-app.l0.md`
    - `docs/workstreams/crate-audits/fret-ui.l0.md`

- [ ] BU-FR-audit-007 Decide audit levels + minimum gates per level (L0/L1/L2).
  - Goal: scale audits across a large workspace without blocking progress.
  - Start from:
    - `docs/workstreams/bottom-up-fearless-refactor-v1.md`

## M1 — Core contracts closure

- [ ] BU-FR-core-010 Produce a short “core kernel surface map” (what is stable, what is experimental) for:
  - `crates/fret-core`, `crates/fret-runtime`, `crates/fret-app`
  - Tie it back to ADR tiers (ADR 0066) and to `docs/runtime-contract-matrix.md` where applicable.

- [ ] BU-FR-core-011 Audit `fret-core` exports and propose a minimal prelude or re-export strategy (if needed).
  - Evidence anchors: `crates/fret-core/src/lib.rs`

- [x] BU-FR-core-012 Add a “module ownership map” section for each core crate as it is refactored.
  - Start with: `crates/fret-core`, then `crates/fret-runtime`, then `crates/fret-app`.
  - Goal: make “where should new code go?” obvious to humans and AI alike.
  - Evidence:
    - `crates/fret-core/README.md`
    - `crates/fret-runtime/README.md`
    - `crates/fret-app/README.md`

- [~] BU-FR-core-013 Write down an explicit async policy per layer (core vs app vs backends vs ecosystem) and add at least one regression gate.
  - Goal: prevent Tokio/executor coupling from leaking into core crates and prevent UI-thread blocking regressions.
  - References: `docs/integrating-tokio-and-reqwest.md`, `docs/integrating-sqlite-and-sqlx.md`.
  - Evidence:
    - `docs/workstreams/bottom-up-fearless-refactor-v1.md` (section “Async policy”)

- [ ] BU-FR-core-014 Define a v1 “serialization stability” checklist for core persisted formats.
  - Target: settings/keymap/layout and any persisted docking state formats.
  - Goal: avoid accidental format drift during internal refactors.

- [x] BU-FR-core-015 Convert docking-related crate-root modules in `fret-core` into a single `dock/` subsystem.
  - Goal: remove `dock_*` crate-root prefixes while keeping the stable re-export surface unchanged.
  - Evidence:
    - `crates/fret-core/src/dock/mod.rs`
    - `crates/fret-core/src/dock/layout.rs`
    - `crates/fret-core/src/dock/op.rs`
    - `crates/fret-core/src/lib.rs`

- [x] BU-FR-core-016 Extract docking persistence helpers and tests out of `dock/mod.rs`.
  - Goal: shrink the main runtime dock graph module and keep layout IO/persistence code discoverable.
  - Evidence:
    - `crates/fret-core/src/dock/persistence.rs`
    - `crates/fret-core/src/dock/tests.rs`

- [x] BU-FR-core-017 Convert `fret-core` scene module to a directory subsystem (`scene/mod.rs`).
  - Goal: enable incremental internal splits of scene recording/validation without crate-root churn.
  - Evidence:
    - `crates/fret-core/src/scene/mod.rs`

- [x] BU-FR-core-018 Convert `fret-core` input module to a directory subsystem (`input/mod.rs`).
  - Goal: enable incremental internal splits of pointer/key/IME/viewport input normalization without crate-root churn.
  - Evidence:
    - `crates/fret-core/src/input/mod.rs`

- [x] BU-FR-core-019 Extract `DockOp` application out of `dock/mod.rs` into a dedicated submodule.
  - Goal: keep the dock graph module focused on core mutation primitives while isolating the large op match.
  - Evidence:
    - `crates/fret-core/src/dock/apply.rs`
    - `crates/fret-core/src/dock/mod.rs`

- [x] BU-FR-core-020 Split docking query/mutation helpers into focused submodules.
  - Goal: keep `dock/mod.rs` as a small facade while enabling incremental refactors of docking algorithms.
  - Evidence:
    - `crates/fret-core/src/dock/query.rs`
    - `crates/fret-core/src/dock/mutate.rs`
    - `crates/fret-core/src/dock/mod.rs`

- [x] BU-FR-core-021 Split scene validation and replay helpers into dedicated submodules.
  - Goal: keep `scene/mod.rs` as a stable facade while enabling incremental internal splits of recording/validation/replay.
  - Evidence:
    - `crates/fret-core/src/scene/validate.rs`
    - `crates/fret-core/src/scene/replay.rs`
    - `crates/fret-core/src/scene/mod.rs`

- [x] BU-FR-core-022 Extract scene fingerprint mixing helpers into a dedicated submodule.
  - Goal: keep the scene facade focused on vocabulary and recording APIs, while isolating hashing/fingerprinting logic.
  - Evidence:
    - `crates/fret-core/src/scene/fingerprint.rs`
    - `crates/fret-core/src/scene/mod.rs`

- [x] BU-FR-core-023 Convert `fret-core` text module to a directory subsystem (`text/mod.rs`).
  - Goal: enable incremental internal splits of text vocabulary (styles, spans, hit testing) without crate-root churn.
  - Evidence:
    - `crates/fret-core/src/text/mod.rs`

- [x] BU-FR-core-024 Extract keyboard code helpers into a dedicated input submodule.
  - Goal: keep `input/mod.rs` focused on the portable event vocabulary while isolating typeahead helpers.
  - Evidence:
    - `crates/fret-core/src/input/keyboard.rs`
    - `crates/fret-core/src/input/mod.rs`

- [x] BU-FR-core-025 Extract viewport input types and mapping helpers into dedicated submodules.
  - Goal: keep `input/mod.rs` focused on portable event vocabulary, while isolating viewport-tooling glue (ADR 0147).
  - Evidence:
    - `crates/fret-core/src/input/viewport.rs`
    - `crates/fret-core/src/input/viewport_input_event_tests.rs`
    - `crates/fret-core/src/input/mod.rs`

- [x] BU-FR-core-026 Convert `fret-runtime` menubar model + patching + wire formats into a directory subsystem.
  - Goal: split the large `menu.rs` into focused submodules without widening the public contract surface.
  - Evidence:
    - `crates/fret-runtime/src/menu/mod.rs`
    - `crates/fret-runtime/src/menu/model.rs`
    - `crates/fret-runtime/src/menu/patch.rs`
    - `crates/fret-runtime/src/menu/apply.rs`
    - `crates/fret-runtime/src/menu/wire.rs`
    - `crates/fret-runtime/src/menu/tests.rs`

- [x] BU-FR-core-027 Convert `fret-runtime` keymap module to a directory subsystem (`keymap/mod.rs`).
  - Goal: remove ambiguous `keymap.rs` / `keymap/` split and enable incremental internal splits by responsibility.
  - Evidence:
    - `crates/fret-runtime/src/keymap/mod.rs`

- [x] BU-FR-core-028 Convert `fret-runtime` model store into a directory subsystem (`model/mod.rs`).
  - Goal: split the large model store implementation by responsibility while keeping the stable re-export surface unchanged.
  - Evidence:
    - `crates/fret-runtime/src/model/mod.rs`
    - `crates/fret-runtime/src/model/store.rs`
    - `crates/fret-runtime/src/model/handle.rs`
    - `crates/fret-runtime/src/model/host.rs`
    - `crates/fret-runtime/src/model/debug.rs`
    - `crates/fret-runtime/src/model/error.rs`

- [x] BU-FR-core-029 Split `fret-runtime` keymap logic into focused submodules.
  - Goal: keep `keymap/mod.rs` as a small facade and isolate loading, matching, conflicts, and display logic.
  - Evidence:
    - `crates/fret-runtime/src/keymap/mod.rs`
    - `crates/fret-runtime/src/keymap/load.rs`
    - `crates/fret-runtime/src/keymap/ops.rs`
    - `crates/fret-runtime/src/keymap/conflicts.rs`
    - `crates/fret-runtime/src/keymap/display.rs`
    - `crates/fret-runtime/src/keymap/types.rs`
    - `crates/fret-runtime/src/keymap/wire.rs`
    - `crates/fret-runtime/src/keymap/error.rs`

- [x] BU-FR-core-030 Convert `fret-runtime` window command gating to a directory subsystem (`window_command_gating/mod.rs`).
  - Goal: split service state, snapshot data, and helper construction functions without changing the contract surface.
  - Evidence:
    - `crates/fret-runtime/src/window_command_gating/mod.rs`
    - `crates/fret-runtime/src/window_command_gating/service.rs`
    - `crates/fret-runtime/src/window_command_gating/snapshot.rs`
    - `crates/fret-runtime/src/window_command_gating/helpers.rs`

- [x] BU-FR-core-031 Split `fret-runtime` menubar wire decoding into submodules (`menu/wire/*`).
  - Goal: isolate v1/v2 wire structs, patch ops decoding, and config parsing while keeping the stable exports unchanged.
  - Evidence:
    - `crates/fret-runtime/src/menu/wire/mod.rs`
    - `crates/fret-runtime/src/menu/wire/load.rs`
    - `crates/fret-runtime/src/menu/wire/v1.rs`
    - `crates/fret-runtime/src/menu/wire/v2.rs`
    - `crates/fret-runtime/src/menu/wire/patch_v1.rs`
    - `crates/fret-runtime/src/menu/wire/patch_v2.rs`
    - `crates/fret-runtime/src/menu/wire/config_v1.rs`
    - `crates/fret-runtime/src/menu/wire/config_v2.rs`

- [x] BU-FR-core-032 Convert `fret-runtime` platform capabilities to a directory subsystem (`capabilities/mod.rs`).
  - Goal: keep capability keys/kinds/quality enums and aggregation logic in focused submodules without changing re-exports.
  - Evidence:
    - `crates/fret-runtime/src/capabilities/mod.rs`
    - `crates/fret-runtime/src/capabilities/kind.rs`
    - `crates/fret-runtime/src/capabilities/exec.rs`
    - `crates/fret-runtime/src/capabilities/qualities.rs`
    - `crates/fret-runtime/src/capabilities/leaf.rs`
    - `crates/fret-runtime/src/capabilities/platform.rs`

## M2 — UI runtime closure

- [x] BU-FR-ui-020 Create a “top 10 refactor hazards” list for `crates/fret-ui` and the gates that cover them.
  - Must include: layout recursion hazards, overlay dismissal drift, IME key arbitration, view-cache reuse drift.
  - Evidence:
    - `docs/workstreams/bottom-up-fearless-refactor-v1.md`

- [ ] BU-FR-ui-021 Link the “closure targets” in this program doc to the existing P0 closure tracker and keep them in sync.
  - Primary: `docs/workstreams/foundation-closure-p0.md`, `docs/workstreams/foundation-closure-p0-todo.md`

- [ ] BU-FR-ui-022 Decide the minimal “authoring ergonomics” convergence target (v1) across:
  - `ecosystem/fret-ui-kit`, `ecosystem/fret-ui-shadcn`, `ecosystem/fret-kit`
  - Scope: fluent builder conventions, `test_id` conventions, and “cached subtree” guidance.

- [x] BU-FR-ui-023 Reduce crate-root “prefix modules” by regrouping related code under subsystem modules.
  - Example: prefer `text/{edit,props,surface}.rs` over `text_edit.rs`, `text_props.rs`, `text_surface.rs`.
  - Goal: improve ownership clarity and reduce “where does this belong?” drift during fearless refactors.
  - Evidence:
    - `crates/fret-ui/src/text/`
    - `crates/fret-ui/src/text/mod.rs`

- [x] BU-FR-ui-024 Regroup text subsystem modules in `crates/fret-ui` under a single `text/` module.
  - Current: `text_input/`, `text_area/`, plus crate-root `text_*` files.
  - Target: `crates/fret-ui/src/text/mod.rs` with clear submodules (`input`, `area`, `edit`, `props`, `style`, `surface`), keeping public exports unchanged.
  - Evidence:
    - `crates/fret-ui/src/text/mod.rs`
    - `crates/fret-ui/src/text/area/mod.rs`
    - `crates/fret-ui/src/text/input/mod.rs`

- [x] BU-FR-ui-025 Regroup layout-related crate-root modules under a dedicated `layout/` subsystem module.
  - Candidates: `layout_constraints.rs`, `layout_pass.rs`, `layout_engine.rs`, and related helpers.
  - Goal: make layout ownership clear and reduce cross-module entanglement.
  - Evidence:
    - `crates/fret-ui/src/layout/mod.rs`
    - `crates/fret-ui/src/layout/engine.rs`
    - `crates/fret-ui/src/layout/constraints.rs`
    - `crates/fret-ui/src/layout/pass.rs`

- [x] BU-FR-ui-026 Regroup theme-related crate-root modules under a dedicated `theme/` subsystem module.
  - Candidates: `theme.rs`, `theme_keys.rs`, `theme_registry.rs`.
  - Goal: keep theme/token code co-located and reduce crate-root prefix drift.
  - Evidence:
    - `crates/fret-ui/src/theme/mod.rs`
    - `crates/fret-ui/src/theme/keys.rs`
    - `crates/fret-ui/src/theme/registry.rs`

- [x] BU-FR-ui-027 Convert scroll-related crate-root modules into subsystem directories.
  - Candidates: `scroll.rs`, `virtual_list.rs`.
  - Goal: keep `scroll` / `virtual_list` as stable module names while enabling further internal split without adding new crate-root prefixes.
  - Evidence:
    - `crates/fret-ui/src/scroll/mod.rs`
    - `crates/fret-ui/src/virtual_list/mod.rs`

## M3 — Renderer closure

- [x] BU-FR-render-030 Define a minimal renderer regression surface list (text, svg, atlas, clip/shadow).
  - Link: `docs/renderer-refactor-roadmap.md`
  - Evidence anchors:
    - `docs/workstreams/bottom-up-fearless-refactor-v1.md`
    - `crates/fret-render/src/renderer/mod.rs`
    - `crates/fret-render/src/text/mod.rs`
    - `crates/fret-render/src/svg/mod.rs`

- [ ] BU-FR-render-031 Inventory the current profiling/inspection workflows and ensure they are runnable on Windows.
  - Link: `docs/tracy.md`, `docs/renderdoc-inspection.md`

- [x] BU-FR-render-032 Add a module ownership map for `crates/fret-render` and document the intended public surface.
  - Goal: make renderer code easier to navigate while keeping runner-facing exports intentional.
  - Evidence:
    - `crates/fret-render/README.md`
    - `crates/fret-render/src/lib.rs`

- [x] BU-FR-render-033 Convert renderer crate-root “module roots” into `mod.rs` subsystems where a directory already exists.
  - Candidates: `text.rs` → `text/mod.rs`, `viewport_overlay.rs` → `viewport_overlay/mod.rs`.
  - Goal: remove ambiguous `foo.rs` / `foo/` splits and enable incremental internal splits without crate-root churn.
  - Evidence:
    - `crates/fret-render/src/text/mod.rs`
    - `crates/fret-render/src/viewport_overlay/mod.rs`

- [x] BU-FR-render-034 Regroup SVG rasterization + caching under a single `svg/` subsystem module.
  - Candidates: `svg.rs`, `svg_cache.rs`.
  - Goal: keep SVG ownership clear and avoid adding more crate-root prefix modules as SVG grows.
  - Evidence:
    - `crates/fret-render/src/svg/mod.rs`
    - `crates/fret-render/src/svg/cache.rs`

## M3.5 — Platform + runner closure

- [x] BU-FR-plat-050 Add a module ownership map for `crates/fret-platform` and document the intended public surface.
  - Goal: make portable platform contracts easy to discover and keep call sites stable.
  - Evidence:
    - `crates/fret-platform/README.md`
    - `crates/fret-platform/src/lib.rs`

- [x] BU-FR-plat-051 Add a module ownership map for `crates/fret-platform-native` and document the intended public surface.
  - Goal: make desktop implementations discoverable and keep exports intentional.
  - Evidence:
    - `crates/fret-platform-native/README.md`
    - `crates/fret-platform-native/src/lib.rs`

- [x] BU-FR-plat-052 Add a module ownership map for `crates/fret-platform-web` and document the intended public surface.
  - Goal: clarify the wasm32-only posture and avoid accidental platform coupling.
  - Evidence:
    - `crates/fret-platform-web/README.md`
    - `crates/fret-platform-web/src/lib.rs`

- [x] BU-FR-plat-053 Convert `crates/fret-platform-web` module roots to `mod.rs` where a directory is the intended owner.
  - Candidates: `wasm.rs` → `wasm/mod.rs`, `native.rs` → `native/mod.rs`.
  - Goal: avoid ambiguous `foo.rs` / `foo/` splits and enable incremental internal splits without crate-root churn.
  - Evidence:
    - `crates/fret-platform-web/src/wasm/mod.rs`
    - `crates/fret-platform-web/src/native/mod.rs`

- [x] BU-FR-plat-054 Split `crates/fret-platform-web` wasm implementation into focused submodules.
  - Goal: keep `WebPlatformServices` as a small facade while enabling incremental refactors of IME/timers/file dialogs with minimal churn.
  - Evidence:
    - `crates/fret-platform-web/src/wasm/mod.rs`
    - `crates/fret-platform-web/src/wasm/ime.rs`
    - `crates/fret-platform-web/src/wasm/timers.rs`
    - `crates/fret-platform-web/src/wasm/file_dialog.rs`

- [x] BU-FR-runner-060 Add a module ownership map for `crates/fret-runner-winit` and document the intended public surface.
  - Goal: make runner glue discoverable without dragging policy/renderer responsibilities into the wrong layer.
  - Evidence:
    - `crates/fret-runner-winit/README.md`
    - `crates/fret-runner-winit/src/lib.rs`

- [x] BU-FR-runner-061 Add a module ownership map for `crates/fret-runner-web` and document the intended public surface.
  - Goal: keep wasm runner posture explicit and avoid accidental cross-target coupling.
  - Evidence:
    - `crates/fret-runner-web/README.md`
    - `crates/fret-runner-web/src/lib.rs`

- [x] BU-FR-launch-062 Add a module ownership map for `crates/fret-launch` and document the intended public surface.
  - Goal: make the runner facade entrypoints easy to navigate while keeping runner internals behind a stable facade.
  - Evidence:
    - `crates/fret-launch/README.md`
    - `crates/fret-launch/src/lib.rs`

- [x] BU-FR-launch-070 Split `crates/fret-launch/src/runner/common.rs` into focused submodules.
  - Goal: reduce a single large “misc glue” file into stable contracts (`WinitAppDriver`, contexts, config) and support types.
  - Evidence:
    - `crates/fret-launch/src/runner/common/mod.rs`
    - `crates/fret-launch/src/runner/common/config.rs`
    - `crates/fret-launch/src/runner/common/context.rs`
    - `crates/fret-launch/src/runner/common/engine_frame_update.rs`
    - `crates/fret-launch/src/runner/common/fn_driver.rs`
    - `crates/fret-launch/src/runner/common/viewport_overlay_3d.rs`
    - `crates/fret-launch/src/runner/common/window_create_spec.rs`
    - `crates/fret-launch/src/runner/common/winit_app_driver.rs`

- [x] BU-FR-launch-071 Convert `crates/fret-launch/src/runner/web.rs` into a directory module facade.
  - Goal: make `runner/web/` the single owner so the wasm runner can be split into stable submodules without churn.
  - Evidence:
    - `crates/fret-launch/src/runner/web/mod.rs`

- [x] BU-FR-launch-072 Extract wasm IME mount plumbing into a dedicated web runner submodule.
  - Goal: keep DOM/IME wrapper details isolated from the event loop and GPU plumbing.
  - Evidence:
    - `crates/fret-launch/src/runner/web/ime_mount.rs`

- [x] BU-FR-launch-073 Extract streaming image update plumbing into a dedicated web runner submodule.
  - Goal: isolate the streaming image contract + GPU YUV path from the rest of the web runner.
  - Evidence:
    - `crates/fret-launch/src/runner/web/streaming_images.rs`

- [x] BU-FR-launch-074 Extract the wasm runner `ApplicationHandler` impl into a dedicated submodule.
  - Goal: keep winit event-loop glue isolated so the remaining `web/mod.rs` can focus on runner state and contracts.
  - Evidence:
    - `crates/fret-launch/src/runner/web/app_handler.rs`

- [x] BU-FR-launch-075 Extract the wasm runner render loop and turn draining into a dedicated submodule.
  - Goal: isolate frame driving (`render_frame`) and effect/event fixed-point draining (`drain_turns`) from the rest of the web runner.
  - Evidence:
    - `crates/fret-launch/src/runner/web/render_loop.rs`

- [x] BU-FR-launch-076 Extract wasm GPU adoption and surface sizing helpers into a dedicated submodule.
  - Goal: isolate async GPU adoption + font seeding and canvas DPI sizing from the frame loop and the event handler glue.
  - Evidence:
    - `crates/fret-launch/src/runner/web/gfx_init.rs`

- [x] BU-FR-launch-077 Extract wasm effect draining (`drain_effects`) into a dedicated submodule.
  - Goal: isolate the large `Effect` matching and streaming snapshot updates from the frame loop and event handler glue.
  - Evidence:
    - `crates/fret-launch/src/runner/web/effects.rs`

- [x] BU-FR-runner-063 Split `crates/fret-runner-winit` crate-root implementation into `mapping/` + `state/` subsystems.
  - Goal: shrink `src/lib.rs` to a stable facade while enabling incremental internal refactors without widening the public surface.
  - Evidence:
    - `crates/fret-runner-winit/src/lib.rs`
    - `crates/fret-runner-winit/src/mapping/mod.rs`
    - `crates/fret-runner-winit/src/state/mod.rs`
    - `crates/fret-runner-winit/src/state/input/mod.rs`

## M4 — Ecosystem rationalization

- [ ] BU-FR-eco-040 Maintain an allowlist for any crate using `fret-ui/unstable-retained-bridge`, and keep the list shrinking.
  - Link: `docs/workstreams/retained-bridge-exit-v1.md`
  - Gate: `tools/check_layering.ps1` (extend if needed)

- [ ] BU-FR-eco-041 Map ecosystem crates into “headless → kit → shadcn → specialized” lanes and mark ownership boundaries.
  - Start from: `docs/repo-structure.md`, `docs/workstreams/ecosystem-status.md`
