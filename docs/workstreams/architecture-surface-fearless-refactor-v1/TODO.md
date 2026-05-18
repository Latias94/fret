# Architecture Surface Fearless Refactor v1 — TODO

Status: Closed
Last updated: 2026-05-17

## M0 — Scope And Evidence Freeze

- [x] ASF-010 [owner=planner] [deps=none] [scope=docs/workstreams/architecture-surface-fearless-refactor-v1]
  Goal: Record the architecture findings, target state, fearless deletion policy, and first gate set.
  Validation: `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md` exist and agree.
  Evidence: `docs/workstreams/architecture-surface-fearless-refactor-v1/DESIGN.md`
  Handoff: This workstream was opened from the 2026-05-17 architecture surface audit.

- [x] ASF-011 [owner=planner] [deps=ASF-010] [scope=docs/workstreams/README.md,docs/README.md]
  Goal: Link this workstream from the relevant index/docs without making it the only source of truth.
  Validation: `rg -n "architecture-surface-fearless-refactor-v1" docs/README.md docs/workstreams/README.md`
  Evidence: `docs/README.md`; `docs/workstreams/README.md`.
  Handoff: Completed on 2026-05-17; the detailed plan remains in this folder.

## M1 — Minimal App Authoring Profile

- [x] ASF-020 [owner=codex] [deps=ASF-010] [scope=ecosystem/fret/Cargo.toml,ecosystem/fret/src,tools/check_consumption_profiles.py,docs]
  Goal: Make `fret --no-default-features` and `fret --no-default-features --features app` either truly backend-free or remove/document those modes so they do not imply backend-free consumption.
  Validation: `cargo tree -p fret --no-default-features -e normal --depth 4` and `cargo tree -p fret --no-default-features --features app -e normal --depth 4` do not contain `wgpu`, `winit`, `fret-launch`, `fret-render`, `fret-platform-native`, or `fret-runner-winit` unless the final documented profile intentionally opts into them.
  Evidence: `tools/check_consumption_profiles.py`; `ecosystem/fret/Cargo.toml`; `ecosystem/fret/README.md`; `docs/crate-usage-guide.md`.
  Handoff: Completed on 2026-05-17; `desktop` now owns the native runner/render stack and `app` remains backend-free.

- [x] ASF-021 [owner=codex] [deps=ASF-020] [scope=ecosystem/fret/src/lib.rs,ecosystem/fret/src/app_entry.rs,ecosystem/fret/README.md,docs/examples]
  Goal: Separate backend-running methods from backend-free app authoring types so `FretApp` or its replacement has an honest feature boundary.
  Validation: `cargo check -p fret --no-default-features --features app` and a targeted template/doc gate if affected.
  Evidence: `ecosystem/fret/src/app_entry.rs`; `ecosystem/fret/src/lib.rs`; `ecosystem/fret/tests/backend_free_app_authoring_profile.rs`; `tools/check_consumption_profiles.py`; `ecosystem/fret/README.md`; `docs/crate-usage-guide.md`.
  Handoff: Completed on 2026-05-17; `FretApp` is now a backend-free authoring spec and desktop window/runner methods remain `desktop`-only.

## M2 — Bootstrap Plan vs Launch Adapter

- [x] ASF-030 [owner=codex] [deps=ASF-020] [scope=ecosystem/fret-bootstrap,crates/fret-launch,docs/adr/IMPLEMENTATION_ALIGNMENT.md]
  Goal: Split backend-free bootstrap planning/default policy from concrete launch/render adapters.
  Validation: `cargo tree -p fret-bootstrap --no-default-features -e normal --depth 4` does not contain `wgpu`, `winit`, `fret-render`, or platform-native runner crates unless an explicitly named feature is enabled.
  Evidence: `ecosystem/fret-bootstrap/src/assets.rs`; `ecosystem/fret-bootstrap/tests/backend_free_bootstrap_profile.rs`; `tools/check_consumption_profiles.py`; `docs/crate-usage-guide.md`; `docs/adr/IMPLEMENTATION_ALIGNMENT.md`.
  Handoff: Completed on 2026-05-17; `fret-bootstrap --no-default-features` now exposes backend-free bootstrap asset/default policy, while `launch` owns the concrete `fret-launch` / `fret-render` adapter surface.

