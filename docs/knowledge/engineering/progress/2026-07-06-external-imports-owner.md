---
type: Work Progress
title: External imports owner helper
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/external-imports-owner
tags: fret,ui-framework,public-surface,external-imports,raw-model,owner
---

# Summary

The external texture import demos and platform external video import demos now share one private
`ExternalImportsModelOwner` helper for the visibility toggle model write.

This keeps the demos classified as low-level external import harnesses while removing duplicated
event-handler `models_mut().update(...)` calls from native texture, wasm texture, AVF video, and MF
video sources.

# Decision

Do not promote this helper into a public framework API. The raw model is still driver-owned
interoperability state tied to imported viewport rendering, not first-contact app state. The helper
is only a crate-private owner boundary so repeated writes stay named and auditable.

# Evidence

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test external_imports_surface external_imports_visibility_writes_stay_behind_owner_helper --no-fail-fast`
  failed because `external_imports_owner.rs` did not exist.
- The same test now requires all four external import demos to call
  `ExternalImportsModelOwner::toggle_surface(...)` and forbids the old direct update forms.
- `cargo check -p fret-examples --lib --tests` passes on the macOS-native path.
- `cargo check -p fret-examples --target wasm32-unknown-unknown --lib` passes on the wasm path.

# Next

Apply this pattern only to advanced/manual harnesses with repeated owner-shaped writes. Do not turn
every low-level model into `LocalState<T>` unless the demo is meant to teach default app authoring.
