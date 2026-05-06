# P1 Closeout Audit - 2026-05-06

Status: P1 cleanup slice closed; lane remains active for P2/P3 sequencing.

## Shipped Outcome

P1 closed the stale teaching/import cleanup without widening public APIs:

- first-party IMUI teaching and proof surfaces route IMUI option/state types through the
  app-facing `fret::imui` facade;
- `fret-ui-editor::imui` remains a thin adapter over declarative editor controls;
- large `fret-ui-kit::imui` owner-split pressure is recorded, but implementation is deferred to a
  narrower follow-on;
- duplicate helper alias review found no current public helper alias worth deleting in this lane.

## No-Delete Verdict

Do not delete more IMUI helpers from this source-audit lane. The current evidence says:

- old duplicate names were already removed by earlier IMUI stack work;
- `imui::adapters` is a contract-only external-adapter seam, not a built-in wrapper family;
- `*_with_options(...)` helpers are canonical explicit-options entry points, not compatibility
  aliases.

## Remaining Work

Continue `imui-imgui-gap-closure-v1` for P2/P3 prioritization only. Split implementation-heavy
work into dedicated follow-ons with owner, repro, and gates.

Recommended next follow-on:

- `imui-debug-draw-owner-split-v1`
- owner: `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- rule: no public API widening or renaming
- gates: current IMUI source gates plus a focused debug-draw smoke/test gate.
