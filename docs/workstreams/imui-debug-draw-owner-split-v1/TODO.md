# ImUi Debug Draw Owner Split v1 - TODO

Status: active
Last updated: 2026-05-06

## Current slice

- [x] Create a narrow follow-on lane from the P1 gap-closure closeout.
- [x] Record the baseline assumptions, repro, gates, and no-public-API rule.
- [x] Split the command model and source-level metadata into `debug_draw_controls/commands.rs`.
- [x] Run the focused debug draw compile/test floor.
- [x] Add a short M1 status note after the command-model slice lands.
- [x] Run the remaining source-policy, catalog, format-check, and diff-check gates.
- [x] Split paint command dispatch into `debug_draw_controls/paint.rs`.
- [x] Add a short M2 status note after the paint-dispatch slice lands.
- [x] Split path and shape sampling into `debug_draw_controls/paths.rs`.
- [x] Add a short M3 status note after the path-slice lands.

## Next slices

- [x] Split the remaining geometry / image / mesh helpers into `geometry.rs` and `paint.rs`
      while keeping public debug draw names stable.
- [ ] Decide whether private tests should remain colocated or move into owner-specific test modules.

## Guardrails

- [ ] Keep all public `fret-ui-kit::imui` debug draw names stable.
- [ ] Keep `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs` compiling without path changes.
- [ ] Keep additive draw-list capability work out of this lane.