- [x] ASF-031 [owner=codex] [deps=ASF-030] [scope=ecosystem/fret-bootstrap,ecosystem/fret,crates/fretboard,docs]
  Goal: Migrate first-party app/template callers onto the new bootstrap/launch split and delete displaced helper aliases.
  Validation: focused `cargo check` for affected packages plus template tests if any are touched.
  Evidence: `ecosystem/fret/src/app_entry.rs`; `ecosystem/fret/tests/backend_free_app_authoring_profile.rs`; `crates/fretboard/src/scaffold/templates.rs`; `crates/fretboard/src/scaffold/mod.rs`; `docs/crate-usage-guide.md`; `docs/examples/todo-app-golden-path.md`.
  Handoff: Completed on 2026-05-17; `FretApp::asset_startup(...)` records backend-free startup specs, generated templates apply plans through `generated_assets::mount(builder)?` / `UiAppBuilder::with_asset_startup(...)`, and stale generated-template `AppUi` call sites were migrated.

## M3 — Public Facade Narrowing

- [x] ASF-040 [owner=codex] [deps=ASF-020] [scope=ecosystem/fret/src/lib.rs,ecosystem/fret/src/view.rs,ecosystem/fret/tests,docs]
  Goal: Define and enforce the narrow `fret::app::prelude::*` Golden Path budget.
  Validation: surface tests assert the prelude contains only the approved app-authoring imports and excludes advanced/compatibility names.
  Evidence: `ecosystem/fret/src/lib.rs`; `docs/crate-usage-guide.md`.
  Handoff: Completed on 2026-05-17; `fret::app::prelude::*` now has a source-level closed pub-use budget plus docs that describe named exports and anonymous extension traits.

- [x] ASF-041 [owner=codex] [deps=ASF-040] [scope=ecosystem/fret/src/view.rs,ecosystem/fret/src/view/local_state.rs,ecosystem/fret/src/actions.rs,ecosystem/fret/src/lib.rs]
  Goal: Split the large view/action authoring implementation into deeper owner modules without widening the public interface.
  Validation: existing `fret` tests plus targeted compile tests for `LocalState`, typed actions, selector/query reads, and advanced raw-model escape hatches.
  Evidence: `ecosystem/fret/src/view/local_state.rs`; `ecosystem/fret/src/view.rs`.
  Handoff: Completed on 2026-05-17; the `LocalState` / `WatchedState` / tracked read owner family now lives in a private `view/local_state.rs` module and is re-exported through `crate::view` without widening the public surface.

## M4 — Ecosystem Taxonomy Closure

- [x] ASF-050 [owner=codex] [deps=ASF-010] [scope=ecosystem/fret-ui-headless,ecosystem/fret-ui-kit,ecosystem/fret-authoring,docs/adr/0154-ecosystem-crate-taxonomy-glue-and-ui-kit-split-v1.md,docs/adr/IMPLEMENTATION_ALIGNMENT.md]
  Goal: Decide and land the headless/primitives/kit taxonomy for one representative primitive family.
  Validation: targeted tests for the chosen family and `python tools/check_layering.py`.
  Evidence: `ecosystem/fret-ui-headless/src/boolean_control.rs`; `ecosystem/fret-ui-kit/src/primitives/{checkbox.rs,switch.rs}`; `ecosystem/fret-ui-shadcn/src/{checkbox.rs,switch.rs}`; `docs/adr/IMPLEMENTATION_ALIGNMENT.md`.
  Handoff: Completed on 2026-05-17; the boolean-control family proves the split without reintroducing `fret-ui-primitives`: pure transitions live in `fret-ui-headless`, kit primitives keep runtime/a11y/model facades, and first-party recipes consume the headless owner directly.

- [x] ASF-051 [owner=codex] [deps=ASF-050] [scope=ecosystem/fret-ui-kit,ecosystem/fret-ui-shadcn,ecosystem/fret-ui-material3]
  Goal: Migrate at least one recipe surface to consume the finalized primitive taxonomy directly instead of depending on broad kit compatibility shims.
  Validation: package tests for the recipe crate and no new backend deps in ecosystem crates.
  Evidence: `ecosystem/fret-ui-shadcn/src/carousel.rs`; `docs/audits/shadcn-carousel.md`; `docs/adr/IMPLEMENTATION_ALIGNMENT.md`.
  Handoff: Completed on 2026-05-17; `fret-ui-shadcn::carousel` now imports `fret_ui_headless::{carousel, embla, snap_points}` directly instead of routing pure engines through `fret_ui_kit::headless`.

