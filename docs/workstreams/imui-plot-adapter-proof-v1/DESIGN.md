# IMUI Plot Adapter Proof v1 - Design

Status: Closed
Last updated: 2026-05-25

## Problem

EWG-060 re-evaluates ListBox, plot adapter, style/theme editor, and porting sugar only after the
canonical workbench exposes repeated friction. Plotting is the only candidate with enough current
pressure because `fret-plot` already has mature declarative panels and tests, while the gap catalog
already names an IMUI plot adapter as a narrow follow-on candidate.

The previous retained-backed public plot facade was intentionally deleted. This lane must not undo
that closeout.

## Target

- `fret-plot/imui` is an optional feature.
- `fret_plot::imui::*_plot_panel` only delegates to `crate::declarative::*_plot_panel`.
- `fret-imui` stays a thin facade and must not depend on `fret-plot`.
- `fret-ui-kit::imui` must not depend on `fret-plot`.
- The old retained plot facade remains deleted.

## Owning Layer

- `ecosystem/fret-plot`: owns plot data/model/style/declarative panel APIs and the optional
  `UiWriter` adapter.
- `ecosystem/fret-authoring`: owns the tiny shared `UiWriter` contract.
- `ecosystem/fret-imui`: remains a frontend facade over authoring/runtime contracts, not a plot
  component collection.
- `ecosystem/fret-ui-kit::imui`: remains the policy-heavy common IMUI kit, not a domain-plot crate.

## Non-Goals

- Add plot helpers to `fret-imui`.
- Add plot dependencies to `fret-ui-kit`.
- Recreate an ImPlot clone in this lane.
- Restore `LinePlotCanvas` or any retained plot bridge.
- Add a style/theme editor, ListBox helper, or porting-sugar API here.

## Proof Surfaces

1. Existing declarative plot panels and plot tests in `ecosystem/fret-plot/src/declarative.rs`.
2. The new optional `fret-plot/imui` adapter compile gate plus the source-policy gate proving the
   adapter is opt-in and declarative-only.

This is enough for the first slice because the adapter is intentionally thin over an existing
declarative control family. A future cookbook or workbench adoption can happen only if the
canonical route shows repeated authoring friction.
