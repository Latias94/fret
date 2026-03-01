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
- [x] Make tooling push schema v2 scripts by default (normalize on write) and warn when schema v1 scripts are observed.
- [x] Tool-launched runs (`--launch` / `--reuse-launch`) reject schema v1 scripts (require an explicit `diag script upgrade --write` migration).
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
- [x] Scrub all inherited `FRET_DIAG_*` env vars in `--launch` mode (prefix-based) so parent-shell overrides cannot
  silently drift tool-launched runs.
- [x] Audit `--launch` entry points to ensure a single per-run config writer is used (`diag run/suite/repro/perf/repeat`
  funnel through `maybe_launch_demo`; evidence: `crates/fret-diag/src/compare.rs:maybe_launch_demo`).
- [x] Tool-launched output safety defaults:
  - [x] `script_auto_dump=false` (avoid "dump on every injected step" explosions)
  - [x] `pick_auto_dump=false` (avoid "dump on every pick" explosions)
- [x] Add a `diag config doctor` (tooling-side) that prints an effective merged config + highlights deprecated keys/envs.
- [x] Eliminate docking multi-window lint false negatives by ensuring focus repair runs before semantics refresh (and on
  layout fast-path frames), so bundles never capture a focused node with empty bounds.
- [x] Add an opt-in pointer input isolation knob for tool-launched scripted runs so accidental real mouse movement/clicks
  do not perturb deterministic playback (especially for cross-window docking/tear-off).

## P0.6: Concurrency hygiene (multiple agents)

- [x] Document that `--dir` / `FRET_DIAG_DIR` is a session boundary and must not be shared across concurrent runs
  (multiple terminals, multiple AI agents). Evidence:
  - `docs/ui-diagnostics-and-scripted-tests.md`
  - `docs/workstreams/diag-v2-hardening-and-switches-v1/per-run-layout.md`
  - skill: `.agents/skills/fret-diag-workflow/SKILL.md`
- [x] Design and implement a session-root layout for tool-launched runs so agents can run in parallel without inventing
  directory naming conventions. Proposed design: `docs/workstreams/diag-v2-hardening-and-switches-v1/concurrency-and-sessions.md`.
  - [x] Add `--session-auto` / `--session <id>` for tool-launched commands (`--launch`) that makes the effective out dir
    `<base_dir>/sessions/<session_id>/`.
  - [x] Add a small `session.json` metadata file in the session root (best-effort).
  - [x] Add a safe discovery command (bounded output): `diag list sessions --dir <base_dir>`.
  - [x] Add a safe cleanup command (dry-run by default): `diag sessions clean --dir <base_dir> --keep <n> [--apply]`.

## P1: Agent-native script ergonomics (ImGui-alignment outcomes)

Track design + roadmap:

- `docs/workstreams/diag-v2-hardening-and-switches-v1/ai-era-debugging-stack.md`

Planned outcomes:

- [ ] Named references / scopes (ImGui `SetRef(...)`-style ergonomics, but semantics-first).
  - [ ] Add a script-level `ref` concept (a named selector + optional `window` target).
  - [x] Add steps to set/clear a base ref so subsequent selector-driven steps can be scoped to a subtree:
    - `set_base_ref` / `clear_base_ref` (schema v2)
    - runtime scopes selector resolution while a base ref is active
  - [ ] Ensure the feature is capability-gated (tooling-side) and does not leak policy into `fret-ui`.
- [ ] Multi-viewport docking evidence (make cross-window failures explainable, not just “timeout”):
  - [x] Export a bounded `window.map.json` sidecar in bundle export dirs (window ids + last bounds + hover detection).
  - [x] Record input routing decisions for dock/tear-out flows (why a hover/click went to a different window):
    - runtime sidecar: `dock.routing.json` (bounded; max 512 entries),
    - tooling: `fretboard diag dock-routing <bundle_dir|bundle.schema2.json> [--json]` (bounded report).
- [x] Add bounded queries to avoid opening large artifacts:
  - `diag windows <bundle_dir|bundle.schema2.json>`
  - `diag dock-routing <bundle_dir|bundle.schema2.json>`
- [x] Add a bounded `diag screenshots <bundle_dir|bundle.schema2.json>` query to locate and summarize GPU screenshots without
  hunting through directories.