## M5 — Shared Menu/Select Policy

- [x] ASF-060 [owner=codex] [deps=ASF-050] [scope=ecosystem/fret-ui-kit,ecosystem/fret-ui-headless,ecosystem/fret-ui-shadcn/src/{select.rs,dropdown_menu.rs,context_menu.rs,menubar.rs}]
  Goal: Extract one shared menu/select interaction module covering a concrete behavior such as roving focus, typeahead, submenu grace intent, dismissal, or entry focus.
  Validation: targeted `fret-ui-shadcn` tests for select/dropdown/context menu parity, plus any new headless/primitive unit tests.
  Evidence: `ecosystem/fret-ui-headless/src/entry_focus.rs`; `ecosystem/fret-ui-kit/src/primitives/{menu/root.rs,select.rs}`; `ecosystem/fret-ui-shadcn/src/select.rs`; `docs/adr/IMPLEMENTATION_ALIGNMENT.md`.
  Handoff: Completed on 2026-05-17; entry-focus target selection is now a headless owner consumed by menu and select runtime adapters. `cargo test -p fret-ui-shadcn --locked --test select_keyboard_navigation -j 1` exposed a pre-existing/adjacent expectation conflict around pointer-open ArrowDown selecting the first item; do not count that command as passed evidence.

- [x] ASF-061 [owner=codex] [deps=ASF-060] [scope=docs/workstreams/shadcn-menu-select-policy-followon-v1,docs/adr/IMPLEMENTATION_ALIGNMENT.md]
  Goal: Record whether the first extraction proves a broader menu/select cleanup lane or should remain a narrow fix.
  Validation: updated workstream note or follow-on split.
  Evidence: `docs/workstreams/shadcn-menu-select-policy-followon-v1/{WORKSTREAM.json,DESIGN.md,TODO.md,EVIDENCE_AND_GATES.md}`.
  Handoff: Completed on 2026-05-17; ASF-060 remains the architecture-surface proof, while remaining shadcn select/menu semantics move to the narrow follow-on. The older `menu-surfaces-alignment-v1` lane stays completed historical OS/in-window menubar scope.

## M6 — Renderer Facade Decision

- [x] ASF-070 [owner=codex] [deps=ASF-010] [scope=crates/fret-render,crates/fret-render-core,crates/fret-render-wgpu,docs/workstreams/renderer-modularity-fearless-refactor-v1,docs/adr/IMPLEMENTATION_ALIGNMENT.md]
  Goal: Decide whether `fret-render` is collapsed into `fret-render-wgpu` or deepened into the renderer interface.
  Validation: a short decision note plus one compile gate demonstrating the chosen profile.
  Evidence: `docs/workstreams/architecture-surface-fearless-refactor-v1/JOURNAL/2026-05-17-asf-070.md`; `docs/workstreams/renderer-modularity-fearless-refactor-v1/CLOSEOUT_AUDIT.md`; `crates/fret-render/src/lib.rs`; `crates/fret-render/tests/facade_surface_snapshot.rs`; `docs/adr/IMPLEMENTATION_ALIGNMENT.md`.
  Handoff: Completed on 2026-05-17; `fret-render` remains the curated default renderer facade. Do not collapse it into `fret-render-wgpu`; future renderer semantic/capability work should open a renderer-specific follow-on instead of widening this architecture-surface lane.

## M7 — Closeout

- [x] ASF-080 [owner=planner] [deps=ASF-020,ASF-030,ASF-040,ASF-050,ASF-060,ASF-070] [scope=docs/workstreams/architecture-surface-fearless-refactor-v1]
  Goal: Close the lane or split remaining work into narrower follow-ons.
  Validation: `EVIDENCE_AND_GATES.md` contains final gate results and `WORKSTREAM.json` status is updated.
  Evidence: `docs/workstreams/architecture-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-05-17.md`.
  Handoff: Completed on 2026-05-17; the lane is closed. Remaining shadcn menu/select policy work is owned by `docs/workstreams/shadcn-menu-select-policy-followon-v1/`; future renderer semantic/capability work should open a renderer-specific follow-on.
