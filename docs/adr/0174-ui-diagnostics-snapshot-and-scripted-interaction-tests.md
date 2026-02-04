---
title: "ADR 0174: UI Diagnostics Snapshot + Scripted Interaction Tests (GPUI Inspector-Aligned)"
---

# ADR 0174: UI Diagnostics Snapshot + Scripted Interaction Tests (GPUI Inspector-Aligned)

Status: Proposed

Scope: debug/observability and test tooling surfaces spanning `crates/fret-ui` (mechanism hooks),
runner integration (`crates/fret-launch`), and developer tooling (`apps/fretboard` / ecosystem tooling crates).

This ADR is intentionally **tooling-oriented** and must not expand `crates/fret-ui` into a policy or component layer
(see ADR 0066). The runtime only provides **mechanism hooks** and small, versioned data shapes.

Related:

- ADR 0036 (Observability strategy): `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
- ADR 0015 (Frame lifecycle): `docs/adr/0015-frame-lifecycle-and-submission-order.md`
- ADR 0028 (Declarative element model): `docs/adr/0028-declarative-elements-and-element-state.md`
- ADR 0020 (Focus/command routing): `docs/adr/0020-focus-and-command-routing.md`
- ADR 0033 (Semantics / a11y): `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- ADR 0066 (`fret-ui` contract surface gates): `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- ADR 0043 (Pending bindings / shortcut arbitration): `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`
- ADR 0017 (DPI / coordinate semantics): `docs/adr/0017-multi-window-display-and-dpi.md`

Non-normative references (Zed/GPUI):

- Inspector registry + picking lifecycle: `repo-ref/zed/crates/gpui/src/inspector.rs`
- “Disable caching while picking” behavior: `repo-ref/zed/crates/gpui/src/view.rs` (`window.is_inspector_picking(cx)`)
- Input dispatch and inspector short-circuit: `repo-ref/zed/crates/gpui/src/window.rs`

## Context

Fret targets editor-grade UI where correctness and “feel” bugs are often cross-cutting:

- input routing and focus/capture across multiple overlay roots,
- invalidation propagation (model/global changes) to layout/paint,
- layout performance (unnecessary solves),
- text/IME edge cases and platform differences,
- docking/drag arbitration and cross-window behavior.

Today, many of the required internal signals exist as ad-hoc pieces:

- `UiTree` exposes debug stats and hit-test snapshots,
- the runner tracks changed models/globals and drains effects deterministically,
- renderer and text system have their own perf snapshots.

But there is no stable, end-to-end workflow for:

1) capturing a **single, structured “diagnostics bundle”** that is sufficient to debug a bug (including by an AI),
2) writing **behavior tests** that reproduce “click this then press that” style issues without brittle pixel/coordinate coupling,
3) replaying a recorded interaction deterministically to validate a fix.

Zed/GPUI provides a useful precedent: an inspector that can pick elements, disable caching while picking to ensure hitboxes
are available, and store per-element inspector state via a registry.

This ADR locks the minimal tooling contracts needed to make Fret similarly debuggable while respecting Fret’s layering rules.

## Goals

1. Provide a versioned, machine-readable UI diagnostics snapshot that can be exported and attached to bug reports.
2. Make “AI-assisted debugging” practical by ensuring the snapshot is **self-contained** and **cross-crate**.
3. Enable scripted interaction tests that select UI targets by **semantics/test IDs**, not raw coordinates.
4. Support deterministic record/replay workflows for hard-to-debug regressions.
5. Keep runtime layering clean:
   - `crates/fret-ui` exposes only mechanism hooks and minimal data snapshots,
   - tooling and policies live above (`fret-ui-app`, `fretboard`, ecosystem crates).

## Non-Goals

- Shipping a production-grade inspector UI as part of core runtime.
- Cross-process debugging/IPC transport (ADR 0036 says in-process only for P0).
- Screenshot-based golden testing as the primary correctness mechanism.
- Perfect platform-level IME reproduction in CI (IME will remain “best effort” for replay).

## Decision

### 1) Define a versioned UI diagnostics snapshot (JSON-first)

Introduce a versioned data shape:

- `UiDiagnosticsSnapshotV1`

This snapshot is **data-only** (serde-friendly) and MUST be exportable to JSON.

Minimum required fields (v1):

**Identity / timing**

- `schema_version: u32` (must be `1`)
- `tick_id: TickId`
- `frame_id: FrameId`
- `window: AppWindowId`
- `timestamp_mono_ns: u64` (monotonic best-effort; optional on wasm)

**Window / DPI**

- `scale_factor: f32`
- `window_size_logical: (f32, f32)`

**Input + routing**

- `recent_events: Vec<RecordedUiEventV1>` (bounded ring buffer; see section 2)
- `last_hit_test: Option<UiHitTestSnapshotV1>` (or equivalent; includes active layer roots and barrier root)

**Focus / capture**

- `focused_node: Option<NodeId>`
- `captured_by_pointer: Vec<(PointerId, NodeId)>` (or a compact map)
- `active_modal_root: Option<UiLayerId>` (or equivalent barrier/root marker)
- `last_input_modality: Option<InputModality>` (for focus-visible debugging)

**Layers / overlays**

- `layers_in_paint_order: Vec<UiLayerSnapshotV1>` (id/root/flags in window-local logical px terms)

**Invalidation + layout/paint work**

- `layout_time_us: u64`, `paint_time_us: u64`
- `layout_engine_solves: u64`, `layout_engine_solve_time_us: u64`
- `paint_cache_hits: u32`, `paint_cache_misses: u32`, `paint_cache_replayed_ops: u32`
- `reason_hints: Vec<UiWorkReasonV1>` (model/global/timer/raf/input/engine)

**Observation / invalidation linkage (critical for AI debugging)**

- `changed_models: Vec<ModelId>` (from runner)
- `changed_globals: Vec<String>` (TypeId stringified in a stable way; see privacy note below)
- `observed_models: Vec<(GlobalElementId, Vec<(ModelId, Invalidation)>)>` (or a compact form)
- `observed_globals: Vec<(GlobalElementId, Vec<(String, Invalidation)>)>`

**Renderer perf (optional, but recommended in v1)**

- `renderer: Option<RendererSnapshotV1>` (batch counts, atlas stats, budgets, bytes uploaded)

Privacy and portability rules:

- Do not record absolute file paths.
- Type identities MUST use stable, non-path strings (e.g. `std::any::type_name::<T>()`) and MUST NOT include machine-local
  paths.

Implementation note (non-normative):

- The current bundle format exports renderer metrics as flat counters under
  `.windows[].snapshots[].debug.stats.renderer_*` when enabled:
  - timings: `renderer_encode_scene_us`, `renderer_prepare_text_us`, `renderer_prepare_svg_us`
  - batching: `renderer_draw_calls`, `renderer_pipeline_switches`, `renderer_bind_group_switches`, `renderer_scissor_sets`
  - churn (best-effort per-frame): `renderer_text_atlas_upload_bytes`, `renderer_text_atlas_evicted_pages`,
    `renderer_intermediate_peak_in_use_bytes`, `renderer_intermediate_pool_evictions`
  - cache: `renderer_scene_encoding_cache_misses`

### 2) Provide per-window ring buffers and a “diagnostics bundle” export

The runner MUST maintain per-window bounded ring buffers for:

- recent core UI events (`fret-core::Event`) after normalization,
- recent effects drained (including commands, docking ops, IME cursor-area updates),
- recent `UiDiagnosticsSnapshotV1` samples (e.g. last ~300 frames).

Export shape (bundle directory):

- a directory under `target/fret-diag/<timestamp>/` by default (or an OS temp dir on wasm),
- with at least:
  - `bundle.json` (the exported diagnostics bundle; includes bounded per-window snapshot history),
  - `latest.txt` pointer file in the parent output directory (best-effort),
  - optional screenshot artifacts when enabled (tooling-driven; see `docs/ui-diagnostics-and-scripted-tests.md`).

This “bundle” is the unit of sharing with other humans and with AI tools.

Implementation note (non-normative):

- The export directory SHOULD be configurable via a CLI flag (e.g. `--out-dir`) and/or an environment variable
  (e.g. `FRET_DIAG_DIR`) so developers can redirect bundles into `.fret/diag/` when desired.
- Implementations MAY allow disabling semantics capture if it is too heavy for a given repro (e.g. `FRET_DIAG_SEMANTICS=0`).

### 3) Define “inspector picking mode” semantics (GPUI-aligned)

To support accurate picking and scripted selection, introduce a debug-only “picking” mode.

When picking mode is active:

1. Input dispatch MUST short-circuit to the picking handler before other mouse handling (GPUI-style).
2. Any caching that can hide hit-testable geometry MUST be disabled for the duration of picking:
   - subtree replay caching / paint cache shortcuts must not cause missing hitboxes.
3. Picking results MUST report both:
   - retained identity: `NodeId`,
   - declarative identity: `GlobalElementId` (when available).

Rationale:

- GPUI disables caching while picking to ensure mouse hit testing has full hitboxes.
- Fret must provide the same “debug truth” property; otherwise behavior tests and inspector workflows become flaky.

### 4) Scripted interaction tests select targets by Semantics + optional Test IDs

Introduce a stable selector vocabulary for tests:

- `UiSelectorV1` with at least:
  - `TestId(String)` (best effort stable; requires authoring support)
  - `RoleAndName { role: SemanticsRole, name: String }`
  - `RoleAndPath { role: SemanticsRole, ancestors: Vec<(SemanticsRole, String)> }`
  - `GlobalElementId(GlobalElementId)` (for low-level harness tests)

Selection MUST be evaluated against the current `SemanticsSnapshot` (ADR 0033), not the paint stream.

Test ID rules:

- Test IDs are debug/test-only metadata.
- They MUST NOT become a styling or policy hook and MUST NOT be required for production apps.
- They MUST NOT leak into accessibility labels by default (keep a11y semantics clean).
- They MUST NOT be mapped into platform accessibility “name/label” fields by the AccessKit bridge.
  - Tests may still select by `RoleAndName` using accessibility-facing fields; `TestId` is an explicit opt-in.

### 5) Define a minimal action script DSL and execution model

Define `UiActionScriptV1` as a sequence of steps:

Note: the implementation currently supports a v1 MVP step set, and also supports a schema v2 extension
(`UiActionScriptV2`) with intent-level steps (see `docs/ui-diagnostics-and-scripted-tests.md`).

- `OpenDemo { name }` (test harness only)
- `Click { target: UiSelectorV1 }`
- `DoubleClick { target: UiSelectorV1 }`
- `Drag { from: UiSelectorV1, to: UiSelectorV1, button, modifiers }`
- `TypeText { text }` (routes via focused text input)
- `PressKey { chord }` (uses keymap vocabulary)
- `Wheel { target: UiSelectorV1, delta }`
- `WaitFrames { n }`
- `WaitUntil { predicate, timeout_frames }` (predicate evaluates on `UiDiagnosticsSnapshotV1`)
- `Assert { predicate }`
- `CaptureBundle { label }` (exports diagnostics bundle on demand)

Execution rules:

- The harness MUST drive frame progression deterministically (no wall-clock dependency by default).
- Each action step MUST optionally emit a snapshot sample so failures can be debugged post-hoc.
- Picking-mode caching disablement SHOULD be enabled while resolving selectors (same reason as section 3).

Implementation note (non-normative):

- A practical MVP is a file-triggered harness: write `script.json`, then “poke” a `script.touch` trigger file; the running app polls this and executes one step per frame.
  - Suggested env vars for overrides: `FRET_DIAG_SCRIPT_PATH`, `FRET_DIAG_SCRIPT_TRIGGER_PATH`, `FRET_DIAG_SCRIPT_AUTO_DUMP`.

### 6) Record and replay workflows

Two record modes are standardized:

1) **Semantic action recording** (preferred):
- record `UiActionScriptV1` steps with semantic selectors (when resolvable),
- record enough context to re-resolve selectors (role/name/test_id).

2) **Raw input recording** (fallback):
- record normalized `fret-core::Event` streams with `TickId`/`FrameId` markers and window metadata.

Replay priority:

- Prefer semantic action replay. If a selector fails to resolve, fall back to raw-coordinate replay only if explicitly requested.

### 7) Crate boundaries and feature gating

**Kernel/runtime (`crates/fret-ui`)**

- MUST NOT add policy-heavy APIs.
- MAY add debug-only hooks to expose required state for snapshots (e.g. observed-models dumps), gated behind
  `cfg(debug_assertions)` or a dedicated feature flag (e.g. `fret-ui/diagnostics`).

**Runner (`crates/fret-launch`)**

- Owns ring buffers, bundle export, and integration with effects/model/global change propagation.

**Tooling (`apps/fretboard` and/or a dedicated ecosystem crate)**

- Owns:
  - parsing/executing `UiActionScriptV1`,
  - replay harness orchestration,
  - saving/loading bundles,
  - optional inspector overlay UI.

### 8) Validation requirements

To consider this subsystem “closed enough to scale”:

- At least one runtime-level behavior test uses `UiActionScriptV1` and semantic selectors to reproduce a real regression
  (focus, overlays, docking, virtualization, or text input).
- A “bundle export” test exists that ensures JSON is parseable and contains required fields.
- A small manual checklist exists for the inspector overlay (if/when implemented).

## Alternatives Considered

1) **Coordinate-only replay**
- Pros: simple to implement.
- Cons: extremely brittle under DPI/layout/font changes; poor CI stability; low AI utility.

2) **Screenshot golden tests**
- Pros: catches visual regressions.
- Cons: expensive and noisy; does not diagnose root cause; high flakiness across GPUs/platforms; not suitable as the
  primary editor-grade correctness strategy.

3) **Tracing-only**
- Pros: low overhead; useful for performance analysis.
- Cons: insufficient for routing/hit-test correctness and selector-based reproduction; requires too much human inference.

Chosen approach: a hybrid of **semantic selectors + structured snapshots + optional tracing**, aligned with GPUI’s inspector
practice and Fret’s determinism contracts.

## Implementation Plan (Non-Normative)

Suggested landing order:

1. Add `UiDiagnosticsSnapshotV1` + serde export surface (tooling crate).
2. Add runner ring-buffer + `.fret/diag/` bundle export (desktop + web best-effort).
3. Add debug hooks to export:
   - layer stack, focus/capture, last hit-test,
   - observed model/global tables for the current frame.
4. Add `UiSelectorV1` resolution via `SemanticsSnapshot`.
5. Add `UiActionScriptV1` executor + a small conformance test suite (nextest).
6. Add optional semantic recording for live demos (fretboard integration).
