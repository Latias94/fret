# Todo Environment/Responsive Lane Freeze Audit — 2026-04-16

Status: Frozen follow-on for the Todo-surfaced render-gap classification

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_FACING_RENDER_GAP_AUDIT_2026-04-03.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/crate-usage-guide.md`
- `apps/fret-examples/src/todo_demo.rs`

## Scope

Close the remaining open sub-question from the Todo-derived render-gap audit:

- when ordinary app code needs responsive/device-shell reads, is that still a missing app-facing
  lane, or is the real fix to complete the explicit `fret::env::{...}` surface?

This note does not reopen recipe promotion, adaptive strategy helpers, or default-prelude growth.

## Assumptions-first checkpoint

1. `fret::env::{...}` is already the documented app/component-facing lane for low-level adaptive
   reads.
   Confidence: Confident.
   Evidence: `docs/crate-usage-guide.md`, `docs/component-author-guide.md`,
   `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`.
2. `todo_demo` is still the canonical first-party proof for the ordinary app lane, so any direct
   `fret_ui_kit::declarative` dependency there should be treated as a façade gap until proven
   otherwise.
   Confidence: Confident.
   Evidence: `apps/fret-examples/src/todo_demo.rs`, `apps/fret-examples/src/lib.rs`,
   `docs/examples/todo-app-golden-path.md`.
3. Query hysteresis/configuration nouns belong with low-level environment queries, not with the
   default prelude and not with the raw component/internal `ElementContext` lane.
   Confidence: Likely.
   Evidence: `ecosystem/fret-ui-kit/src/declarative/mod.rs`,
   `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`.

## Findings

### 1) The remaining Todo pressure was a façade gap, not a reason to widen the default lane

Before this follow-on, `todo_demo` still had to import:

- `ViewportQueryHysteresis`
- `viewport_width_at_least(...)`
- `primary_pointer_can_hover(...)`
- `viewport_tailwind`

from `fret_ui_kit::declarative` directly, even though the public docs already taught
`fret::env::{...}` as the explicit secondary lane for those reads.

That mismatch meant the current repo posture was internally inconsistent:

- the docs said “explicit env lane,”
- while the first-party Todo proof still demonstrated a direct kit import to finish the job.

### 2) The missing piece was query-configuration coverage on `fret::env`

The real blocker was narrower than “responsive helpers are missing from `fret`.”

`fret::env` already re-exported the query functions, but it did not expose the corresponding
configuration nouns needed by ordinary app code:

- `ContainerQueryHysteresis`
- `ViewportQueryHysteresis`
- `ViewportOrientation`

Without those nouns, the documented explicit lane was incomplete for real-world app code.

### 3) The correct fix is to complete the explicit lane, not to relabel it as raw debt

The post-fix posture is now:

- low-level environment/responsive reads stay off `fret::app::prelude::*`,
- they stay on explicit `fret::env::{...}` imports,
- and query-configuration nouns stay on that same explicit lane.

This means the Todo-derived classification is now stable:

- keep raw style escape hatches explicit,
- keep environment/responsive helpers explicit but non-default,
- and reserve app-facing render-sugar follow-ons for ordinary helper authoring gaps rather than
  for adaptive query imports.

## Evidence

- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `docs/crate-usage-guide.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `cargo nextest run -p fret -E 'test(root_surface_exposes_explicit_env_module) or test(crate_usage_guide_keeps_query_guidance_on_grouped_app_surfaces)'`
- `cargo nextest run -p fret-examples -E 'test(todo_demo_prefers_default_app_surface)'`
- `cargo check -p fret -p fret-examples --all-targets`

## Outcome

The Todo-surfaced environment/responsive classification is now frozen:

1. `fret::env::{...}` is the explicit app/component-facing lane for low-level adaptive reads.
2. `ContainerQueryHysteresis`, `ViewportQueryHysteresis`, and `ViewportOrientation` belong on that
   same explicit lane.
3. Direct `fret_ui_kit::declarative` imports for those ordinary app-facing reads are no longer the
   taught first-party proof posture.
