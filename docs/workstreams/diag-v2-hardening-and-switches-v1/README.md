---
title: Diag v2 Hardening + Switches Refactor v1
status: draft
date: 2026-02-26
scope: diagnostics, automation, artifacts, config, fearless-refactor
---

# Diag v2 Hardening + Switches Refactor v1

This workstream finishes the “v2” transition and simplifies diagnostics configuration (“switches”) so the system stays:

- **portable** (native + web via DevTools WS),
- **deterministic** (capability-gated, stable `reason_code` + bounded evidence),
- **small-by-default** (AI packets, indexes, slices; no “open a 200MB `bundle.json`”),
- **fearless-refactor friendly** (clear seams; legacy paths boxed behind shims).

Related / prerequisites:

- Living usage: `docs/ui-diagnostics-and-scripted-tests.md`
- Simplification tracker: `docs/workstreams/diag-simplification-v1.md`
- Capabilities + evidence: `docs/workstreams/diag-extensibility-and-capabilities-v1/README.md`
- AI packet + indexing: `docs/workstreams/diag-ai-agent-debugging-v1.md`
- Bundle schema v2 tracker: `docs/workstreams/diag-bundle-schema-v2.md`
- Contracts: `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`, `docs/adr/0189-ui-diagnostics-extensibility-and-capabilities-v1.md`
- Script library modularization (taxonomy + suites): `docs/workstreams/diag-v2-hardening-and-switches-v1/script-library.md`
- Canonical per-run artifact layout: `docs/workstreams/diag-v2-hardening-and-switches-v1/per-run-layout.md`
- Migration runbook + guardrails: `docs/workstreams/diag-v2-hardening-and-switches-v1/migration-support.md`
- Compatibility inventory table: `docs/workstreams/diag-v2-hardening-and-switches-v1/compat-matrix.md`

## Problem statement

Today the “v2 direction” is real, but not complete:

1. **Multiple “v2s” exist** (script schema v2, bundle schema v2, artifact v2-ish layout, sidecars, AI packets).
2. **Compatibility logic is spread** (filesystem vs WS differences, schema v1/v2 bundle reading, step fallbacks, legacy env vars, capability aliases).
3. **Switches are too many and too leaky**:
   - config is split across env vars, flags, and config files,
   - some toggles overlap (e.g. dump screenshots vs scripted screenshots),
   - reserved env vars in `--launch` mode are easy to misuse.

This is not “wrong engineering”; it is expected during a rapid capability build-out. The risk now is that the transitional
paths become permanent, and the system’s complexity grows faster than its debugging value.

## Findings (in-tree gaps to close for “v2 done”)

These are concrete gaps observed in the current implementation that keep v2 from being “done” (compat paths still
required, inconsistent semantics, or transport divergence). Each item includes evidence anchors.

1) Runtime still accepts and upgrades script schema v1

- Why it matters: “v2 is default” is not the same as “v2 is the only execution path”. Keeping v1 parsing in the runtime
  increases drift/complexity and makes compat removal harder.