- [x] Hardening: guard non-convergent scroll scripts (reduce “scroll forever until timeout” flake):
  - detect impossible `require_fully_within_window=true` when the target is larger than the padded window inner rect,
  - fail fast with a stable `reason_code` and bounded evidence (instead of spamming wheel events until timeout).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_scroll.rs`, `ecosystem/fret-bootstrap/src/ui_diagnostics/labels.rs`.
- [x] Hardening: guard impossible “stable frames” configs (reduce avoidable timeouts):
  - detect `stable_frames > timeout_frames` for stability-gated steps (`wait_bounds_stable`, `click_stable`, `click_selectable_text_span_stable`),
  - fail fast with stable `reason_code` (instead of waiting to timeout).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs`, `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer.rs`, `ecosystem/fret-bootstrap/src/ui_diagnostics/labels.rs`.
- [x] Hardening: guard impossible `ensure_visible(within_window=true)` on oversized targets (reduce avoidable timeouts):
  - detect when the target bounds exceed the padded inner window rect,
  - fail fast with stable `reason_code` (instead of waiting to timeout).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_visibility.rs`, `ecosystem/fret-bootstrap/src/ui_diagnostics/labels.rs`.
- [ ] Fast mode policy (determinism + speed):
  - [ ] Make “fast mode vs human-speed” explicit via config (stabilization defaults, animation handling).
  - [ ] Add a bounded “fast mode” smoke suite (runs faster than today without introducing flake).
- [ ] Headless seed (CI-friendly):
  - [ ] Define the minimum supported headless target for native runners.
  - [ ] Add one end-to-end headless smoke suite that produces the same bounded artifacts (manifest + sidecars).

## P2: Time-series visual evidence (optional, evidence UX)

- [ ] Add an opt-in export for bounded time-series screenshots (PNG sequence around failure).
- [ ] Optional: add tooling-side GIF encoding as a post-process (not required for core correctness gates).

## P0.5: Script library modularization (UX scalability)

- [x] Define a folder taxonomy for `tools/diag-scripts/` (by product area + suite intent). See: `docs/workstreams/diag-v2-hardening-and-switches-v1/script-library.md`.
- [x] Decide whether suites should be:
  - [ ] registry-driven (preferred), or
  - [x] glob-driven (acceptable for small sets, but brittle long-term). (v1 decision: curated suite directories + redirect stubs)
- [x] As an intermediate step, switch built-in suite definitions away from Rust-side hard-coded file lists:
  - `diag suite`: curated suite directories + deterministic `**/*.json` expansion,
  - `diag perf`: suite membership resolved via the promoted registry (`tools/diag-scripts/index.json` `suite_memberships`).
- [x] De-duplicate `diag perf` suite script lists by using a single source of truth:
  - `perf_seed_policy::scripts_for_perf_suite_name` is now used by `diag perf` for suite name expansion.
- [x] Move `diag suite` specialized harnesses off Rust-side hard-coded script lists:
  - scripts now live under `tools/diag-scripts/suites/<suite-name>/` and are expanded deterministically at runtime.
- [x] Allow `diag suite <suite-name>` to run any `tools/diag-scripts/suites/<suite-name>/` directory even when the suite
  name was not added to a Rust-side allowlist (suite-specific env defaults remain opt-in).
- [x] Normalize `perf_seed_policy` suite script paths to canonical taxonomy (keep compatibility for redirect-stub scopes in preset files).
- [x] Ensure capability inference resolves `script_redirect` stubs (screenshots / required caps / env defaults).
- [x] Add a script registry file (v1, generated; scope: suites + `_prelude`):
  - [x] file: `tools/diag-scripts/index.json`
  - [x] generator/check: `python tools/check_diag_scripts_registry.py [--write]` (stdlib-only; suitable for CI)
  - [x] fields: `id`, `path`, `tags`, `target_hints`, `required_capabilities`, `suite_memberships`
- [x] CI guardrail: `.github/workflows/consistency-checks.yml`
- [x] Allow `diag run` to accept a promoted `script_id` from `tools/diag-scripts/index.json` (in addition to explicit paths),
  and print suggestions when the id is unknown.
- [x] Add a small discoverability helper to list promoted scripts:
  - [x] `fretboard diag list scripts` (reads `tools/diag-scripts/index.json` and prints `id -> path`)
- [x] Add a small discoverability helper to list known suites:
  - [x] `fretboard diag list suites` (reads `tools/diag-scripts/index.json` and prints `suite_memberships` counts)
- [x] Improve “unknown suite / missing script path” errors to suggest bounded discovery helpers:
  - `diag list suites`, `diag list scripts` (avoid grepping the repo or artifacts).
- [x] Ensure promoted canonical scripts are schema v2 (keep `script_redirect` stubs as schema v1):
  - [x] `fretboard diag doctor scripts` reports and suggests upgrades for promoted schema v1 scripts.
  - [x] Upgrade the remaining promoted schema v1 scripts via `diag script upgrade --write`.
- [x] Prefer `--suite-prelude` for shared resets (`tools/diag-scripts/_prelude/*`) and document the convention.
- [x] Document a migration runbook (dry-run plan → apply moves → validate suites) and link it from `docs/ui-diagnostics-and-scripted-tests.md`.
- [x] Decide path-move compatibility strategy:
  - [ ] registry-first (no moves) to decouple suites from filenames,
  - [x] then move to folders with either redirects (preferred) or “big bang” rewrites.
- [x] If using redirects, implement tooling redirect resolution:
  - [x] add `script_redirect` stub support with loop detection,
  - [x] ensure redirects never reach the runtime (tooling resolves before push).
- [x] Add a migration helper script (plan + apply moves + optional redirects/rewrite).
- [x] Add a guardrail so the taxonomy stays stable:
  - [x] `tools/diag-scripts/migrate-script-library.py --check-root` detects “root scripts” (supports optional filters like `--include-prefix ui-gallery-`).
  - [x] Promote the check into tooling (`fretboard diag doctor scripts`) so drift is visible without relying on Python or ad-hoc greps.
  - [x] document the expected target folders for common categories (ui-gallery, docking, tooling).
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
  - [x] `ui-gallery/pagination` (1 script; post-merge batch)
  - [x] `ui-gallery/scroll-area` (1 script; post-merge batch)
  - [x] `ui-gallery/misc` (redirect-only; 0 canonical scripts)
  - [ ] Replace misc redirects with direct suite references (optional; reduce redirect chain depth)
  - [x] `docking/arbitration` (33 scripts)
- [x] `tooling/external-texture-imports` (9 scripts)
- [x] `tooling/todo` (4 scripts)
- [x] `_prelude/*` (2 scripts)
- [x] post-merge drift batches:
  - [x] `*zinc-dark.json` screenshot scripts (7 scripts; moved into `ui-gallery/ai`, `ui-gallery/command`, and `ui-gallery/dropdown-menu` with redirects)
- [x] Update references after path moves (chosen approach):
  - [x] replace hard-coded lists with registry/directory inputs, or
  - [ ] scripted rewrite of code/docs references (large diff; less preferred).
- [ ] (If needed) write a migration script to move scripts into subfolders and update references:
  - [ ] updates `crates/fret-diag/src/diag_suite_scripts.rs` (or replaces it with a registry reader),
  - [ ] updates any other hard-coded references under `crates/fret-diag/src` (search for `tools/diag-scripts/`),
  - [ ] updates any docs that reference old paths,
  - [ ] optional: runs `fretboard diag script normalize --write` on moved scripts.

## P1: Manifest-first artifacts (transport-neutral)

- [x] Write a single “canonical per-run layout” doc (point to `docs/workstreams/diag-simplification-v1.md` and reconcile terminology). See: `docs/workstreams/diag-v2-hardening-and-switches-v1/per-run-layout.md`.
- [x] Ensure filesystem transport produces a per-run manifest for `diag run/suite/repro/perf`.
- [x] Provide an opt-in manual `diag poke --wait --record-run` workflow that writes a tooling-owned per-run manifest directory for a dump.
- [x] Ensure DevTools WS transport always materializes a per-run manifest alongside `script.result.json`.
- [x] Add an FS dump request surface to carry dump metadata (label/max snapshots/request id), matching WS:
  - [x] tooling writes `dump.request.json` and touches a trigger (`crates/fret-diag/src/transport/fs.rs`),
  - [x] runtime consumes `dump.request.json` for trigger-driven dumps (`ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`),
  - [x] runtime records dump metadata in `bundle.dumped` event logs (beyond the directory name).
- [x] Make `diag pack --ai-only` succeed from manifest + sidecars without `bundle.json` (including extracted share zips where sidecars live under `_root/`).
- [x] Add `diag artifact lint` that validates:
  - [x] manifest schema,
  - [x] chunk list hashes (when present),
  - [x] sidecar schema versions,
  - [x] consistent run id + timestamps.

## P2: Box compatibility logic

- [ ] Create `compat/` modules in tooling for:
  - [x] legacy capability aliases (`script_v2` → `diag.script_v2`, etc),
  - [x] v1 bundle schema reading,
  - [x] v1 script schema reading (if still supported).
- [x] Ensure capabilities are advertised consistently across transports (filesystem + DevTools WS):
  - [x] gesture capabilities include `diag.gesture_long_press` and `diag.gesture_swipe` when supported.
- [x] Close multi-window gaps in schema v2 steps:
  - [x] add optional `window` targeting to selector-driven steps that currently lacked it (e.g. `click_stable`, `wheel`, pointer moves),
  - [x] update capability inference (keep using `diag.multi_window`) and ensure fail-fast gating.
  - [x] Capability-gate runner cursor overrides (`set_cursor_*`) via `diag.cursor_screen_pos_override` so cross-window
    docking scripts fail fast when runner support is missing.
  - [ ] (optional) consider adding `window` to `capture_screenshot` for “no-opinion” per-window evidence collection.
- [x] Input determinism: optionally isolate external (non-script) keyboard input during scripted runs (parallel to pointer isolation),
  with an explicit escape hatch for interactive debugging.
- [x] Clipboard determinism: add script steps to set/assert clipboard text (capability-gated), so paste flows can be tested
  without depending on ambient OS clipboard contents.
  - Guidance: for smoke validation prefer “set clipboard → paste into an input → assert via semantics” over “get/assert
    clipboard text”, because clipboard readbacks can depend on runner callbacks and are easier to flake under harnesses.
  - [ ] (optional) Consider a sandboxed clipboard mode for tool-launched scripted runs to avoid mutating the OS clipboard
    (reduce surprise during local repros; enables parallel runs).
- [x] OS integration determinism: add a capability-gated script step to inject "incoming open" payloads (paths/tokens) to
  cover "open with..." flows deterministically.
- [x] Harness integration: ensure `fret-ui-gallery` records diagnostics events for platform-delivered events (not just
  script-injected ones) so `event_kind_seen` predicates can observe injected OS integration events.
  - Evidence: `fret_bootstrap::ui_diagnostics::maybe_consume_event` and key app drivers call it from `handle_event`
    before dispatching to the UI tree (consistent ignore → record → intercept ordering).
- [x] Extend `FilesystemCapabilitiesV1` with optional identity fields (additive):
  - [x] `runner_kind`, `runner_version`,
  - [x] optional `protocol_versions`/`schemas` hints for tooling.
- [x] Create `transport/` seam contract and ensure all FS vs WS differences are isolated there.
  - Evidence: `crates/fret-diag/src/transport/seam.rs`, `crates/fret-diag/src/lib.rs`
- [x] Add a “legacy usage” marker into `triage.json` / `ai.packet.json` when compat fallbacks were used.
  - Evidence: `crates/fret-diag/src/triage_json.rs`, `crates/fret-diag/src/commands/ai_packet/budget.rs`

## P3: Deprecations + debt removal

- [ ] Turn off legacy writers by default:
  - [x] Add config switches (`write_bundle_json`, `write_bundle_schema2`) and wire them into the runtime dump writer.
  - [x] Tool-launched runs default to `write_bundle_json=false` and `write_bundle_schema2=true` (small-by-default artifacts).
    - Tooling treats failure to write `diag.config.json` as a `--launch` error (avoid silent fallback to defaults).
  - [x] Provide a tool-launched escape hatch for deep debugging:
    - `--launch-write-bundle-json` (requires `--launch`) makes tooling write a per-run config with `write_bundle_json=true`.
    - Not supported for `diag matrix` (too many runs; high risk of output explosion).
  - [x] Decide whether manual defaults should also flip (and document the migration plan for downstream consumers).
    - Decision: do not flip manual defaults yet; keep manual runs compat-first unless `FRET_DIAG_CONFIG_PATH` is used.
  - [ ] Deprecate/remove flags that are now represented as config fields or capabilities.
  - [ ] Delete unused env var parsing paths once CI/scripts migrate (tracked by a migration checklist).

## Migration support (fearless refactor safety)

- [x] Provide a script migration guide (runbook + guardrails): `docs/workstreams/diag-v2-hardening-and-switches-v1/migration-support.md`
  - [ ] (optional) document `diag pick-apply` workflows (when they stabilize).
  - [ ] (optional) add `diag script normalize --check` to CI once script churn stabilizes.
- [x] Add a “compat matrix” table: `docs/workstreams/diag-v2-hardening-and-switches-v1/compat-matrix.md`
