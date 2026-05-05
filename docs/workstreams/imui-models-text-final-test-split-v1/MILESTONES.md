# ImUi Models Text Final Test Split v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Split Target

- [x] Confirm the only tests left in `models_text.rs` are basic changed-signal, lifecycle/bounds,
  and push-id identity stability.
- [x] Keep behavior coverage identical while removing the aggregate file.

## M1 - Landed Split

- [x] Move basic coverage into `models_text_basic.rs`.
- [x] Move lifecycle and bounds coverage into `models_text_lifecycle.rs`.
- [x] Move push-id reorder coverage into `models_text_identity.rs`.
- [x] Delete `models_text.rs`.
- [x] Keep the original `models_text` filter green.

## M2 - Closeout

- [x] Record the closed lane and evidence gates.
- [x] Update the IMUI gap audit, roadmap, todo tracker, and workstream catalog.
