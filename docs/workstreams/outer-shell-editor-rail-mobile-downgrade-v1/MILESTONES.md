# Outer-Shell Editor Rail Mobile Downgrade v1 — Milestones

Status: Closed closeout lane
Last updated: 2026-04-11

## M0 — Baseline

Exit criteria:

- The repo's current device-shell proof and editor-rail proof are read together.
- The lane states whether the remaining mobile question is about shell ownership or inner rail
  ownership.

Primary evidence:

- `M0_BASELINE_AUDIT_2026-04-11.md`
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
- `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`

Current status:

- Closed on 2026-04-11 via `M0_BASELINE_AUDIT_2026-04-11.md`.

## M1 — Verdict

Exit criteria:

- The lane names the correct mobile downgrade owner.
- The lane closes if the current answer is "outer shell owns it, no shared helper yet".

Primary evidence:

- `CLOSEOUT_AUDIT_2026-04-11.md`
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`

Current status:

- Closed on 2026-04-11.
- Verdict: editor-rail mobile downgrade composition remains outer-shell owned. No shared helper is
  justified yet beyond the existing generic device-shell strategy helper and explicit app-local
  branch composition.
