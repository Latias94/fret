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

- Why it matters: as scripts accumulate, a single `tools/diag-scripts/` folder becomes hard to navigate, review, and
- Status (2026-02-27): **in progress**. A taxonomy + redirect strategy exists, but we still need enforcement to prevent
  new scripts from landing back in the root and to reduce suite brittleness long-term (registry).
- Evidence:
  - built-in suites are curated directory inputs via redirect stubs: `tools/diag-scripts/suites/` and
    `crates/fret-diag/src/diag_suite_scripts.rs`
  - some suites/helpers still hard-code individual script paths: `crates/fret-diag/src/diag_suite.rs`

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

### G3: Box compatibility logic behind seams

Compatibility must be explicit and removable:

- isolate “legacy bundle/script readers” behind `compat/` modules,
- isolate filesystem vs WS differences behind `transport/` and `artifact_store/`,
- ensure failures always produce a local `script.result.json` with stable `reason_code` (tooling-side too).

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

#### Registry shape (draft)

If we add `tools/diag-scripts/index.json`, keep it minimal and additive:

- `schema_version`
- `scripts[]`:
  - `id` (stable, dotted): e.g. `ui_gallery.dialog.escape_focus_restore`
  - `path` (repo-relative)
  - `tags[]` (small): e.g. `smoke`, `overlay`, `ime`, `perf`
  - `suites[]` (optional): e.g. `ui-gallery`, `ui-gallery-text-ime`
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
