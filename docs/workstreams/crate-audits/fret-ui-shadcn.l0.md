# Crate audit (L0) — `fret-ui-shadcn`

## Crate

- Name: `fret-ui-shadcn`
- Path: `ecosystem/fret-ui-shadcn`
- Owners / adjacent crates: `crates/fret-ui` (mechanism), `ecosystem/fret-ui-kit` (policy primitives), `ecosystem/fret-ui-headless` (state machines)
- Current “layer”: ecosystem component facade (shadcn/ui v4 taxonomy + recipes)

## 1) Purpose (what this crate *is*)

- A shadcn/ui v4-aligned **naming + taxonomy** surface so users can transfer shadcn knowledge and recipes directly.
- Declarative-only component surface (retained widgets are intentionally not part of the public API).
- Owns shadcn-like interaction outcomes (overlay behavior, menu semantics, focus/roving/typeahead policy) on top of the mechanism substrate.

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/lib.rs`
- ADR 0066: `docs/adr/0066-fret-ui-runtime-contract-surface.md`

## 2) Public contract surface

- Key exports / stable types:
  - Large set of component types re-exported from `src/lib.rs` via `pub mod` + `pub use`.
  - `prelude` provides the “golden path” ergonomic imports for app code.
- “Accidental” exports to consider removing:
  - `pub use` count is high; accidental export drift is a real refactor hazard.
  - Consider keeping internals private and using explicit “front door” modules to reduce churn.
- Feature flags and intent:
  - `web-goldens`: heavyweight conformance tests; should remain opt-in for inner-loop speed.
  - `app-integration`, `state-*`: opt-in integration layers (good posture) but need compile gates.

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/lib.rs`
- `ecosystem/fret-ui-shadcn/Cargo.toml`

## 3) Dependency posture

- Backend coupling risks:
  - No direct `winit`/`wgpu`/`web-sys` deps (good: ecosystem components remain backend-agnostic).
- Layering policy compliance:
  - Expected: policy-heavy outcomes remain here (ecosystem), mechanism remains in `crates/fret-ui`.
- Compile-time hotspots / heavy deps:
  - Several single files are “god modules” (5k–6k LOC): `select.rs`, `dropdown_menu.rs`, `context_menu.rs`, `menubar.rs`.
  - Conformance tests are large; feature gating helps, but module structure still matters.

Evidence anchors:

- `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate fret-ui-shadcn`
- `ecosystem/fret-ui-shadcn/src/select.rs`
- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`

## 4) Module ownership map (internal seams)

- Overlay surfaces + interaction policy
  - Files: `ecosystem/fret-ui-shadcn/src/select.rs`, `dropdown_menu.rs`, `context_menu.rs`, `menubar.rs`, `popover.rs`, `tooltip.rs`, `hover_card.rs`, `dialog.rs`, `drawer.rs`, `sheet.rs`, `navigation_menu.rs`
- Form controls and inputs
  - Files: `ecosystem/fret-ui-shadcn/src/input*.rs`, `textarea.rs`, `checkbox.rs`, `radio_group.rs`, `switch.rs`, `slider.rs`, `native_select.rs`
- Data-heavy surfaces (table/grid)
  - Files: `ecosystem/fret-ui-shadcn/src/data_table*.rs`, `data_grid_canvas.rs`, `data_grid_canvas/*`
- Theming + tokens alignment
  - Files: `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`, `layout.rs`, `overlay_motion.rs`
- Test + conformance infrastructure (web goldens)
  - Files: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`, `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`, `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`

## 5) Refactor hazards (what can regress easily)

- Overlay placement + dismissal + focus restore behavior
  - Failure mode: click-through, anchor drift under scroll/wheel, wrong stacking order, focus loss surprises.
  - Existing gates:
    - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (web-golden-backed)
    - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` (paint-level chrome)
  - Missing gate to add: a small “always-run” diag suite that covers a few overlay interactions without requiring web-goldens.
- Public surface drift via large `pub use` set
  - Failure mode: downstream code depends on internal helpers; refactors become breaking changes.
  - Existing gates: none.
  - Missing gate to add: public surface snapshot (or an explicit “exports policy” doc + CI check).
- Feature-combo compilation drift (`state-*`, `app-integration`)
  - Failure mode: optional integration features silently break on one platform/target.
  - Existing gates: none.
  - Missing gate to add: `cargo check -p fret-ui-shadcn --features app-integration` and `--features state` gates.

## 6) Code quality findings (Rust best practices)

- The crate is intentionally policy-heavy, but file sizes suggest the *module boundaries* are not yet strong enough.
- A common pattern is “component + internal policy + test helpers” co-located; consider separating “public component API” from “policy engines” from “test harness plumbing”.

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/lib.rs` (re-export surface + `prelude`)
- `ecosystem/fret-ui-shadcn/src/select.rs`

## 7) Recommended refactor steps (small, gated)

1. Split the largest overlay modules into internal submodules behind a stable facade (keep `pub use` stable, move helpers private) — outcome: reviewable diffs — gate: `cargo nextest run -p fret-ui-shadcn` (with and without key feature flags).
2. Add compile gates for opt-in features — outcome: stop feature drift — gate: `cargo check -p fret-ui-shadcn --features app-integration` and `cargo check -p fret-ui-shadcn --features state`.
3. Keep expanding fixture-driven conformance where appropriate — outcome: less Rust “god test” churn — gate: `--features web-goldens` conformance suites.

## 8) Open questions / decisions needed

- Do we want a formal “exports policy” for `fret-ui-shadcn` (what is stable vs internal), or do we rely purely on convention and review?

