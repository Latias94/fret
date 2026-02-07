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

- [ ] BU-FR-guard-005 Add a lightweight “largest files report” to keep module size drift visible.
  - Goal: prevent new god files from appearing unnoticed during refactors.
  - Implementation options: a small PowerShell script under `tools/` or a `fretboard` subcommand.

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

- [ ] BU-FR-core-013 Write down an explicit async policy per layer (core vs app vs backends vs ecosystem) and add at least one regression gate.
  - Goal: prevent Tokio/executor coupling from leaking into core crates and prevent UI-thread blocking regressions.
  - References: `docs/integrating-tokio-and-reqwest.md`, `docs/integrating-sqlite-and-sqlx.md`.

- [ ] BU-FR-core-014 Define a v1 “serialization stability” checklist for core persisted formats.
  - Target: settings/keymap/layout and any persisted docking state formats.
  - Goal: avoid accidental format drift during internal refactors.

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

- [ ] BU-FR-render-030 Define a minimal renderer regression surface list (text, svg, atlas, clip/shadow).
  - Link: `docs/renderer-refactor-roadmap.md`
  - Evidence anchors: `crates/fret-render/src/renderer/mod.rs`, `crates/fret-render/src/text/mod.rs`

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

## M4 — Ecosystem rationalization

- [ ] BU-FR-eco-040 Maintain an allowlist for any crate using `fret-ui/unstable-retained-bridge`, and keep the list shrinking.
  - Link: `docs/workstreams/retained-bridge-exit-v1.md`
  - Gate: `tools/check_layering.ps1` (extend if needed)

- [ ] BU-FR-eco-041 Map ecosystem crates into “headless → kit → shadcn → specialized” lanes and mark ownership boundaries.
  - Start from: `docs/repo-structure.md`, `docs/workstreams/ecosystem-status.md`
