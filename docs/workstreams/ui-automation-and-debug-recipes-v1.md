---
title: UI Automation + Debug Recipes v1
status: draft
date: 2026-01-30
scope: diagnostics, automation, profiling, debugging
---

# UI Automation + Debug Recipes v1

This workstream aims to make “AI-friendly debugging” and “repeatable performance triage” first-class in Fret:

- reproduce issues deterministically (prefer semantics selectors over coordinates),
- capture a portable bundle that is sufficient to debug (including offline / by an AI),
- attribute performance hotspots to meaningful UI subtrees and mechanisms,
- keep everything layered: `fret-ui` stays a mechanism/contract layer (ADR 0066), policy lives above.

## Problem statement

Today we already have many ad-hoc debugging tools (logs, dumps, RenderDoc, Tracy, bundle exports). The missing piece is a
single, unified, extensible workflow that:

1) scales to “lots of small correctness/feel bugs” (focus, overlays, input capture, stale paint, missing invalidations),
2) scales to performance debugging (worst-frame search, attribution, regression thresholds),
3) is robust enough that an AI agent can drive it end-to-end (start app → run script → capture artifacts → triage → compare).

Examples of target bugs:

- sliders that don’t drag correctly or have incorrect capture semantics,
- UI that fails to repaint (e.g. search results update but pixels don’t, or text disappears until another event),
- overlay barriers that block underlay input unexpectedly,
- heavy layout hot spots (e.g. Taffy measure/solve dominated frames).

User-facing performance concerns this workstream should explicitly answer:

- **CPU utilization**: avoid “mysterious background work” (unnecessary redraws, polling loops, runaway invalidation).
- **Memory footprint**: keep native apps lightweight (no browser runtime), and produce evidence for regressions.
- **Redraw efficiency**: settle “immediate-mode overdraw” debates with measurable signals (paint/cache reuse/invalidation),
  not philosophy.

## Existing foundation (what we should build on)

Fret already has most of the primitives needed for a “test-engine-like” workflow:

