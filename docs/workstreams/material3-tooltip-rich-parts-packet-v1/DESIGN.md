# Material 3 Tooltip Rich Parts Packet v1 - Design

Status: Active
Last updated: 2026-05-28

## Problem

The closed Material 3 component sweep left two tooltip follow-ons:

- Rich tooltip actions may need interactivity, but Fret tooltip overlays are currently pointer
  transparent by contract.
- Plain and rich tooltip `test_id`/chrome wiring should be de-duplicated after selector behavior is
  locked.

Compose Material3 rich tooltips expose distinct title, supporting text, and optional action content.
Fret's `RichTooltip` currently exposes only the tooltip root and chrome selectors, which makes
automation unable to target the rich text parts without brittle structure assumptions.

## Target State

- `RichTooltip` exposes stable dotted part IDs for title and supporting text where those parts are
  present.
- `PlainTooltip` and `RichTooltip` use a shared Material recipe helper for root/chrome semantics
  wiring.
- Tooltip provider delay, safe-hover, pointer tracking, Escape close, and click-through overlay
  behavior remain in `fret-ui-kit`.
- Rich tooltip action interactivity remains explicitly split until a mechanism/ADR-backed overlay
  contract change is justified.

## Truth Set

- Truth 1: A rich tooltip with a title exposes `tooltip`, `tooltip.chrome`, `tooltip.title`, and
  `tooltip.supporting-text` selectors after hover open.
- Truth 2: A rich tooltip without a title exposes `tooltip`, `tooltip.chrome`, and
  `tooltip.supporting-text`, but not a fake `tooltip.title`.
- Truth 3: Plain tooltip selector behavior remains unchanged.
- Truth 4: Tooltip overlays remain pointer transparent; recipe selectors do not imply interactive
  rich tooltip action support.

## Layer Mapping

- `ecosystem/fret-ui-material3/src/tooltip.rs`: Material recipe owns tooltip chrome, text parts,
  token application, and stable `test_id` surfaces.
- `ecosystem/fret-ui-kit`: owns tooltip delay-group, safe-hover, pointer transit, overlay request,
  and pointer transparency.
- `crates/*`: no mechanism change in this lane.

## Non-Goals

- Do not add rich tooltip action APIs in this slice.
- Do not change `OverlayKind::Tooltip` hit-testing or underlay blocking.
- Do not add a generic scene/overlay mechanism for interactive tooltip content without ADR
  evidence.

## Upstream References

- Compose Material3 `Tooltip.kt`: `RichTooltip(title, action, text)` and focusable/action notes.
- Base UI Tooltip: popup/positioner/root are headless tooltip parts, not rich Material content
  taxonomy.
- Existing Fret evidence: `material3_overlay_feedback_packet_v1.md`.