- Evidence:
  - runtime reads `UiActionScriptV1` (filesystem trigger): `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`
  - v1→v2 upgrade path: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_types.rs`

2) Window targeting is inconsistent across v2 steps

- Why it matters: multi-window is a core Fret goal; scripts should not silently lose correctness when crossing windows.
- Status (2026-02-27): **mostly closed for selector-driven steps**. The script schema now supports optional `window`
  targeting across the common selector-driven steps (including “stable” click/scroll flows), and tooling can infer
  `diag.multi_window` when the target is an “other window”.
- Evidence:
  - Step schema now carries `window` for stable click + scroll + pointer steps:
    `crates/fret-diag-protocol/src/lib.rs`
  - Tooling infers `diag.multi_window` when `window` targets require it:
    `crates/fret-diag/src/script_tooling.rs`, `crates/fret-diag/src/lib.rs`
  - Runtime routes the `window` target consistently for selector-driven steps:
    `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`

3) Filesystem vs DevTools WS divergence: dump request metadata (labels) is lost in FS mode

- Why it matters: transport divergence forces tooling to special-case behavior. For dumps, WS supports labels and request
  correlation; filesystem dump is currently just a `touch`, dropping metadata.
- Status (2026-02-27): **closed**. Filesystem transport now supports a structured `dump.request.json` carrying dump
  metadata; runtime consumes it for trigger-driven dumps.
- Evidence:
  - tooling writes `dump.request.json` + trigger touch: `crates/fret-diag/src/transport/fs.rs`
  - runtime consumes the request: `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`

4) Capabilities schema is minimal; runner identity is not surfaced

- Why it matters: capabilities are a key contract surface; we benefit from optional `runner_kind` / `runner_version` /
  `protocol_versions` fields for auditability and easier triage.
- Status (2026-02-27): **closed** (additive). `FilesystemCapabilitiesV1` now carries optional identity hints, and the
  runtime emits them when available.
- Evidence:
  - protocol schema: `crates/fret-diag-protocol/src/lib.rs`
  - runtime emission: `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`

5) Artifact v2 (manifest + chunks) is not yet the single source of truth

- Why it matters: as long as `bundle.json` is the “real artifact”, compat cannot be retired and web/WS transport remains
  constrained. The canonical artifact should be manifest/chunks with `bundle.json` as an optional compatibility view.
- Evidence:
  - chunking/materialization direction: `docs/workstreams/diag-simplification-v1.md`
  - schema2 writing is still opt-in via config/env: `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump.rs`,
    `docs/ui-diagnostics-and-scripted-tests.md`

6) Script library layout is flat; discoverability and ownership do not scale

- Why it matters: as scripts accumulate, a single `tools/diag-scripts/` folder becomes hard to navigate, review, and maintain.
- Status (2026-02-27): **mostly closed for “flat root”**. Canonical scripts are moved into a taxonomy with redirect
  stubs, and CI checks prevent new canonical scripts from landing back in `tools/diag-scripts/*.json`.
  A minimal, generated registry exists at `tools/diag-scripts/index.json` (scope: scripts reachable from in-tree suites
  + `_prelude`) and is validated in CI.
- Evidence:
  - built-in suites are curated directory inputs via redirect stubs: `tools/diag-scripts/suites/` and
    `crates/fret-diag/src/diag_suite_scripts.rs`
  - `diag suite` no longer hard-codes script file lists; specialized harness suites are also directory-driven via
    `tools/diag-scripts/suites/<suite-name>/`: `crates/fret-diag/src/diag_suite.rs`

7) `diag perf <suite-name>` suite expansion drift risk (duplicate lists)

- Why it matters: perf suite expansion and perf baseline seed policy both depend on “what scripts are in the suite”.
  Duplicating suite lists creates silent inconsistencies (different scripts, different ordering, different keys).
- Status (2026-02-27): **closed**. `diag perf` suite expansion and perf baseline seed scoping are both derived from the
  **promoted script registry** (`tools/diag-scripts/index.json`) via `suite_memberships`.
  - Perf suites are expressed as curated `script_redirect` stubs under `tools/diag-scripts/suites/perf-*/`.
  - Tooling resolves redirects and generates the promoted registry; Rust-side suite name expansion reads the registry and
    selects scripts by suite membership, with deterministic ordering.
- Evidence:
  - perf entrypoint: `crates/fret-diag/src/diag_perf.rs`
  - suite membership resolver: `crates/fret-diag/src/perf_seed_policy.rs`
  - promoted registry: `tools/diag-scripts/index.json`, `tools/check_diag_scripts_registry.py`
  - suites: `tools/diag-scripts/suites/perf-*/`

8) Pointer kind (“mouse/touch/pen”) is supported, but needs a single canonical doc section

- Why it matters: for automation/debugging, “works with mouse” is not equivalent to “works with touch” (hover, capture,
  gesture recognizers, focus semantics, scroll/wheel behavior). Scripts should be able to express intent, and tooling
  should capability-gate + surface evidence of the effective pointer type.
- Status (2026-02-27): **supported end-to-end**. Script steps can carry optional `pointer_kind`, tooling infers required
  capabilities, runtime synthesizes events with the right `pointer_type`, and bundles surface a `primary_pointer_type`.
- Evidence:
  - protocol: `crates/fret-diag-protocol/src/lib.rs` (`UiPointerKindV1` + `pointer_kind` fields on steps)
  - capability inference: `crates/fret-diag/src/script_tooling.rs`
  - runtime event synthesis + labels: `ecosystem/fret-bootstrap/src/ui_diagnostics/input_event_synthesis.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics/labels.rs`
  - runtime script execution: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer.rs`

