# M0 Baseline Audit — 2026-04-11

Status: assumptions-first baseline for the active lane

## Assumptions

1. **Shared adaptive classification is already shipped.**
   - Confidence: Confident
   - Evidence:
     - `ecosystem/fret-ui-kit/src/adaptive.rs`
     - `ecosystem/fret/src/lib.rs`
     - `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
   - Consequence if wrong:
     - this lane would be premature because the missing problem would still be lower-level helper
       ownership, not upper-interface presentation design.

2. **`Sidebar` is intentionally frozen as an app-shell surface, not the editor/panel answer.**
   - Confidence: Confident
   - Evidence:
     - `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
     - `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`
     - `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
   - Consequence if wrong:
     - any new upper-interface guidance that keeps sidebar bounded would be invalid.

3. **The responsive dialog proof is intentionally explicit rather than already being a wrapper candidate.**
   - Confidence: Confident
   - Evidence:
     - `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
     - `apps/fret-ui-gallery/tests/device_shell_recipe_wrapper_surface.rs`
     - `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
   - Consequence if wrong:
     - this lane should immediately turn into a narrower dialog/drawer wrapper extraction lane.

4. **Editor rail downgrade remains an outer-shell concern.**
   - Confidence: Confident
   - Evidence:
     - `docs/workstreams/outer-shell-editor-rail-mobile-downgrade-v1/CLOSEOUT_AUDIT_2026-04-11.md`
     - `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
     - `apps/fret-examples/src/workspace_shell_demo.rs`
   - Consequence if wrong:
     - the lane would need to reopen editor-rail helper extraction, which is out of scope here.

5. **The current missing piece is synthesis, not another mechanism.**
   - Confidence: Likely
   - Evidence:
     - `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
     - `docs/workstreams/device-shell-recipe-wrapper-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
     - `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
   - Consequence if wrong:
     - we would need to pause and prove a concrete missing API instead of documenting the
       upper-interface owner split first.

6. **A generic cross-family helper is still not justified by current evidence.**
   - Confidence: Likely
   - Evidence:
     - `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
     - `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
     - `docs/workstreams/device-shell-recipe-wrapper-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
   - Consequence if wrong:
     - this lane should narrow immediately to the specific family where repetition is already
       reviewable.

## Current repo snapshot

- The repo already distinguishes:
  - low-level facts in `fret::env`,
  - shared classification in `fret::adaptive`,
  - family-local wrappers in recipe crates,
  - and outer-shell mobile downgrade for editor rails.
- The repo does **not** yet provide one first-open document for:
  - when same-feature different-presentation branching should stay explicit,
  - when a wrapper is allowed,
  - and when a surface must stay out of the app-shell recipe story.

## Immediate conclusion

The next correct step is a narrow documentation/design follow-on:

- freeze the upper-interface owner split,
- record the helper-extraction threshold,
- and leave current source gates in place rather than forcing a premature implementation change.
