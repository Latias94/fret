---
type: Work Progress
title: External texture imports owner helper
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/external-texture-owner
tags: fret,ui-framework,public-surface,external-texture,raw-model,owner
---

# Summary

The native and wasm external texture import demos now share one private
`ExternalTextureImportsModelOwner` helper for the visibility toggle model write.

This keeps the demos classified as low-level external import harnesses while removing duplicated
event-handler `models_mut().update(...)` calls from both target-specific files.

# Decision

Do not promote this helper into a public framework API. The raw model is still driver-owned
interoperability state tied to imported viewport rendering, not first-contact app state. The helper
is only a crate-private owner boundary so repeated writes stay named and auditable.

# Evidence

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test external_texture_imports_surface external_texture_imports_visibility_writes_stay_behind_owner_helper --no-fail-fast`
  failed because `external_texture_imports_owner.rs` did not exist.
- The same test now requires native and web event handlers to call
  `ExternalTextureImportsModelOwner::toggle_surface(...)` and forbids the old direct update forms.
- `cargo check -p fret-examples --lib --tests` passes.
- `cargo check -p fret-examples --target wasm32-unknown-unknown --lib` passes.

# Next

Apply this pattern only to advanced/manual harnesses with repeated owner-shaped writes. Do not turn
every low-level model into `LocalState<T>` unless the demo is meant to teach default app authoring.
