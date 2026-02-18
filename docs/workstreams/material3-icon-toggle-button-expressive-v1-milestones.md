# Material 3 Expressive Icon Toggle Button (v1) — Milestones

## Overview

The goal is a first-class, gateable, Expressive-aligned toggleable icon button surface in
`ecosystem/fret-ui-material3`, with a UI gallery demo and diagnostics evidence.

Prefer “policy in ecosystem, mechanism in core”:

- Keep Material decisions inside `fret-ui-material3`.
- Only propose `crates/fret-ui` changes if we cannot express the outcome using existing pressable +
  indication + semantics mechanisms.

## M0 — Parity audit + anchors (baseline)

**Exit criteria**

- Upstream references identified (Compose `IconToggleButtonShapes` + MUI ToggleButton).
- In-tree anchors listed (component, tokens, gallery surface, existing stability tests).
- Baseline diag script exists and captures Standard + Expressive states.

## M1 — Toggle contract surface (interactive behavior)

**Goal**

Make “toggleable icon button” a real component contract, not a styling hint.

**Exit criteria**

- A first-class `IconToggleButton` API exists (model-driven or callback-driven).
- Clicking updates checked state (or invokes onCheckedChange) deterministically.
- Keyboard activation parity is validated (Space/Enter).

## M2 — Expressive shape morph (checked + pressed)

**Goal**

Match Compose Expressive outcome: shape depends on (pressed, checked), with animation.

**Exit criteria**

- Checked shape is represented (tokens or derived fallback).
- Shape selection rule implemented: `pressed > checked > unchecked`.
- A stability gate exists (scene structure stable while pressed; geometry stabilizes after settle).

## M3 — A11y semantics locked (role + flags)

**Goal**

Pick a single portable semantics strategy and prevent drift.

**Exit criteria**

- Role/flags decision documented in the refactor plan.
- A headless semantics snapshot test asserts:
  - role,
  - checked/selected flags,
  - label/test_id behavior.

## M4 — UI gallery + diag gate (reviewable evidence)

**Goal**

Make the behavior reviewable without “human timing”, and regressions easy to catch.

**Exit criteria**

- UI gallery page contains interactive toggle demo with stable `test_id`s.
- Diag script updated to use `test_id` selectors.
- Script captures bundles + screenshots for both Standard and Expressive variants.

## M5 — Cleanup + adoption

**Exit criteria**

- Remove redundant `.toggle(true)` / `.selected(...)` patterns from docs/examples in favor of the
  new `IconToggleButton` surface.
- Document how to migrate existing callsites (if any).