## Goals

### G1: Define “Done for v2”

Lock a concrete definition of “v2 is complete” for the diag stack:

- Scripts: schema v2 is the default authoring + execution path, with strong intent-level steps.
- Artifacts: transport-neutral artifact layout has a **typed manifest** and supports chunked payloads; `bundle.json` is a compatibility view.
- Tooling: gates, packing, triage, query, slice work on the manifest/chunks fast-path when available, falling back to `bundle.json` when necessary.
- Switches: a single canonical config surface exists; env vars and CLI flags are thin overrides (not competing sources of truth).

### G2: Consolidate switches (config + overrides)

Make configuration predictable:

- A **single canonical config file** is the primary interface (`FRET_DIAG_CONFIG_PATH`).
- Env vars remain supported but are explicitly treated as overrides and are minimally scoped.
- Tooling writes per-run configs deterministically when it launches the app; “manual mode” remains possible.
  - Status (2026-02-28): `diag run/suite/repro/perf --launch` all funnel through a single helper that writes
    `<out_dir>/diag.config.json`, sets `FRET_DIAG_CONFIG_PATH` for the child, and treats config write failure as a hard
    launch error (no silent fallback to large `bundle.json` defaults). Tool-launched runs also scrub inherited
    `FRET_DIAG_*` env vars from the parent shell (pass explicit `diag --env KEY=VALUE` when you truly need a one-off
    override).
  - Audit (2026-02-28): all `--launch` entry points (`diag run/suite/repro/perf/repeat/script`) call the same helper
    (`crates/fret-diag/src/compare.rs:maybe_launch_demo`) to ensure consistent per-run config + env policy.
  - Smoke (2026-02-28): `ui-gallery-gesture-tap-smoke` passes under `--launch` and produces schema2-only bundle exports
    by default (`bundle.schema2.json` present, raw `bundle.json` absent).
  - Smoke (2026-02-28): `ui-gallery-table-smoke` passes again after relaxing a too-strict
    `bounds_within_window` check to `visible_in_window` (some page roots can be taller than the window).
  - Smoke (2026-02-28): `ui-gallery-empty-background-gradient-screenshot` passes under `--launch` and writes a PNG under
    `target/fret-diag/screenshots/<bundle_dir>/` (tool-launched per-run config sets `screenshots_enabled=true` as needed).
  - Convenience (2026-02-28): a tiny smoke suite exists at `tools/diag-scripts/suites/diag-hardening-smoke/` for quick
    post-merge verification:
    - `cargo run -p fretboard -- diag suite diag-hardening-smoke --launch -- cargo run -p fret-ui-gallery --release`

### G3: Box compatibility logic behind seams

Compatibility must be explicit and removable:

- isolate “legacy bundle/script readers” behind `compat/` modules,
- isolate filesystem vs WS differences behind `transport/` and `artifact_store/`,
- ensure failures always produce a local `script.result.json` with stable `reason_code` (tooling-side too).

## Compatibility inventory (snapshot)

This section is a “what still exists” checklist. It is intentionally explicit so we can remove compat paths without
accidentally breaking day-to-day debugging.

### Capabilities

- **Legacy capability aliases (tooling-side):** tooling maps un-namespaced runner capabilities (e.g. `script_v2`) to
  namespaced `diag.*` (see `crates/fret-diag/src/compat/mod.rs`).
- **Legacy control-plane capabilities (DevTools WS):** WS session descriptors may include un-namespaced control-plane
  caps (`inspect`, `pick`, `scripts`, `bundles`) alongside namespaced `devtools.*` (see
  `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`).

Exit plan:

1) Stop advertising un-namespaced control-plane caps once downstream tooling no longer relies on them.
2) Remove `compat::normalize_capability` mappings once all runners advertise only `diag.*`.

### Script schema v1

- **Tooling auto-upgrade (manual-only):** tooling can upgrade `schema_version=1` scripts to schema v2 on execution
  (`crates/fret-diag/src/compat/script.rs`). This keeps old scripts runnable when iterating manually.
- **Tool-launched runs are v2-only:** when using `--launch` / `--reuse-launch`, tooling rejects schema v1 scripts
  (requires an explicit `diag script upgrade --write` migration).
- **Runtime gating:** runtime can reject schema v1 scripts when `allow_script_schema_v1=false` (tooling writes this
  explicitly for `--launch` runs via the config file).

