---
type: Work Progress
title: Examples embedded viewport owner helper
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/examples-embedded-viewport-owner
tags: fret,ui-framework,public-surface,examples,embedded-viewport,raw-model,owner
---

# Summary

`apps/fret-examples/src/embedded_viewport_demo.rs` now routes its startup readout write through a
demo-local `EmbeddedViewportDemoModelOwner`.

The demo remains a Tier A embedded viewport interop surface because it owns
`EmbeddedViewportSurface`, explicit viewport hooks, and offscreen render target recording. The
cleanup only removes the copyable pattern of writing `last_input` directly from `View::init(...)`.

# Decision

Keep the advanced interop classification. This demo teaches the embedded viewport mechanism, not
ordinary app state authoring. The local owner helper keeps raw `ModelStore` mutation named and
auditable while avoiding a premature public embedded-viewport app binding.

# Evidence

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test embedded_viewport_demo_surface embedded_viewport_demo_model_writes_stay_behind_owner_helper --no-fail-fast`
  failed because `EmbeddedViewportDemoModelOwner` did not exist.
- The same test now requires `EmbeddedViewportDemoModelOwner::set_last_input(...)` and forbids
  direct `models_mut().update(...)` calls in the demo source.
- `cargo nextest run -p fret-examples --test embedded_viewport_demo_surface --no-fail-fast`
  passes.

# Next

Continue shrinking first-contact raw model pressure in `apps/fret-examples/src` by choosing either
the table stress demo owner boundary or a dedicated IMUI editor proof state-contract lane.
