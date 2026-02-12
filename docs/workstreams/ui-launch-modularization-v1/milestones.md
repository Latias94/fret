# UI + Launch Modularization v1 — Milestones

Status: Draft

Milestones are landable checkpoints (each should be buildable + gated).

## M0 — Plan agreed + gates green

Exit criteria:

- Workstream docs reviewed (README + TODO + milestones).
- Minimal gates confirmed runnable:
  - `python tools/check_layering.py`
  - `cargo fmt`
  - `cargo nextest run -p fret-ui`

## M1 — `fret-ui` tree split (structural)

Exit criteria:

- `crates/fret-ui/src/tree/mod.rs` is “thin”.
- `tree/*` responsibilities separated (state/mount/dispatch/layout/diag).
- Gates:
  - `python tools/check_layering.py`
  - `cargo nextest run -p fret-ui`

## M2 — `fret-ui` elements/context split (follow-up)

Exit criteria:

- Reduce coupling in `elements/*` between “mechanism contracts” and “compat/diagnostics”.
- Tighten visibility; avoid new public surface creep.
- Gates green for touched crates.

## M3 — `fret-launch` desktop runner split (structural)

Exit criteria:

- Desktop runner wiring split into focused modules (lifecycle/render/effects/diag).
- `cargo nextest run -p fret-launch` is green.
- No new dependencies introduced.

## M4 — `fret-launch` web runner split (structural)

Exit criteria:

- Web runner wiring split similarly (loop/effects/streaming).
- Gates green; no intentional behavior changes.

## M5 — Postmortem + Track C decision

Exit criteria:

- Identify remaining seams and decide Track C (if any):
  - A: keep as modules only
  - B: feature-isolate only
  - C: split `fret-launch` by topology (`desktop` vs `web`)

