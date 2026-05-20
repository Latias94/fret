# Fearless Refactor Architecture Issues Ledger

Date: 2026-05-18
Status: Active issue ledger

## Purpose

This ledger records architecture issues discovered while planning the next fearless-refactor pass.
It is intentionally not a new umbrella workstream. The umbrella remains
`bottom-up-fearless-refactor-v1`; concrete execution should happen in narrow owner lanes with one
repro, one gate set, and one evidence set per slice.

Use this file when deciding what to run next, or when a closed lane should stay closed and a new
narrow follow-on is needed.

## Operating Stance

- Pre-1.0 compatibility is not a default constraint. Delete old aliases, bridge paths, and redundant
  wrappers when first-party callers can be migrated and gates prove the new path.
- Do not reopen closed broad lanes. Open a narrower follow-on only after there is fresh evidence and
  a concrete gate.
- Keep `crates/fret-ui` mechanism-only. Shared component policy belongs in `fret-ui-headless`,
  `fret-ui-kit`, or a recipe crate.
- Codex goals should target one bounded `TODO.md` task, not this entire program.

## Current Guardrail Snapshot

Recent local checks:

- `python3 tools/check_layering.py` passed.
- `python3 tools/check_consumption_profiles.py` passed.
- `cargo check -p fret-ui-shadcn --no-default-features` passed.
- `python3 tools/report_largest_files.py --top 25 --min-lines 3000` identified the large-file
  hotspots below.

## Issues

### FIR-001 - Retained bridge blast radius is still the clearest compatibility debt

Priority: P0
Owner lane: `docs/workstreams/retained-bridge-exit-v1/`

Problem:

`fret_ui::retained_bridge` is explicitly unstable and feature-gated, but it remains enabled by five
ecosystem crates. This is acceptable only while it is actively shrinking.

Evidence:

- Runtime bridge warning: `crates/fret-ui/src/retained_bridge.rs`
- Feature gate: `crates/fret-ui/src/lib.rs`
- Allowlist enforcement: `tools/check_layering.py` (`unstable_retained_bridge_allowlist`)
- Current allowlist: `fret-docking`, `fret-node`, `fret-chart`, `fret-plot`, `fret-plot3d`
- Existing plan: `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1.md`

Next action:

- Start with docking. Audit retained bridge call sites, classify each as `delete`, `migrate`, or
  `keep temporarily with gate`, then land one small removal/migration slice.

Minimum gates:

- `cargo nextest run -p fret-docking`
- `python3 tools/check_layering.py`

### FIR-002 - shadcn menu/select policy surfaces are too large for safe evolution

Priority: P0
Owner lane: new narrow follow-on only after a concrete repro; do not reopen
`shadcn-menu-select-policy-followon-v1`.

Problem:

`select.rs`, `dropdown_menu.rs`, `context_menu.rs`, and `menubar.rs` remain large, drift-prone
recipe surfaces. The previous Select ArrowDown follow-on is closed and should stay closed unless a
new shared-policy repro appears.

Evidence:

- Closed lane: `docs/workstreams/shadcn-menu-select-policy-followon-v1/CLOSEOUT_AUDIT_2026-05-17.md`
- Large files from local report:
  - `ecosystem/fret-ui-shadcn/src/select.rs` (~11093 lines)
  - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (~10782 lines)
  - `ecosystem/fret-ui-shadcn/src/context_menu.rs` (~10323 lines)
  - `ecosystem/fret-ui-shadcn/src/menubar.rs` (~7492 lines)

Next action:

- Run a source audit for repeated roving/typeahead collection, submenu grace/focus transfer, and
  dismissal/focus restore logic across at least two surfaces.
- If a repeated behavior has a failing or missing gate, open a focused follow-on for that behavior
  only.

Minimum gates:

- `cargo nextest run -p fret-ui-headless`
- `cargo nextest run -p fret-ui-kit`
- Focused `cargo nextest run -p fret-ui-shadcn` tests for the touched family.
- `python3 tools/check_layering.py`

### FIR-003 - Huge conformance/test sources need fixture-driven burn-down

Priority: P1
Owner lane: `bottom-up-fearless-refactor-v1` until a dedicated fixture burn-down lane is opened.

Problem:

Several hand-authored Rust test sources are too large to review comfortably. They are active
architecture risk because they slow refactors, increase merge conflicts, and make parity updates
hard to audit.

Evidence:

- `ecosystem/delinea/src/engine/tests.rs` (~18176 lines)
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (~14287 lines)
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` (~12511 lines)
- Existing bottom-up task: `BU-FR-guard-004`

Next action:

- Continue converting shadcn overlay/layout conformance matrices into JSON fixtures plus thin Rust
  runners before adding new cases.

Minimum gates:

- The focused test being converted.
- `python3 tools/check_layering.py`
- `git diff --check`

### FIR-004 - `fret-launch` remains a glue gravity well

Priority: P1
Owner lane: `docs/workstreams/framework-modularity-fearless-refactor-v1/`

Problem:

`fret-launch` still carries platform, web, winit, wgpu, effect-draining, and platform SDK weight.
The consumption profile gates are now healthy, but the implementation split is not complete.

Evidence:

- Existing TODO: `docs/workstreams/framework-modularity-fearless-refactor-v1/TODO.md`
- Audit snapshot: `python3 tools/audit_crate.py --crate fret-launch`
- Consumption gate: `python3 tools/check_consumption_profiles.py`

Next action:

- Audit `crates/fret-launch` by platform responsibility and split the first behavior-preserving
  implementation module or crate boundary without changing the public facade.

Minimum gates:

- `python3 tools/check_consumption_profiles.py`
- `cargo nextest run -p fret-launch`
- `python3 tools/check_layering.py`

### FIR-005 - Public facade surfaces are still broad

Priority: P1
Owner lanes:

- `docs/workstreams/framework-modularity-fearless-refactor-v1/`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/`

Problem:

The `fret` and runtime-facing facades are improved but still broad. Broad facades make it harder to
teach the golden path and easier for compatibility escape hatches to look stable.

Evidence:

- `python3 tools/audit_crate.py --crate fret`: `36 pub mod`, `131 pub use`
- `python3 tools/audit_crate.py --crate fret-runtime`: `52 pub mod`, `62 pub use`
- Closed architecture-surface finding: app prelude is now narrower, but public promotion rules are
  still open in the framework-modularity TODO.

Next action:

- Add or refresh public-surface snapshot tests before deleting aliases, then remove one stale
  explicit compatibility lane at a time.

Minimum gates:

- `cargo nextest run -p fret`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`

### FIR-006 - `fret-ui` layout should not be redesigned, but clean-geometry needs a private owner

Priority: P1
Owner lane: `docs/workstreams/fret-ui-layout-architecture-audit-v1/`

Problem:

The layout architecture is conceptually sound, but `tree/layout/node.rs` mixes ordinary layout
execution with the clean-geometry proof model. Future proof additions will become expensive to
review if the model stays in the same file.

Evidence:

- `docs/workstreams/fret-ui-layout-architecture-audit-v1/ARCHITECTURE_DECISION_2026-05-18.md`
- `docs/workstreams/fret-ui-layout-architecture-audit-v1/ARCHITECTURE_INVENTORY_2026-05-18.md`

Next action:

- Extract clean-geometry proof helpers into a private module without changing behavior.

Minimum gates:

- `cargo nextest run -p fret-ui`
- Relevant layout/resize clean-geometry tests.
- `python3 tools/check_layering.py`

### FIR-007 - Diagnostics code has its own large-file pressure

Priority: P2
Owner lane: one of the active diag fearless-refactor lanes after state resolution.

Problem:

`crates/fret-diag` has multiple large files. This is less urgent than retained bridge or shadcn
policy debt, but it will slow diagnostics evolution if left alone.

Evidence:

- `crates/fret-diag/src/tests.rs` (~9488 lines)
- `crates/fret-diag/src/diag_campaign.rs` (~9391 lines)
- `crates/fret-diag/src/stats/bundle_stats_report.inc.rs` (~7363 lines)
- `crates/fret-diag/src/cli/cutover.rs` (~6733 lines)

Next action:

- Resolve the active diag lane state, then split one test/campaign matrix into fixtures or
  responsibility modules.

Minimum gates:

- Focused `cargo nextest run -p fret-diag`
- `python3 tools/check_layering.py`

## First Executable Slice

Start with `retained-bridge-exit-v1` because it matches the current pre-1.0 compatibility posture
and has an active owner lane.

Recommended first task:

- `RBX-M1-010`: docking retained bridge usage audit and first removal/migration slice.

Why:

- It has a concrete allowlist owner.
- It directly shrinks unstable runtime surface area.
- It does not require reopening a closed menu/select lane.
- It can be validated with crate-local tests plus layering.