Exit plan (fearless):

1) Ensure in-repo committed scripts are schema v2 only (except `script_redirect` stubs).
2) Add a CI gate that fails if canonical scripts regress to schema v1.
3) Keep manual upgrade available (`diag script normalize`) but stop auto-upgrading for tool-launched runs.

### Bundle schema + artifact views

- **Schema sniffing:** tooling sniffs bundle schema versions from a bounded JSON prefix (to avoid loading large JSON)
  and records compat markers (see `crates/fret-diag/src/compat/bundle.rs`, `crates/fret-diag/src/triage_json.rs`).
- **Legacy views:** per-run directories may include `bundle.json` and/or `bundle.schema2.json` as compatibility views,
  with sidecars as accelerators (see `docs/workstreams/diag-v2-hardening-and-switches-v1/per-run-layout.md`).

Exit plan:

1) Prefer manifest + sidecars for all tooling flows.
2) Stop materializing/writing `bundle.json` unless explicitly requested (`diag pack` / share flows should not require it).
   - Tool-launched escape hatch: `--launch-write-bundle-json` (requires `--launch`; not supported for `diag matrix`).

## Non-goals

- Breaking `crates/fret-diag-protocol` or changing ADR-owned meaning of fields.
- Removing the ability to run old scripts immediately (we will stage deprecations).
- Introducing policy into mechanism crates (`crates/fret-ui`, `crates/fret-core`).

## Proposed direction

### 1) One artifact model: “run directory” + manifest (canonical), `bundle.json` (compat)

Make the per-run directory layout canonical across transports:

- `<out_dir>/<run_id>/manifest.json` (canonical, typed)
- `<out_dir>/<run_id>/script.json` (canonical)
- `<out_dir>/<run_id>/script.result.json` (canonical)
- `<out_dir>/<run_id>/artifacts/*` (optional, chunked)
- `<out_dir>/<run_id>/bundle.json` and/or `<out_dir>/<run_id>/bundle.schema2.json` (compat views, optional)
- sidecars (`bundle.meta.json`, `bundle.index.json`, `test_ids.index.json`, `frames.index.json`) remain optional accelerators

Rules:

- Tooling MUST be able to produce a useful “AI share packet” without materializing `bundle.json`.
- When `bundle.json` exists, it must be derived from manifest/chunks, not the other way around.

### 2) Switches refactor: config is data, overrides are layered

Define a single config object (schema v1) that covers:

- paths (trigger/script/screenshot/pick/inspect),
- ring buffer sizing,
- semantics capture mode + budgets,
- evidence budgets / redaction,
- screenshot policies (bundle-scoped BMP vs on-demand PNG),
- determinism knobs (fixed frame delta, warmup frames),
- devtools embed/chunk options.

Resolution order (highest precedence first):

1. CLI explicit flags (tooling-only).
2. Env var overrides (minimal, documented).
3. `FRET_DIAG_CONFIG_PATH` JSON (canonical).
4. Tooling defaults (when launching; writes the config file).
5. Runtime defaults (safe fallback).

The runtime should treat the config file as the primary contract; env vars are compatibility shims.

### 3) Capability gating becomes the only way to express optional features

Switches must not be a backdoor for missing capabilities.

Examples:

- If `capture_screenshot` exists in the script, tooling infers `diag.screenshot_png` and fails fast when missing.
- If a script uses window targeting, tooling infers `diag.multi_window`.

All “optional runner behavior” must have a capability name, and scripts should declare
`meta.required_capabilities` when they are intentionally narrow.

### 4) Deprecation strategy: stop *writing* legacy first, keep *reading* longer

Fearless refactor rule: do not break existing repos/scripts overnight.

Stage deprecations:

- Stage A: stop generating legacy artifacts by default; keep generation behind explicit flags.
- Stage B: keep reading legacy in tooling, but mark legacy usage in `triage.json` and `ai.packet.json`.
- Stage C: remove legacy reading paths once repo scripts and CI are migrated (separately tracked).

Note: treat “runtime accepts legacy inputs” similarly:

- Stage A: tooling normalizes and pushes v2 scripts by default; runtime still accepts v1.
- Stage B: runtime v1 parsing becomes opt-in (config/feature) and is disabled by default for tool-launched runs.
- Stage C: remove runtime v1 parsing once migration is complete.

