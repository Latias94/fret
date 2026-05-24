# UI Overlay Focus Dismissal Oracle v1

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

The runtime focus/overlay refactor shipped the key mechanism fixes: snapshot-backed dispatch,
outside-press containment, prevent-default focus clearing, hover rerender gates, and several
diagnostic scripts. The remaining risk is policy drift. Dialog, popover, dropdown, context menu,
hovercard, and docking interactions should share one oracle vocabulary for dismissal and focus
outcomes.

## Target State

- Fixture-backed oracle cases describe outside press, focus outside, escape, focus restore,
  nested modal scopes, and modal barrier behavior.
- Runtime mechanism tests and ecosystem policy tests can consume the same expected outcomes.
- Policy remains in `ecosystem/*`; `crates/fret-ui` provides mechanism hooks and diagnostics only.

## First Slice

Pick one existing behavior family, likely non-modal outside press focus handling, and express it as
a small fixture/oracle that can drive both a unit test and a diag script expectation.

## Non-goals

- Rewriting the overlay stack.
- Moving Radix dismissal policy into `crates/fret-ui`.
- Replacing existing diag scripts before the oracle proves one family.
