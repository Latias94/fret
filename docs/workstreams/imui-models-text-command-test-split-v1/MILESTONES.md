# ImUi Models Text Command Test Split v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Split Target

- [x] Pick the next smallest reviewable slice from `models_text.rs`: single-line command-policy
  tests.
- [x] Keep behavior coverage identical while reducing the old module's command-policy surface.

## M1 - Landed Split

- [x] Move command tests into `models_text_commands.rs`.
- [x] Keep shared test harness access through `use super::*`.
- [x] Keep the original `models_text` filter green.

## M2 - Closeout

- [x] Record the closed lane and evidence gates.
- [x] Update the IMUI gap audit, roadmap, todo tracker, and workstream catalog.