- Scripted actions + selectors (semantics/test_id first): `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- Inspect/picker overlay producing stable selectors: `docs/debugging-ui-with-inspector-and-scripts.md`
- Portable bundle export (`bundle.json`) + offline viewer: `apps/fretboard/src/diag.rs`, `tools/fret-bundle-viewer`
- Perf triage and matrix compare: `fretboard diag perf|stats|compare|matrix`, `tools/diag_matrix_ui_gallery.ps1`
- Tracy integration via `tracing`: `docs/tracy.md`
- RenderDoc scriptable inspection: `docs/renderdoc-inspection.md`, `apps/fret-renderdoc`
- “Stale paint” detection based on semantics bounds vs scene fingerprint: `apps/fretboard/src/diag.rs` (`--check-stale-paint`)

This workstream should *not* replace those pieces; it should unify and extend them.

## Known gaps (as of 2026-01-30)

This section is a quick “what’s real today vs what we want in v1” checklist.

- **`diag repro` is not yet the full unified runner.**
  - `fretboard diag repro <script|suite>` exists as a convenience wrapper.
  - It writes `FRET_DIAG_DIR/repro.summary.json` and packs `FRET_DIAG_DIR/repro.zip`.
  - It writes `FRET_DIAG_DIR/evidence.index.json` as a lightweight index of the key artifacts/checks/resources for AI/CI.
  - For suites, it packs **multiple** bundles under stable zip prefixes (and includes script sources under `_root/scripts/`).
  - It can best-effort request RenderDoc autocapture (`--with renderdoc`) and attempt post-capture exports via
    `fret-renderdoc dump` into `FRET_DIAG_DIR/renderdoc/inspect/` (and includes those artifacts in `repro.zip`).
  - Tracy support is still partial: `--with tracy` enables `FRET_TRACY=1` and can auto-inject `--features fret-bootstrap/tracy`
    for `cargo run` launches, but capture export still requires the Tracy UI (no automated `tracy-capture` integration yet).
  - The machine summary is still evolving (it is not yet a stabilized CI gate format).
- **Screenshot capture remains split into two modes.**
  - `FRET_DIAG_SCREENSHOT=1` writes `frame.bmp` during bundle dumps (dump-triggered, bundle-scoped).
  - `FRET_DIAG_SCREENSHOTS=1` enables the on-demand PNG protocol used by script steps like `capture_screenshot`.
  - These are intentionally separate today, but the UX and documentation should keep them unambiguous.
- **High-level intent actions are still missing.**
  - Scripts mostly use low-level v1 steps (`click`, `drag_pointer`, `wheel`, `wait_until`, ...).
  - `set_slider_value`, `menu_select`, and `scroll_into_view` are not first-class yet.
- **Repaint checks remain best-effort, but are now automation-friendly.**
  - In addition to stale paint / stale scene checks, `--check-semantics-changed-repainted` can flag “semantics changed but
    paint did not” when `semantics_fingerprint` changes without a `scene_fingerprint` change, and it can emit a structured
    evidence file via `--dump-semantics-changed-repainted-json`.
  - For “expected pixels must change” assertions, `--check-pixels-changed <test_id>` can hash the screenshot region inside
    the target semantics bounds and gate on first-vs-last changes (writes `check.pixels_changed.json` to `FRET_DIAG_DIR`).
- **Tracy / RenderDoc are only partially integrated.**
  - `--with renderdoc` wires env vars for autocapture, attempts `fret-renderdoc dump` post-run (best-effort, requires
    RenderDoc installed), and includes `.rdc` + dump JSON/PNG in `repro.zip`.
  - `--with tracy` enables `FRET_TRACY=1` and can auto-inject `--features fret-bootstrap/tracy` for `cargo run` launches, but
    capture export still requires the Tracy UI (no automated `tracy-capture` integration yet).

## Goals (v1)

1. **One-command repro packaging**
   - A single CLI entrypoint that can run scripts/suites, capture bundles, and pack a shareable zip + machine summary.
2. **High-level interaction library**
   - Add robust, composable actions so scripts can read like “user intent” rather than low-level input.
   - Example: `set_slider_value`, `scroll_into_view`, `menu_select`, `drag_to_target`, `ensure_focused`.
3. **Better “missing repaint” debugging**
   - Expand automated checks so “UI did not refresh” becomes a diagnosable, triageable failure with evidence.
4. **Performance queries as data, not just logs**
   - Standardize the performance report surface so tools can query:
     - worst frame(s),
     - layout/paint breakdown,
     - top solve/measure hotspots (already partially available),
     - view-cache reuse and invalidation categories,
     - optional GPU-side markers (future).
5. **Extensible contracts**
   - Adding a new action, a new check, or a new exported metric should not require rewriting tooling.

## Non-goals (v1)

- Screenshot goldens as the primary correctness strategy.
- Shipping a production inspector UI inside `fret-ui`.
- Perfect IME record/replay in CI (best effort only; see ADR 0174).

## Design principles

- **Semantics-first**: selectors and assertions evaluate against the `SemanticsSnapshot` (ADR 0033), not pixels.
- **Layering**:
  - `crates/fret-ui`: mechanism hooks + small debug snapshots (feature-gated).
  - `ecosystem/fret-bootstrap`: diagnostics service, script runner, bundling logic.
  - `apps/fretboard`: orchestration, packing, comparisons, thresholds.
- **Versioned artifacts**:
  - `bundle.json` and script schemas are versioned and forward-compatible (unknown fields ignored).
- **Determinism**:
  - scripts run “one step per frame” (or a deterministic frame loop), not wall-clock sleeps.
- **Failure is an artifact**:
  - every failure produces a bundle (and optionally screenshots/RenderDoc dumps) to debug offline.

## Proposed architecture (unified framework)

### 1) “Recipe runner” (CLI orchestration)

Introduce a single “do the right thing” command (name TBD):

- `fretboard diag repro <script|suite> --launch -- <cmd...>`

Output:

- `repro.zip` containing:
  - `bundle.json` (or multiple bundles, with stable naming),
  - `triage.json` and `stats.json`,
  - `script.json` + `script.result.json`,
  - optional screenshots (`FRET_DIAG_SCREENSHOTS=1`),
  - optional RenderDoc inspection JSON (offline, from `.rdc`),
  - optional Tracy metadata (at least “how to correlate”; future: capture export).
- `repro.summary.json` (small, stable, machine-readable):
  - pass/fail, failing step, selected bundle path(s),
  - worst frame ids and top hotspots,
  - known gating checks results.

This consolidates the “human workflow” into a single, AI-friendly interface.

### 2) Action library (high-level steps)

Add “intent-level” steps (either as new script schema steps, or as a compiler that expands to v1 steps):

- `ensure_visible { target }` (scroll/wheel until visible; asserts within viewport bounds)
- `drag_to { target, to_x|to_y|to_target }` (pointer drag with bounds-aware start/end)
- `set_slider_value { target, value }`
  - requires semantics to expose a meaningful value (see Core hooks below),
  - resolves to drag + `wait_until` predicate on semantics value change.
- `menu_select { path: ["File","Close"] }` (open menus + click items)
- `type_text_into { target, text }` (focus target if needed, then type)
- `assert_repaint { target, mode }` (see “missing repaint” below)

Selection strategy:

- Prefer `test_id`.
- Fallback to `role_and_path` / `role_and_name`.
- Only use `node_id`/coordinates as an explicit fallback mode (debug-only).

### 3) “Missing repaint” and correctness checks

The goal is: when a repaint bug happens, we can answer “why didn’t we repaint?” without guessing.

Checks to standardize (CLI gates + bundle evidence):

- **Semantics moved but scene fingerprint didn’t** (already: `--check-stale-paint`).
- **Semantics changed but scene fingerprint didn’t** (tooling: `--check-semantics-changed-repainted`):
  - requires a `semantics_fingerprint` per snapshot (core hook).
  - optional: `--dump-semantics-changed-repainted-json` writes `check.semantics_changed_repainted.json` next to
    `bundle.json` for machine-readable evidence (AI/CI triage).
- **Expected-to-change region didn’t repaint** (new, screenshot-backed optional):
  - tooling: `--check-pixels-changed <test_id>`
  - requires `capture_screenshot` steps (and screenshots enabled),
  - computes a region hash inside the target semantics bounds and asserts it changed,
  - writes `check.pixels_changed.json` to `FRET_DIAG_DIR` for AI/CI triage.
- **Invalidation accounting** (new optional):
  - when an action should cause layout or paint, assert we saw a matching invalidation walk/source/category.

This is intentionally “best effort” but must produce actionable artifacts, not just a boolean.

### 4) Performance query surface

Standardize the perf surface so tools can query it uniformly:

- per snapshot:
  - `layout_time_us`, `paint_time_us`, `total_time_us` (already in stats),
  - `layout_engine_solves` + top measures (already exported),
  - invalidation walks (already exported),
  - cache-root reuse/replay counters (already exported).
- per bundle:
  - worst-N frames by `time` and by `invalidation`,
  - stable “hotspot labels” (prefer `test_id`, else element path).

Tooling gates (CI/automation):

- `fretboard diag perf` supports threshold gating on the per-run “top frame” timings:
  - `--max-top-total-us <n>`
  - `--max-top-layout-us <n>`
  - `--max-top-solve-us <n>` (layout-engine solve time)
- When any threshold is set, it writes `check.perf_thresholds.json` to `FRET_DIAG_DIR` and exits non-zero on failure.
- For maintainable CI configs, tooling can also read/write a small baseline file:
  - `--perf-baseline-out <path> --perf-baseline-headroom-pct <n>` writes a `kind=perf_baseline` JSON file,
  - `--perf-baseline <path>` loads thresholds per script and gates accordingly.

Notes (current status):

- **Process-level resource footprint** is captured (best-effort) via `resource.footprint.json` and referenced from
  `repro.summary.json`. `diag repro` can also gate on these values:
  - `--max-working-set-bytes <n>`
  - `--max-peak-working-set-bytes <n>`
  - `--max-cpu-avg-percent-total-cores <pct>`
  When any threshold is set, it writes `check.resource_footprint.json` and exits non-zero on failure.
  On Windows this uses native APIs; on non-Windows platforms it uses lightweight sampling (via `sysinfo`) while waiting
  for the demo to exit (CPU sampling is cadence-sensitive).
- **Redraw-efficiency gates** now include an “idle should not paint” trailing-streak gate
  (`--check-idle-no-paint-min <n>`; evidence: `check.idle_no_paint.json`) and a “view cache reuse should be stable”
  trailing-streak gate (`--check-view-cache-reuse-stable-min <n>`; evidence: `check.view_cache_reuse_stable.json`).

Future (optional, gated):

- GPU timing via wgpu timestamp queries (per pass or coarse per-frame), exported as `gpu_time_us`.
- Upload/budget metrics (align with renderer budget ADRs).

### 5) Core hooks we may need (acceptable refactors)

To support robust automation and repaint debugging, we likely need a few core-level improvements:

1. **Semantics value for range controls**
   - A slider should expose a numeric value (and optionally min/max/step) via semantics (ADR 0033-aligned).
   - Scripts can then `set_slider_value` by driving pointer drag until the value predicate matches.
2. **`semantics_fingerprint`**
   - A small hash per snapshot enabling “semantics changed but paint didn’t” detection.
3. **Deterministic “action resolution” mode**
   - Ensure selector resolution and picking can temporarily disable caching/hitbox elision (ADR 0174).
4. **More structured invalidation attribution**
   - The bundle should answer: “this action caused invalidation X because Y” (without scraping logs).

These must remain feature-gated and must not turn `fret-ui` into a policy layer.

## Open questions

- Do we add a **Script v2** schema (recommended), or compile v2 recipes into v1 steps for backwards compatibility?
- Where should the high-level action library live?
  - Option A: in `fret-bootstrap` (runtime-side, closest to the engine)
  - Option B: in `fretboard` (tooling-side compiler, easiest iteration)
- How far should we go with GPU profiling in v1 vs v2?

## Suggested next reads

- ADR 0174 (diagnostics + scripted tests): `docs/adr/0174-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- Debugging workflow: `docs/debugging-ui-with-inspector-and-scripts.md`
- Bundles + scripts details: `docs/ui-diagnostics-and-scripted-tests.md`
- Tracy correlation: `docs/tracy.md`
- RenderDoc inspection: `docs/renderdoc-inspection.md`
