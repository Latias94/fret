---
title: Diag v2 Hardening + Switches Refactor v1 - TODO
status: draft
date: 2026-02-26
scope: diagnostics, automation, artifacts, config, refactor
---

# TODO

This file is a check-list style tracker. Milestone framing lives in `milestones.md`.

## P0: Switches consolidation (config layering)

- [x] Document config resolution order (CLI vs env vs config file vs defaults) and link it from `docs/ui-diagnostics-and-scripted-tests.md`.
- [x] Audit `tools/diag-configs/diag.config.example.json` for drift; ensure every field is either:
  - implemented, or
  - explicitly documented as “planned / ignored by runtime”.
- [x] Make tooling push schema v2 scripts by default (normalize/upgrade on write) and warn when schema v1 scripts are observed.
- [x] Add a runtime compat switch for schema v1 scripts:
  - [x] enable/disable v1 script parsing explicitly (default: enabled for manual, disabled for tool-launched runs),
  - [x] record legacy usage in `script.result.json` evidence (so triage can detect compat paths).
- [x] Ensure `diag config doctor` validates the example config (no unknown keys).
- [x] Define and document a minimal env var set (the rest become deprecated aliases):
  - [x] `FRET_DIAG`
  - [x] `FRET_DIAG_CONFIG_PATH`
  - [x] `FRET_DIAG_GPU_SCREENSHOTS`
  - [x] `FRET_DIAG_REDACT_TEXT`
  - [x] `FRET_DIAG_FIXED_FRAME_DELTA_MS`
  - [x] Document deprecated aliases + removal plan (P2/P3): `docs/workstreams/diag-v2-hardening-and-switches-v1/deprecations.md`.
- [x] Define “reserved env vars” policy for `--launch` (tooling-owned) and enforce it uniformly.
- [x] Add a `diag config doctor` (tooling-side) that prints an effective merged config + highlights deprecated keys/envs.

## P0.5: Script library modularization (UX scalability)

- [x] Define a folder taxonomy for `tools/diag-scripts/` (by product area + suite intent). See: `docs/workstreams/diag-v2-hardening-and-switches-v1/script-library.md`.
- [x] Decide whether suites should be:
  - [ ] registry-driven (preferred), or
  - [x] glob-driven (acceptable for small sets, but brittle long-term). (v1 decision: curated suite directories + redirect stubs)
- [x] As an intermediate step, switch built-in suites from hard-coded file lists to directory inputs (deterministic `**/*.json` expansion).
- [x] Ensure capability inference resolves `script_redirect` stubs (screenshots / required caps / env defaults).
- [ ] Add a script registry file (draft):
  - [ ] file: `tools/diag-scripts/index.json` (or `index.toml`)
  - [ ] fields: `id`, `path`, `tags`, `target_hints`, `required_capabilities`, `suite_memberships`
- [ ] Prefer `--suite-prelude` for shared resets (`tools/diag-scripts/_prelude/*`) and document the convention.
- [x] Document a migration runbook (dry-run plan → apply moves → validate suites) and link it from `docs/ui-diagnostics-and-scripted-tests.md`.
- [ ] Decide path-move compatibility strategy:
  - [ ] registry-first (no moves) to decouple suites from filenames,
  - [ ] then move to folders with either redirects (preferred) or “big bang” rewrites.
- [x] If using redirects, implement tooling redirect resolution:
  - [x] add `script_redirect` stub support with loop detection,
  - [x] ensure redirects never reach the runtime (tooling resolves before push).
