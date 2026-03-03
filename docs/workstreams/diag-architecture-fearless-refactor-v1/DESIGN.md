# Diagnostics Architecture (Fearless Refactor v1)

Status: Draft (workstream note)

This workstream is an **umbrella refactor plan** for Fret’s diagnostics stack (“diag”):

- make the codebase **cleaner and more modular** (less monolith churn),
- make diagnostics **easy to extend** (especially from `ecosystem/*`),
- strengthen **layout correctness + layout performance** debugging,
- keep the current “artifact-first” philosophy: the portable unit remains a **bundle + evidence**.

This is not a “rewrite.” The posture is fearless refactor: **stabilize seams, isolate responsibilities, add gates,
then move code in small landable steps**.

## Why this exists

Diag already provides a lot of power:

- bundles (`bundle.schema2.json`) as portable repro artifacts,
- scripted actions + assertions (selectors/predicates),
- `fretboard diag` tooling (triage/lint/hotspots/perf/pack),
- early DevTools GUI + WS transport direction.

The main problems are architectural and long-term:

- “tooling engine” responsibilities are concentrated in a large crate (`crates/fret-diag`),
- runtime snapshot/export logic lives inside `ecosystem/fret-bootstrap` and is hard to extend without direct coupling,
- layout debugging exists (hotspots + Taffy dumps) but is not yet a coherent user-facing workflow,
- extension points exist informally (stores/snapshots), but are not a crisp contract.

## Goals

1. **Clean boundaries**
   - Protocol types stay in `crates/fret-diag-protocol`.
   - Transport stays in `crates/fret-diag-ws` (FS transport remains file-based; WS for web runner).
   - Tooling engine becomes modular and extensible, and frontends stay thin.
2. **Extensibility (ecosystem-first)**
   - Ecosystem crates can contribute diagnostics without central churn and without widening stable core surfaces.
3. **Layout correctness + performance**
   - “Correctness” = stable, inspectable geometry invariants per `test_id` / semantics.
   - “Performance” = explainable hotspots + tight regressions gates (solve/measure/paint).
4. **Artifact invariants**
   - A run yields a local, shareable artifact directory with stable filenames, plus bounded AI-friendly packets.
5. **Future-proofing**
   - Keep schema versioning explicit and additive.
   - Prefer “typed for core invariants + JSON extensions for ecosystem” over ad-hoc new fields forever.

## Non-goals (v1)

- Replace the semantics system (ADR 0033) or change the long-term UI authoring paradigm.
- Invent a new scripting language (JSON scripts remain the committed artifact).
- Ship a production-grade remote debugging product (local-only defaults remain fine).
- Perfect “layout explain like Flutter” in one step (we’ll stage toward it).

## Current architecture (today)

High-level layering (with evidence anchors):

- **Protocol (stable serde types):**
  - `crates/fret-diag-protocol/src/lib.rs`
- **Tooling engine (CLI/GUI shared):**
  - `crates/fret-diag/src/lib.rs`
- **WS transport:**
  - `crates/fret-diag-ws/src/server.rs`
  - `crates/fret-diag-ws/src/client.rs`
- **Runtime capture/export + script executor + inspector:**
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- **CLI frontend:**
  - `apps/fretboard/src/cli.rs`
- **DevTools GUI (early):**
  - `apps/fret-devtools/src/native.rs`
- **Layout deep debug escape hatch (Taffy dumps):**
  - `crates/fret-ui/src/tree/layout/taffy_debug.rs`
  - env wiring: `crates/fret-ui/src/runtime_config.rs`

## Target architecture (north star)

Think in four layers:

1. **Runtime capture layer (in-app)**
   - Collects snapshots (semantics + debug) and executes script steps.
   - Exposes a small “control plane” (inspect/pick/script/bundle/screenshot).
2. **Protocol layer (versioned schema)**
   - Stable JSON schema types for control-plane messages and artifacts.
   - Typed core invariants; forward-compatible unknown-field tolerance.
3. **Tooling engine layer (host-side)**
   - Materializes artifacts locally, performs lint/triage/compare/perf gates, packs bounded repro zips.
   - Transport-agnostic (filesystem vs WS is an implementation detail).
4. **Frontends**
   - `fretboard` CLI: thin mapping from args to engine operations.
   - DevTools GUI: UX on top of the same engine.
   - Optional: MCP server adapter mapping to the same engine (no new “secret” features).

The critical refactor principle:

> New features must land behind a narrow interface in layer (1) or (3), not by growing cross-cutting glue.

## Extensibility: ecosystem contributions (proposed)

### Problem

Today, ecosystem “extra diagnostics” are mostly added by:

- hardcoding new fields in the debug snapshot schema,
- or adding direct dependencies in the runtime exporter.

That scales poorly: it increases coupling and churn, and it makes it hard for ecosystem crates to ship their own
debugging stories.

### Proposed contract: “extensions” as structured JSON (bounded)

Add a single stable slot in the runtime snapshot output:

- `debug.extensions: map<string, ExtensionPayload>`

Where each entry is:

- namespaced key (examples: `fret.docking.interaction.v1`, `fret.ui_kit.virtual_list.v1`),
- payload is JSON, but with explicit `schema_version` inside each extension value,
- payloads must be bounded by size and clipped with a clip-report (same philosophy as other evidence).

This provides a clean “ecosystem seam” without exploding the typed schema.

### Runtime wiring options (incremental)

Option A (lowest churn, good first step): keep extension production inside `fret-bootstrap`

- Provide a `UiDiagnosticsExtensionsRegistry` in `fret-bootstrap` that stores a list of closures:
  - `fn(app, window, ui_tree) -> Option<(key, json_value)>`
- `UiDiagnosticsService` calls these after building the core snapshot.
- Ecosystem crates register extensions in their own init paths (no exporter dependencies required).

