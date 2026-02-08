# Crate audit (L0) — `fret-ui-kit`

## Crate

- Name: `fret-ui-kit`
- Path: `ecosystem/fret-ui-kit`
- Owners / adjacent crates: `fret-ui`, `fret-runtime`, `fret-ui-headless`, `fret-ui-shadcn`
- Current “layer”: ecosystem policy + component substrate (declarative-only)

## 1) Purpose (what this crate *is*)

- General-purpose, domain-agnostic component/policy layer built on top of `crates/fret-ui` (mechanism layer).
- Provides token-driven styling (`LayoutRefinement` / `ChromeRefinement`) and a higher-level declarative authoring surface (`UiBuilder` + declarative modules).
- Hosts overlay orchestration policy (`OverlayController` + window-scoped overlays like toasts/tooltips) intended to be reused across apps and recipes.

Evidence anchors:

- `ecosystem/fret-ui-kit/src/lib.rs`

## 2) Public contract surface

- Key exports / stable types:
  - styling: `LayoutRefinement`, `ChromeRefinement`, `MetricRef`, `ColorRef`, `Space`, `Radius`, `ShadowPreset`, `WidgetState*`
  - overlay orchestration: `OverlayController`, `OverlayRequest`, `OverlayKind`, `OverlayPresence`, snapshot types
  - authoring surface: `UiBuilder`, `UiPatch`, `UiExt`, `Stylable` / `StyledExt`
  - “UI-kit prelude”: `fret_ui_kit::prelude::*`
- Feature flags and intent:
  - `imui`, `dnd`, `icons`, `recipes` are opt-in; `unstable-internals` exposes internal overlay modules directly.
- “Accidental” exports to consider removing (L0 hypothesis):
  - `#[doc(hidden)]` diagnostics-only exports from `window_overlays` are public; confirm whether they should live in `fret-bootstrap` or a dedicated diagnostics crate.

Evidence anchors:

- `ecosystem/fret-ui-kit/src/lib.rs`
- `ecosystem/fret-ui-kit/Cargo.toml`

## 3) Dependency posture

- Backend coupling risks: none obvious in direct deps; depends on portable crates (`fret-ui`, `fret-core`, `fret-runtime`) plus ecosystem helpers.
- Layering policy compliance: expected; should remain backend-agnostic and extractable.
- Compile-time / complexity hotspots (by file size):
  - `src/window_overlays/tests.rs` (~9.3k LOC) is a major “god test file” risk.
  - `src/declarative/table.rs` (~4.9k LOC) and `src/imui.rs` (~4.8k LOC) are large and likely own multiple responsibilities.

Evidence anchors:

- `ecosystem/fret-ui-kit/Cargo.toml`
- `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate fret-ui-kit`
- `pwsh -NoProfile -File tools/report_largest_files.ps1 -Top 30 -MinLines 800`

## 4) Module ownership map (internal seams)

- Styling + tokens — token-driven layout/chrome vocabulary and refinements
  - Files: `ecosystem/fret-ui-kit/src/style.rs`, `ecosystem/fret-ui-kit/src/theme_tokens.rs`, `ecosystem/fret-ui-kit/src/styled.rs`
- Declarative authoring surface — declarative components + `UiBuilder` patch DSL
  - Files: `ecosystem/fret-ui-kit/src/declarative/*`, `ecosystem/fret-ui-kit/src/ui_builder.rs`, `ecosystem/fret-ui-kit/src/ui_builder_impls.rs`
- Overlay policy substrate — placement/arbitration, request/stack snapshots, window-scoped overlays
  - Files: `ecosystem/fret-ui-kit/src/overlay_controller.rs`, `ecosystem/fret-ui-kit/src/overlay/*`, `ecosystem/fret-ui-kit/src/window_overlays/*`
- Primitives + recipes — higher-level interactive policies and composed controls
  - Files: `ecosystem/fret-ui-kit/src/primitives/*`, `ecosystem/fret-ui-kit/src/recipes/*`

## 5) Refactor hazards (what can regress easily)

- Overlay lifecycle + focus restoration correctness
  - Failure mode: focus lost/incorrect restore; dismissal semantics drift; pointer occlusion ordering drift.
  - Existing gates: extensive unit tests under `window_overlays` and `overlay_controller`.
  - Missing gate to add: a minimal `fretboard diag` suite that exercises a `fret-ui-kit` overlay flow end-to-end in a demo app (to catch runner/host integration drift).
- “God test file” brittleness in `window_overlays/tests.rs`
  - Failure mode: slow review, accidental duplication, hard-to-read intent; high merge-conflict rate; fixture drift hidden in code.
  - Existing gates: `cargo nextest run -p fret-ui-kit` covers it, but the file itself is a maintenance hazard.
  - Missing gate to add: data-driven fixtures (JSON) + a thin harness to reduce LOC and make scenario matrices reviewable.
- Warnings hygiene in ecosystem crates
  - Failure mode: unused variables / dead code accumulate; clippy becomes noisy and unusable as a gate.
  - Existing gates: tests run clean but currently emit `unused_variables` warnings.
  - Missing gate to add: per-crate “warnings must be zero” policy when landing L1+ refactors (or enforce via `clippy -D warnings` at L2).

Evidence anchors:

- `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
- `ecosystem/fret-ui-kit/src/declarative/table.rs` (warning today: `unused variable: group_aggs_any`)

## 6) Code quality findings (Rust best practices)

- Warnings present in baseline:
  - `unused variable: group_aggs_any` in `ecosystem/fret-ui-kit/src/declarative/table.rs` (reported by `cargo nextest run -p fret-ui-kit`).
- File-size hotspots suggest mixed responsibilities:
  - `declarative/table.rs` likely mixes model/state, rendering, interaction policy, and virtualization glue.
  - `imui.rs` suggests an integration facade that might benefit from directory-module structure (`imui/mod.rs` + focused submodules).

Evidence anchors:

- `ecosystem/fret-ui-kit/src/declarative/table.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`

## 7) Recommended refactor steps (small, gated)

1. Eliminate baseline warnings in `fret-ui-kit` — regain “zero warnings” posture for this crate — gate: `cargo nextest run -p fret-ui-kit` (and eventually `cargo clippy ... -D warnings` for L2).
2. Split `src/window_overlays/tests.rs` into directory tests + fixtures:
   - Move scenario matrices to `ecosystem/fret-ui-kit/tests/fixtures/*.json` (or `src/window_overlays/fixtures/`), keep a thin harness — gate: existing nextest coverage + add at least one decode fixture test.
3. Split `src/declarative/table.rs` by responsibility (`model`, `layout`, `virtualization`, `interaction`, `tests`) under `declarative/table/` — gate: `cargo nextest run -p fret-ui-kit`.
4. Decide the long-term home for diagnostics-only exports:
   - Keep public but move behind a `diagnostics` feature, or relocate to a dedicated diagnostics crate — gate: `cargo nextest run -p fret-ui-kit` + `tools/check_layering.ps1`.

## 8) Open questions / decisions needed

- Should `window_overlays` diagnostics-only exports remain in `fret-ui-kit`, or should `fret-bootstrap` own the bundle export vocabulary?
- What is the intended long-term boundary between `fret-ui-kit` overlay policy and `crates/fret-ui` overlay mechanism (to prevent “policy leakage” back into `crates/`)?

