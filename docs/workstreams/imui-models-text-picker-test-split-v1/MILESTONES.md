# ImUi Models Text Picker Test Split v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Split Target

- [x] Pick the smallest reviewable slice from the test-architecture gap: completion/history picker
  tests inside `models_text.rs`.
- [x] Decide not to introduce fixtures in this slice because the tests are multi-frame procedural
  interactions.

## M1 - Landed Split

- [x] Move picker tests into `models_text_picker.rs`.
- [x] Keep shared harness access through `use super::*`.
- [x] Keep the original `models_text` behavior coverage intact.

## M2 - Closeout

- [x] Record the closed lane and evidence gates.
- [x] Update the IMUI gap audit, roadmap, todo tracker, and workstream catalog.
