# Material 3 Switch Diagnostics Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The component matrix still marked Switch as `packet_done_known_follow_ons`, even though the
Material3 parity harness already had a Switch adapter report with no mismatches. This packet adds
fresh gallery diagnostics and focused Rust gates so the matrix can distinguish "seed packet" from
current verified closure.

## Truth

- Switch recipe owns track/handle/icon composition, selected-state animation, and stable part ids.
- Material foundation owns shared state-layer/ripple and minimum interactive target sizing.
- The UI Gallery should expose default, disabled, icons-both, and selected-icon-only variants with
  stable root/chrome/track/handle/icon selectors.
- No kit-policy or mechanism change is justified by the current evidence.

## Boundaries

- Do not move Switch animation into shared kit policy in this packet.
- Do not change component code unless fresh diagnostics show drift.
- Treat pixel-level upstream Material Web motion comparison as future hardening, not a blocker for
  the current Fret evidence closure.

## Reference Axis

- Material Web and Material spec for icon/chrome/motion vocabulary.
- Compose Material3 for switch semantics, checked state, and touch target behavior.
- Fret Material3 parity harness report for already measured switch parts.
- Fret UI Gallery diagnostics for current runtime evidence.
