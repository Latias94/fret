---
type: Work Progress
title: Hotpatch smoke demo model-owner tightening
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/hotpatch-smoke-model-owner
tags: fret,demo,hotpatch,model-owner,public-surface
---

# Summary

`hotpatch_smoke_demo.rs` remains a dev-only maintainer smoke harness, but its event and command
paths no longer write models directly through `app.models_mut().update(...)`.

# Changes

- Added `HotpatchSmokeModelOwner`.
- Routed counter increment writes through `HotpatchSmokeModelOwner::increment_counter(...)`.
- Routed debug status writes through `HotpatchSmokeModelOwner::set_debug(...)`.
- Added a source-surface regression test that forbids direct update bypasses in the demo source.

# Verification

- `cargo nextest run -p fret-examples hotpatch_smoke_demo_routes_model_writes_through_owner --no-fail-fast`
- `cargo check -p fret-demo --bin hotpatch_smoke_demo --features hotpatch`
