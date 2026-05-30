# Material 3 ChipSet Roving Packet v1

Date: 2026-05-28
Status: Closed

## Problem

The component matrix kept `ChipSet` as a known follow-on because early inventory marked grouping
behavior as low direct evidence. Later work added two pieces of direct proof: the chip visual
diagnostics packet covers the gallery state-matrix surface, and the focused Rust roving test proves
that a chip trailing action can take focus without breaking ChipSet roving.

## Target State

- ChipSet exposes a stable root selector.
- ChipSet uses group semantics for a set of related chips.
- ArrowLeft/ArrowRight and Home/End roving remain stable and RTL-aware.
- Multi-action chips keep their internal primary/trailing focus handoff without confusing the
  parent roving container.
- The policy stays recipe-owned until another design-system consumer proves a kit abstraction is
  needed.

## Source Truth

- Compose Material3 chip samples use normal layout containers such as `Row` and `FlowRow` around
  chips; there is no upstream Compose `ChipSet` container API to port directly.
- Base UI `ToolbarRoot` and `CompositeRoot` are supporting headless references for composite roving
  containers and keyboard focus management.
- Fret `ChipSet` is therefore a Material3 ecosystem recipe convenience layer, not a core mechanism.

## Layer Ownership

- `ecosystem/fret-ui-material3/src/chip_set.rs`: group semantics, gap/wrap defaults, roving
  navigation, RTL-aware arrow mapping, loop behavior, and root selector.
- Individual chip recipe files: chip semantics, chrome, trailing actions, and roving tab-stop
  delegation.
- `fret-ui` roving primitives: existing mechanism substrate only.
- `fret-ui-kit`: no extraction in this packet; future follow-on only if another design system needs
  the same reusable container policy.
- diagnostics/test harness: selector, gallery visual, and roving handoff gates.

## In Scope

- Close the ChipSet matrix residual.
- Record the source, recipe, diagnostics, and test-harness evidence.
- Keep the current boundary explicit.

## Out Of Scope

- New ChipSet public API.
- Moving roving policy into `fret-ui-kit`.
- New `crates/*` focus or semantics mechanisms.
- Pixel-perfect chip group visual comparison against upstream screenshots.

## Closeout Condition

This lane is complete once the dedicated packet exists, the component matrix row points at it, and
the focused selector and roving gates pass.