- [x] Add a migration helper script (plan + apply moves + optional redirects/rewrite).
- [ ] Execute incremental taxonomy migrations (small batches + redirects + closure checks):
  - [x] `ui-gallery/select` (17 scripts)
  - [x] `ui-gallery/combobox` (22 scripts)
  - [x] `ui-gallery/text-ime` (2 scripts)
  - [x] `ui-gallery/text-wrap` (5 scripts)
  - [x] `ui-gallery/text` (5 scripts)
  - [x] `ui-gallery/shadcn-conformance` (7 scripts)
  - [x] `ui-gallery/overlay` (40 scripts; batch-migrated)
  - [x] `ui-gallery/code-editor` (42 scripts; batch-migrated)
  - [x] `ui-gallery/markdown-editor` (24 scripts)
  - [x] `ui-gallery/layout` (4 scripts)
  - [x] `ui-gallery/perf` (70 scripts; batch-migrated)
  - [x] `ui-gallery/date-picker` (5 scripts)
  - [x] `ui-gallery/material3` (37 scripts)
  - [x] `ui-gallery/ai` (67 scripts; batch-migrated)
  - [x] `ui-gallery/menubar` (18 scripts)
  - [x] `ui-gallery/command` (14 scripts)
  - [x] `ui-gallery/data-table` (16 scripts)
  - [x] `ui-gallery/context-menu` (9 scripts)
  - [x] `ui-gallery/dropdown-menu` (5 scripts)
  - [x] `ui-gallery/button` (11 scripts)
  - [x] `ui-gallery/checkbox` (9 scripts)
  - [x] `ui-gallery/sidebar` (8 scripts)
  - [x] `ui-gallery/drawer` (6 scripts)
  - [x] `ui-gallery/sonner` (7 scripts)
  - [x] `ui-gallery/table` (9 scripts)
  - [x] `ui-gallery/code-view` (4 scripts)
  - [x] `ui-gallery/control-chrome` (4 scripts)
  - [x] `ui-gallery/collapsible` (7 scripts)
  - [x] `ui-gallery/dropdown` (6 scripts)
  - [x] `ui-gallery/navigation` (10 scripts)
  - [x] `ui-gallery/carousel` (5 scripts)
  - [x] `ui-gallery/toggle` (5 scripts)
  - [x] `ui-gallery/theme` (4 scripts)
  - [x] `ui-gallery/typography` (4 scripts)
  - [x] `ui-gallery/virtual-list` (5 scripts)
  - [x] `ui-gallery/input` (4 scripts)
  - [x] `ui-gallery/misc` (redirect-only; 0 canonical scripts)
  - [ ] Replace misc redirects with direct suite references (optional; reduce redirect chain depth)
  - [x] `docking/arbitration` (33 scripts)
  - [x] `tooling/external-texture-imports` (9 scripts)
  - [x] `tooling/todo` (4 scripts)
  - [x] `_prelude/*` (2 scripts)
- [x] Update references after path moves (chosen approach):
  - [x] replace hard-coded lists with registry/directory inputs, or
  - [ ] scripted rewrite of code/docs references (large diff; less preferred).
- [ ] (If needed) write a migration script to move scripts into subfolders and update references:
  - [ ] updates `crates/fret-diag/src/diag_suite_scripts.rs` (or replaces it with a registry reader),
  - [ ] updates any other hard-coded references under `crates/fret-diag/src` (search for `tools/diag-scripts/`),
  - [ ] updates any docs that reference old paths,
  - [ ] optional: runs `fretboard diag script normalize --write` on moved scripts.

## P1: Manifest-first artifacts (transport-neutral)

- [ ] Write a single “canonical per-run layout” doc (point to `docs/workstreams/diag-simplification-v1.md` and reconcile terminology).
- [ ] Ensure filesystem transport always produces a per-run manifest (even in manual `poke` workflows).
- [ ] Ensure DevTools WS transport always materializes a per-run manifest alongside `script.result.json`.
- [ ] Add an FS dump request surface to carry dump metadata (label/max snapshots/request id), matching WS:
  - [ ] tooling writes `dump.request.json` (or equivalent) and touches a trigger,
  - [ ] runtime consumes it and includes metadata in `bundle.dumped` event logs.
- [ ] Make `diag pack --ai-only` succeed from manifest + sidecars without `bundle.json`.
- [ ] Add `diag artifact lint` that validates:
  - [ ] manifest schema,
  - [ ] chunk list hashes (when present),
  - [ ] sidecar schema versions,
  - [ ] consistent run id + timestamps.

## P2: Box compatibility logic

- [ ] Create `compat/` modules in tooling for:
  - [ ] legacy capability aliases (`script_v2` → `diag.script_v2`, etc),
  - [ ] v1 bundle schema reading,
  - [ ] v1 script schema reading (if still supported).
- [ ] Close multi-window gaps in schema v2 steps:
  - [ ] add optional `window` targeting to selector-driven steps that currently lack it (e.g. `click_stable`, `wheel`, pointer moves),
  - [ ] update capability inference (keep using `diag.multi_window`) and ensure fail-fast gating.
- [ ] Extend `FilesystemCapabilitiesV1` with optional identity fields (additive):
  - [ ] `runner_kind`, `runner_version`,
  - [ ] optional `protocol_versions`/`schemas` hints for tooling.
- [ ] Create `transport/` seam contract and ensure all FS vs WS differences are isolated there.
- [ ] Add a “legacy usage” marker into `triage.json` / `ai.packet.json` when compat fallbacks were used.

## P3: Deprecations + debt removal

- [ ] Turn off legacy writers by default:
  - [ ] avoid writing `bundle.json` unless requested; prefer schema2/manifest.
- [ ] Deprecate/remove flags that are now represented as config fields or capabilities.
- [ ] Delete unused env var parsing paths once CI/scripts migrate (tracked by a migration checklist).

## Migration support (fearless refactor safety)

- [ ] Provide a script migration guide:
  - [ ] `diag pick-apply` workflows,
  - [ ] `diag script normalize --check` in CI,
  - [ ] “upgrade script v1 → v2” helper (if any v1 scripts remain).
- [ ] Add a “compat matrix” table for:
  - [ ] bundle schema variants,
  - [ ] script schema variants,
  - [ ] transports,
  - [ ] required capabilities.
