# Direction Infrastructure v1 - Design

Status: Closed
Last updated: 2026-05-30

## Problem

Fret already has `fret_core::LayoutDirection`, Radix-style direction providers in
`fret-ui-kit`, shadcn direction facades, Material3 layout direction context, and overlay placement
direction handling. The current gap is not a missing enum; it is duplicated direction policy across
component crates and no clear contract for which direction behavior belongs in mechanism,
primitive, or recipe layers.

## Source Stack

- Radix/shadcn: `DirectionProvider` + `useDirection`, defaulting to LTR.
- Base UI: `DirectionProvider` enables component behavior, while the app still sets HTML/CSS
  direction separately.
- Compose: `LayoutDirection` is a layout-local input and relative placement resolves physical
  coordinates from logical coordinates.
- Fret existing code:
  - `fret_core::LayoutDirection`
  - `fret-ui` overlay placement direction support
  - `fret-ui-kit::primitives::direction`
  - `fret-ui-kit::primitives::roving_focus_group`
  - `fret-ui-shadcn::direction`
  - Material3 foundation layout direction context

## Layering Decision

- `fret-core`: owns the portable `LayoutDirection` data type.
- `crates/fret-ui`: owns runtime/layout mechanisms that need direction to compute physical
  geometry, such as overlay placement and future Flex/logical-edge layout.
- `ecosystem/fret-ui-kit`: owns reusable headless/primitive direction policy helpers, such as
  Radix-like provider resolution, horizontal arrow semantics, and logical visual index helpers.
- `ecosystem/fret-ui-shadcn` and `ecosystem/fret-ui-material3`: own design-system facades and
  component-specific policy only.

## First Slice

Promote the duplicated horizontal ArrowLeft/ArrowRight RTL table and horizontal visual index helper
to `fret-ui-kit::primitives::direction`, then migrate representative shadcn and Material3 callsites
to use it.

This deliberately does not implement global Flex RTL layout. That is the next mechanism-layer
follow-on once the shared policy surface is stable.
