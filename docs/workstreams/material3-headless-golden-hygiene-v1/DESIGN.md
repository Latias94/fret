# Material3 Headless Golden Hygiene v1 Design

Status: Closed
Last updated: 2026-05-31

## Problem

`ecosystem/fret-ui-material3/tests/radio_alignment.rs` has grown beyond Radio alignment coverage.
The binary contains Radio geometry and ripple-origin checks, but it also hosts broad Material3
headless golden suites for controls, navigation, overlays, app bars, and other surfaces.

That shape turns a focused Radio gate into a god test. A stale navigation or overlay golden can fail
the default `radio_alignment` gate even when the Radio behavior under review is correct.

## Decision

Keep the existing broad headless suites available as explicit maintenance tests, but remove stale
navigation and overlay broad goldens from the default `radio_alignment` gate with `#[ignore]`
annotations that name the canonical focused gates.

The default ownership becomes:

- Radio-specific coverage stays in `radio_alignment`.
- Navigation default behavior coverage lives in `navigation_state` and related component state
  gates.
- Overlay default behavior coverage lives in `menu_state`, `dialog_state`, `tooltip_state`,
  `select_behavior`, and shared automation-surface gates.
- Broad headless golden refreshes are opt-in maintenance runs through ignored tests.

This is intentionally a gate-boundary refactor, not a golden refresh.

## Non-Goals

- Do not update stale expected golden payloads in this lane.
- Do not rewrite the full `radio_alignment` binary into fixture-driven files in one step.
- Do not hide failures from focused component behavior/state tests.

## Follow-On Shape

A future lane can split the broad suites out of `radio_alignment.rs` into dedicated fixture-driven
test modules. That work should preserve these boundaries:

- one owner per suite family;
- focused default state/behavior gates;
- opt-in broad golden refresh gates;
- JSON fixtures for large repeated matrix rows where practical.