Option B (stronger layering, more work): move the registry to a generic runtime store

- Introduce a `WindowDiagnosticsExtensionsStore` (likely in `crates/fret-runtime`) keyed by `(window, frame_id, key)`.
- Ecosystem crates publish into the store; exporter just drains it.

We should start with Option A for speed, then graduate to Option B if it reduces coupling measurably.

### Tooling consumption

Tooling should treat extensions as optional:

- never hard-fail parsing unknown extensions,
- provide `fretboard diag query extension <key>` (eventually) and viewer rendering hooks,
- allow lints/gates to depend on extensions only when the script declares the capability.

## Layout debugging: correctness + performance (staged)

### What exists today

- Layout performance signal exists in debug snapshots:
  - `layout_engine_solves`, `layout_hotspots`, `widget_measure_hotspots` (runtime snapshot types live under
    `ecosystem/fret-bootstrap/src/ui_diagnostics/*`).
- Taffy deep dumps exist behind env flags and write JSON to disk.

### Gaps

- There is no first-class “layout artifact” tied to a repro run (it’s an external file dump).
- There is no single UX that connects:
  - “which semantics node/test_id is wrong” ↔ “which layout subtree is suspicious” ↔ “what changed between runs”.

### Proposed layout artifacts

1. **Portable layout sidecars (bundle-scoped)**
   - Allow a script step to request a Taffy dump for a selected subtree.
   - The dump is written as a **sidecar** in the bundle dir, e.g.:
     - `layout.taffy.<label>.json`
   - The sidecar includes minimal metadata:
     - window, scale factor, root bounds, selector used, frame id.

2. **Layout correctness gates (semantics-first)**
   - Prefer correctness assertions on semantics bounds keyed by `test_id`:
     - “bounds stable”, “bounds approx equal”, “inside window padding”, “min size”, etc.
   - Reserve the Taffy dump for “explain why”, not as the primary gate.

3. **Layout performance gates**
   - Promote stable thresholds keyed by suite/script tags:
     - p95 solve time, p95 measure time, top hotspots caps.

## Tooling refactor: make `crates/fret-diag` less monolithic

### Guiding rule

Keep “core engine” code small and stable; push specialized policy to plug-in seams.

### Proposed module boundaries (within the crate first)

Within `crates/fret-diag`, move toward:

- `transport/*` (FS + WS parity)
- `artifacts/*` (manifest, chunking, materialization, integrity)
- `bundle/*` (schema2 preference, indexing, meta extraction, streaming reads)
- `script/*` (push/run/repeat/suite orchestration)
- `gates/*` (perf gates, lint rules, reason codes)
- `compare/*` (diff + reports)
- `pack/*` (bounded zips, AI packets)
- `registry/*` (script registry, suite registry, check registry)

Only after boundaries hold should we consider splitting crates.

### De-monolithization target: “registries”

Introduce explicit registries so adding features does not require editing giant match statements:

- `SuiteRegistry`: suite name → list of scripts + prelude policy
- `CheckRegistry`: check name → check implementation (lint/perf/hotspots)
- `ScriptLibrary`: promoted scripts discovery + metadata indexing

Status (as of 2026-03-03):

- Initial scaffolding landed:
  - `crates/fret-diag/src/registry/suites.rs`
  - `crates/fret-diag/src/registry/checks.rs`
  - `crates/fret-diag/src/diag_list.rs` now resolves suites via `SuiteRegistry`.
  - `crates/fret-diag/src/diag_suite.rs` now resolves promoted + suite-dir scripts via `SuiteResolver`.
  - `crates/fret-diag/src/diag_suite.rs` centralizes builtin suite resolution + default env injection via `resolve_builtin_suite_scripts` (table-driven match).
  - `crates/fret-diag/src/post_run_checks.rs` begins routing post-run gates via `CheckRegistry` (starting with `gc_sweep_liveness`, `notify_hotspot_file_max`, `triage_hint_absent_codes`, and pixel gates).
  - `crates/fret-diag/src/registry/checks.rs` exposes `CheckRegistry::wants_post_run_checks` so orchestration can decide whether to run post-run checks without duplicating check-specific logic.
  - `crates/fret-diag/src/registry/checks.rs` exposes `CheckRegistry::wants_bundle_artifact` so orchestration can request a bounded bundle dump without hard-coding per-check conditions.
  - `crates/fret-diag/src/registry/checks.rs` exposes `CheckRegistry::wants_screenshots` so launch wiring can enable screenshots without hard-coding per-check conditions.
  - Artifacts boundary: `crates/fret-diag/src/artifact_store.rs` (`RunArtifactStore`) routes per-run artifact writes/materialization behind a focused API.
  - `CheckRegistry` now owns additional post-run gates (generic + some UI gallery gates) to reduce `post_run_checks.rs` churn.

## Plan (phased)

See:

- `docs/workstreams/diag-architecture-fearless-refactor-v1/TODO.md`
- `docs/workstreams/diag-architecture-fearless-refactor-v1/MILESTONES.md`

## Evidence & gates (baseline expectations)

For any landable refactor step:

- `cargo fmt`
- `cargo nextest run -p fret-diag` (and other touched crates)
- `python3 tools/check_layering.py` when crate boundaries move
- At least one regression artifact when behavior changes:
  - unit/integration test and/or `tools/diag-scripts/*.json` + `fretboard diag run/suite`

## Open questions

- Where should the long-term “extensions registry” live: `fret-bootstrap` (ecosystem) or `fret-runtime` (core)?
- Do we need a typed “layout snapshot protocol” (beyond sidecars) for DevTools live inspection?
- What is the minimal layout “explainability” payload that is worth capturing without perf cliffs?
