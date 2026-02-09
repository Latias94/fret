# Crate audit (L0) — `fret-ui-headless`

## Crate

- Name: `fret-ui-headless`
- Path: `ecosystem/fret-ui-headless`
- Owners / adjacent crates: `fret-core` (portable types), component ecosystems (`fret-ui-kit`, `fret-ui-shadcn`)
- Current “layer”: ecosystem policy infrastructure (headless state machines / deterministic helpers)

## 1) Purpose (what this crate *is*)

- A collection of headless, reusable state machines and deterministic helpers (no theme/recipe policy).
- Intended to be shared by component ecosystems (`fret-ui-kit`, `fret-ui-shadcn`) without depending on `crates/fret-ui` internals.
- Contains several “interaction primitives” (typeahead, roving focus, presence, hover intent) and a large headless table engine.

Evidence anchors:

- `ecosystem/fret-ui-headless/src/lib.rs`
- `ecosystem/fret-ui-headless/Cargo.toml`

## 2) Public contract surface

- Key exports / stable types:
  - Calendar: `CalendarMonth`, `DateRangeSelection` and related helpers.
  - Interaction helpers: `FadePresence` / `ScaleFadePresence`, `HoverIntent`, `RovingFocus`, `Typeahead`, tooltip intent/delay groups.
  - Table engine: `table::*` (TanStack-aligned vocabulary).
- “Accidental” exports to consider removing:
  - `table` currently re-exports a very large API surface from `table/mod.rs`, which increases churn risk when refactoring internals.
- Feature flags and intent:
  - No long-lived feature gates; this is okay, but it makes “API sprawl” more important to manage via module boundaries and tests.

Evidence anchors:

- `ecosystem/fret-ui-headless/src/lib.rs`
- `ecosystem/fret-ui-headless/src/table/mod.rs`

## 3) Dependency posture

- Backend coupling risks: none observed (no `winit`/`wgpu`/`web-sys`).
- Layering policy compliance: expected for an ecosystem headless crate.
- Compile-time hotspots / heavy deps:
  - Table subsystem is large (5k+ LOC in `row_model.rs`) and will dominate incremental compile time when touched.
  - External dep `virtualizer` is a potential contract seam (virtualization behavior must remain stable across refactors).

Evidence anchors:

- `ecosystem/fret-ui-headless/Cargo.toml`
- `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate fret-ui-headless`

## 4) Module ownership map (internal seams)

- Calendar and calendar variants
  - Files: `ecosystem/fret-ui-headless/src/calendar.rs`, `ecosystem/fret-ui-headless/src/calendar_solar_hijri.rs`
- Deterministic interaction helpers (small state machines)
  - Files: `ecosystem/fret-ui-headless/src/presence.rs`, `ecosystem/fret-ui-headless/src/hover_intent.rs`, `ecosystem/fret-ui-headless/src/roving_focus.rs`, `ecosystem/fret-ui-headless/src/typeahead.rs`, `ecosystem/fret-ui-headless/src/tooltip_*`
- Scroll / viewport helpers
  - Files: `ecosystem/fret-ui-headless/src/grid_viewport.rs`, `ecosystem/fret-ui-headless/src/scroll_area*.rs`
- Headless table engine (TanStack-aligned)
  - Files: `ecosystem/fret-ui-headless/src/table/*`
  - High-risk hotspots:
    - `ecosystem/fret-ui-headless/src/table/row_model.rs`
    - `ecosystem/fret-ui-headless/src/table/filtering.rs`
    - `ecosystem/fret-ui-headless/src/table/sorting.rs`
    - `ecosystem/fret-ui-headless/src/table/tanstack_state.rs`

## 5) Refactor hazards (what can regress easily)

- Determinism vs `HashMap`/`HashSet` iteration in hot paths (table)
  - Failure mode: non-deterministic output ordering causes flicker or unstable virtualization keys when consumers accidentally depend on iteration order.
  - Existing gates: none obvious at L0 (needs targeted tests).
  - Missing gate to add: a fixture-driven table snapshot test (stable row order, stable grouping output) with deterministic expectations.
- Row identity strategy defaults (table)
  - Failure mode: callers rely on index-based `RowKey` and selection/expansion state breaks when data reorders.
  - Existing gates: docs in `RowKey` suggest stable keys, but no executable contract.
  - Missing gate to add: a unit test that demonstrates reorder behavior and “why you must provide stable keys”.
- Calendar selection semantics (range normalization / partial selection)
  - Failure mode: subtle off-by-one or incomplete selection semantics drift, breaking downstream components.
  - Existing gates: none obvious at L0 (needs targeted tests).
  - Missing gate to add: small fixture tests for `DateRangeSelection::apply_click` and normalization.

Evidence anchors:

- `ecosystem/fret-ui-headless/src/table/row_model.rs` (`RowKey` docs + row maps)
- `ecosystem/fret-ui-headless/src/calendar.rs` (`DateRangeSelection`)

## 6) Code quality findings (Rust best practices)

- Many modules are small, well-documented, and deterministic by construction (tick-driven timelines in `presence.rs` are a good example).
- The table engine’s surface and module sizes are the primary maintainability risk.
- Consider a stronger internal API boundary for `table` (keep `pub use` curated, prefer a smaller “front door”).

Evidence anchors:

- `ecosystem/fret-ui-headless/src/presence.rs`
- `ecosystem/fret-ui-headless/src/table/mod.rs`

## 7) Recommended refactor steps (small, gated)

1. Introduce a fixture-driven table regression harness (a few small JSON fixtures + thin Rust runner) — outcome: table refactors become fearless — gate: `cargo nextest run -p fret-ui-headless`.
2. Split `ecosystem/fret-ui-headless/src/table/row_model.rs` into smaller modules (IDs, arena, builders, indexing) while keeping public types stable — outcome: reviewable diffs — gate: table fixture tests + `cargo fmt`.
3. Add targeted unit tests for `DateRangeSelection` semantics — outcome: calendar interactions stable — gate: `cargo nextest run -p fret-ui-headless`.

## 8) Open questions / decisions needed

- Should `table` expose a curated “stable API” module (and keep internal helpers private), to reduce churn as TanStack parity work continues?