### 5) UX tightening (without big new UI)

Reduce “you need to know too many commands” by defining a small set of golden flows:

- “Run 1 script and share”: `diag repro` profile that always produces `repro.ai.zip`.
- “Triage 1 failure”: `diag triage` that prefers manifest/chunks and prints the failure anchor summary.
- “Find selector quickly”: `diag query test-id` against `test_ids.index.json` / packet, not `bundle.json`.

### 6) Script library modularization (paths + migration)

Treat script paths as part of the developer UX:

- introduce a small folder taxonomy (example below),
- add a script registry (index file) so suites can be defined by tags rather than “magic filenames”,
- provide a one-shot migration tool that:
  - moves scripts into the new folders,
  - updates any hard-coded references (e.g. suite lists),
  - optionally normalizes JSON (`diag script normalize --write`) to keep diffs stable.

Goal: allow scripts to grow without turning `tools/diag-scripts/` into an unmaintainable dumping ground.

#### Migration blast radius (why this needs a plan)

Today, script paths appear in many places beyond the folder itself:

- Tooling hard-codes script paths for some suites and perf helpers:
  - built-in suites are directory inputs (membership lives in `tools/diag-scripts/suites/`): `crates/fret-diag/src/diag_suite_scripts.rs`
  - `crates/fret-diag/src/diag_suite.rs`
  - `crates/fret-diag/src/diag_perf.rs`
- Docs and ADR evidence anchors reference script paths (many files under `docs/`).
- Some helper scripts and examples also reference workspace script paths:
  - `apps/fretboard/src/cli.rs` (help examples)
  - `apps/fret-devtools/README.md` (devtools UX)

This means “move files on disk” is a cross-repo refactor unless we add a compatibility layer for old paths.

#### Proposed taxonomy (example)

Keep the top-level buckets small and stable. Prefer “product area” then “intent”.

```
tools/diag-scripts/
  _prelude/
  tooling/
  ui-gallery/
    overlay/
    layout/
    text-ime/
    text-wrap/
    code-editor/
    markdown-editor/
    combobox/
    select/
    shadcn-conformance/
    perf/
  docking/
    arbitration/
    motion-pilot/
  web/
```

Notes:

- Put shared reset scripts under `_prelude/` and drive them via `--suite-prelude`.
- Avoid “misc/”: if a script does not fit, the taxonomy is missing a bucket.
- Prefer stable IDs over filenames for long-lived suites; filenames remain human-friendly.
- `diag suite` already supports directory inputs and expands `**/*.json` with deterministic ordering (sorted set),
  which makes folder-based suites a viable intermediate step before a full registry.

#### Registry shape (v1, generated)

`tools/diag-scripts/index.json` is generated (do not edit by hand) via:

- `python tools/check_diag_scripts_registry.py --write`

Design constraints (v1):

- Scope is intentionally small: scripts reachable from in-tree suites plus `_prelude/*` (not the entire library).
- IDs are path-independent by default (derived from file stem). If needed, a script may provide `meta.id` to override the
  default and avoid collisions while still allowing fearless path moves.

Fields:

- `schema_version`
- `scripts[]`:
  - `id` (stable, path-independent): `meta.id` or file stem
  - `path` (repo-relative)
  - `tags[]` (small): e.g. `smoke`, `overlay`, `ime`, `perf`
  - `suite_memberships[]` (optional): e.g. `ui-gallery`, `ui-gallery-text-ime`, `_prelude`
  - `required_capabilities[]` (optional; mirrors `meta.required_capabilities`)
  - `target_hints[]` (optional; mirrors script meta)

Tooling can then support:

- built-in suites that resolve to registry queries (by `suites[]` or `tags[]`),
- ad-hoc runs via `diag suite --glob` / `--script-dir` against the folder layout,
- migration that only needs to update `index.json`, not many Rust lists.

This is intentionally not a “new DSL”: scripts remain JSON; the registry is only discovery and suite membership.

Decision (recommended): make named suites registry-driven; keep `--glob` / `--script-dir` for ad-hoc runs and local experimentation.

#### Compatibility strategy for path moves (recommended)

To keep the refactor fearless and avoid updating dozens (or hundreds) of doc anchors in one PR, prefer a two-stage approach:

Stage 1: Registry-first (no file moves)

