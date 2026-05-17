# Architecture Surface Fearless Refactor v1 — TODO

Status: Active
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

- [ ] ASF-030 [owner=unassigned] [deps=ASF-020] [scope=ecosystem/fret-bootstrap,crates/fret-launch,docs/adr/IMPLEMENTATION_ALIGNMENT.md]
  Goal: Split backend-free bootstrap planning/default policy from concrete launch/render adapters.
  Validation: `cargo tree -p fret-bootstrap --no-default-features -e normal --depth 4` does not contain `wgpu`, `winit`, `fret-render`, or platform-native runner crates unless an explicitly named feature is enabled.
  Evidence: `fret-bootstrap` tree output and tests for plan/default construction.
  Handoff: Keep `fret-bootstrap` as a composition module, not a second runtime.

- [ ] ASF-031 [owner=unassigned] [deps=ASF-030] [scope=ecosystem/fret-bootstrap,ecosystem/fret,apps/fretboard,docs]
  Goal: Migrate first-party app/template callers onto the new bootstrap/launch split and delete displaced helper aliases.
  Validation: focused `cargo check` for affected packages plus template tests if any are touched.
  Evidence: First-party callers use the target surface only.
  Handoff: Avoid carrying both old and new helper names in the default path.

## M3 — Public Facade Narrowing

- [ ] ASF-040 [owner=unassigned] [deps=ASF-020] [scope=ecosystem/fret/src/lib.rs,ecosystem/fret/src/view.rs,ecosystem/fret/tests,docs]
  Goal: Define and enforce the narrow `fret::app::prelude::*` Golden Path budget.
  Validation: surface tests assert the prelude contains only the approved app-authoring imports and excludes advanced/compatibility names.
  Evidence: Updated public surface tests and docs.
  Handoff: Names not in the budget should move to explicit modules or direct crates.

- [ ] ASF-041 [owner=unassigned] [deps=ASF-040] [scope=ecosystem/fret/src/view.rs,ecosystem/fret/src/actions.rs,ecosystem/fret/src/lib.rs]
  Goal: Split the large view/action authoring implementation into deeper owner modules without widening the public interface.
  Validation: existing `fret` tests plus targeted compile tests for `LocalState`, typed actions, selector/query reads, and advanced raw-model escape hatches.
  Evidence: Smaller owner modules with stable public re-exports.
  Handoff: This is a structure refactor; delete dead compatibility bridges as they are found.

## M4 — Ecosystem Taxonomy Closure

- [ ] ASF-050 [owner=unassigned] [deps=ASF-010] [scope=ecosystem/fret-ui-headless,ecosystem/fret-ui-kit,ecosystem/fret-authoring,docs/adr/0154-ecosystem-crate-taxonomy-glue-and-ui-kit-split-v1.md,docs/adr/IMPLEMENTATION_ALIGNMENT.md]
  Goal: Decide and land the headless/primitives/kit taxonomy for one representative primitive family.
  Validation: targeted tests for the chosen family and `python tools/check_layering.py`.
  Evidence: ADR alignment row updated with concrete shipped state.
  Handoff: If a new crate is needed, name it once and make re-export shims temporary or delete them.

- [ ] ASF-051 [owner=unassigned] [deps=ASF-050] [scope=ecosystem/fret-ui-kit,ecosystem/fret-ui-shadcn,ecosystem/fret-ui-material3]
  Goal: Migrate at least one recipe surface to consume the finalized primitive taxonomy directly instead of depending on broad kit compatibility shims.
  Validation: package tests for the recipe crate and no new backend deps in ecosystem crates.
  Evidence: Recipe imports demonstrate the target dependency path.
  Handoff: Prefer a vertical proof over moving every primitive at once.

## M5 — Shared Menu/Select Policy

- [ ] ASF-060 [owner=unassigned] [deps=ASF-050] [scope=ecosystem/fret-ui-kit,ecosystem/fret-ui-headless,ecosystem/fret-ui-shadcn/src/{select.rs,dropdown_menu.rs,context_menu.rs,menubar.rs}]
  Goal: Extract one shared menu/select interaction module covering a concrete behavior such as roving focus, typeahead, submenu grace intent, dismissal, or entry focus.
  Validation: targeted `fret-ui-shadcn` tests for select/dropdown/context menu parity, plus any new headless/primitive unit tests.
  Evidence: One behavior is tested once at the owner module and consumed by multiple recipe files.
  Handoff: Do not start by splitting files mechanically; extract behavior only where it creates locality.

- [ ] ASF-061 [owner=unassigned] [deps=ASF-060] [scope=docs/workstreams/menu-surfaces-alignment-v1,docs/adr/IMPLEMENTATION_ALIGNMENT.md]
  Goal: Record whether the first extraction proves a broader menu/select cleanup lane or should remain a narrow fix.
  Validation: updated workstream note or follow-on split.
  Evidence: documented decision with code/test anchors.
  Handoff: Split a separate menu/select workstream if the surface becomes the main project.

## M6 — Renderer Facade Decision

- [ ] ASF-070 [owner=unassigned] [deps=ASF-010] [scope=crates/fret-render,crates/fret-render-core,crates/fret-render-wgpu,docs/workstreams/renderer-modularity-fearless-refactor-v1,docs/adr/IMPLEMENTATION_ALIGNMENT.md]
  Goal: Decide whether `fret-render` is collapsed into `fret-render-wgpu` or deepened into the renderer interface.
  Validation: a short decision note plus one compile gate demonstrating the chosen profile.
  Evidence: updated renderer docs/workstream and Cargo feature behavior.
  Handoff: If deepening requires a renderer-specific design, split it out before implementation.

## M7 — Closeout

- [ ] ASF-080 [owner=planner] [deps=ASF-020,ASF-030,ASF-040,ASF-050,ASF-060,ASF-070] [scope=docs/workstreams/architecture-surface-fearless-refactor-v1]
  Goal: Close the lane or split remaining work into narrower follow-ons.
  Validation: `EVIDENCE_AND_GATES.md` contains final gate results and `WORKSTREAM.json` status is updated.
  Evidence: closeout audit or final status note.
  Handoff: Remaining open work must be owned by a narrower lane.
