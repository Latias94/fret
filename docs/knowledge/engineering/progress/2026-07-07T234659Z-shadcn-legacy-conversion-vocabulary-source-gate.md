---
type: "Work Progress"
title: "Shadcn legacy conversion vocabulary source gate"
description: "Work Progress for Shadcn legacy conversion vocabulary source gate."
timestamp: 2026-07-07T23:46:59Z
tags: ["fret", "shadcn", "public-surface", "conversion", "source-policy"]
---

# Summary

Added a source-policy regression gate in `fret-ui-shadcn` that scans the crate `src/` tree and
rejects legacy conversion vocabulary outside the policy test itself.

# Details

- Changed `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`.
- The new `crate_source_tree_avoids_legacy_conversion_vocabulary` test reuses the existing
  `visit_rust_files(...)` helper and fails on `UiIntoElement`, `UiChildIntoElement`,
  `UiHostBoundIntoElement`, or `UiBuilderHostBoundIntoElementExt`.
- This hardens the `docs/shadcn-declarative-progress.md` claim that first-party shadcn surfaces
  should stay on `IntoUiElement<H>` or app-facing `Ui` / `UiChild` aliases instead of the deleted
  split conversion traits.
- Focused verification passed:
  `cargo nextest run -p fret-ui-shadcn crate_source_tree_avoids_legacy_conversion_vocabulary`.

# Next Action

Run the standard surface/boundary/format/wiki gates, then commit and push `main` if clean.

# Citations

- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`
- `docs/shadcn-declarative-progress.md`