- Add `tools/diag-scripts/index.json` and make named suites resolve scripts via the registry.
- Keep existing file layout temporarily, but make discoverability scale immediately (tags, suites, ownership).

Stage 2: Move scripts into subfolders (with path compatibility)

Option A (preferred): keep legacy paths as redirects (tooling-resolved)

- Move the real scripts into the new folder layout.
- Leave small JSON stubs at the old locations that redirect to the new paths.
- Tooling resolves redirects when reading workspace scripts (for `diag run`, built-in suites, and any helper that reads
  `tools/diag-scripts/*.json`).

Redirect stub shape (tooling-only; runtime never sees it directly):

```json
{
  "schema_version": 1,
  "kind": "script_redirect",
  "to": "tools/diag-scripts/ui-gallery/overlay/overlay-torture.json"
}
```

Rules:

- Redirect resolution MUST be loop-safe (depth cap + visited set).
- Tooling SHOULD normalize the final resolved script JSON before pushing/writing.
- Script tooling (`diag script validate|lint|normalize`) SHOULD resolve redirect stubs before operating, and `--write` SHOULD
  update the resolved target script (not the stub).

Option B: “big bang” path updates

- Move scripts and update every reference in code + docs in the same PR.
- Not recommended unless combined with a scripted rewrite and strong review discipline.

#### Migration runbook (draft)

This runbook is designed to keep the refactor reviewable and reversible.

1) Generate a plan (dry-run)

```powershell
python tools/diag-scripts/migrate-script-library.py --plan-out .fret/diag-script-library-migration.plan.json
```

2) Apply moves with legacy redirects (preferred when docs/ADRs contain many path anchors)

```powershell
python tools/diag-scripts/migrate-script-library.py --apply --write-redirects --plan-out .fret/diag-script-library-migration.plan.json
```

3) Validate scripts and suites

- Validate scripts: `cargo run -p fretboard -- diag script validate tools/diag-scripts`
- Normalize scripts (optional, to stabilize diffs): `cargo run -p fretboard -- diag script normalize tools/diag-scripts --write`
- Run suites using directory inputs as a transition step:
  - `cargo run -p fretboard -- diag suite --script-dir tools/diag-scripts/ui-gallery`

4) Optional: rewrite references (not recommended if redirects are in use)

```powershell
python tools/diag-scripts/migrate-script-library.py --apply --rewrite-references code
```

Notes:

- Prefer “registry-first (no moves)” if you want to decouple suite membership from filenames before touching paths.
- If redirects are used, treat them as temporary and track removal under P3 debt removal.

## Definition of done (v2 completion checklist)

We consider this workstream complete when:

1. Every `diag run/suite/repro/perf` produces a per-run directory with a manifest (both filesystem and WS).
2. `diag pack --ai-only` can succeed from manifest + sidecars without any bundle monolith present.
3. Runtime config uses `FRET_DIAG_CONFIG_PATH` as the primary entry point; env var overrides are minimal and documented.
4. Legacy writers are off by default, behind explicit flags.
5. Compatibility logic is isolated and has a tracked removal plan (linked from `todo.md`).

## Implementation priority (recommended)

P0 (high ROI, low risk):

- Switch resolution + documentation: make config layering unambiguous; ensure `diag.config.example.json` matches reality.
- Make manifest presence universal for tool-launched runs; ensure failure modes still emit `script.result.json`.
- Start boxing legacy script support: concentrate v1 parsing/upgrading behind an explicit compat seam.

P1 (artifact core):

- Promote manifest/chunks as the canonical artifact store (writer + reader fast paths).
- Make AI packet generation and packing prefer manifest/chunks.
- Close multi-window gaps in schema v2 steps (consistent window targeting for selector-driven steps).
- Add an FS dump request surface that can carry dump metadata (label, max snapshots, request id), matching WS.

P2 (compat boxing):

- Move legacy readers/writers behind `compat/` modules and add lint warnings for legacy usage.
- Move legacy env var parsing behind a compat layer once `FRET_DIAG_CONFIG_PATH` is canonical.

P3 (debt removal):

- Remove unused/duplicated env vars and flags after migration.

See:

- TODO tracker: `docs/workstreams/diag-v2-hardening-and-switches-v1/todo.md`
- Milestones: `docs/workstreams/diag-v2-hardening-and-switches-v1/milestones.md`
