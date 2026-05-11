---
title: Shadcn Parity Discovery Harness v2 Design
status: active
date: 2026-05-11
---

# Design

## Scope

This lane expands the completed v1 shadcn parity discovery harness into a coverage-driven sweep.
Its job is to keep the existing locked surfaces stable and add new discovery coverage where the
highest-risk overlay, navigation, and responsive combinations are still uncovered.

## Owns

- Coverage manifest design and ordering.
- Fixture and diagnostics-script expansion for uncovered surfaces.
- Layer classification for every new non-passing result.
- Promotion of confirmed findings into reusable regression gates.

## Does Not Own

- Core UI mechanism redesign unless a discovered gap proves it belongs in `crates/fret-ui`.
- Recipe polish that is already locked by v1 unless a v2 sweep exposes a regression.
- Broad component-library growth without a concrete discovery target.

## Source Precedence

1. Upstream shadcn docs and registry source.
2. UI Gallery docs-path snippets and diagnostics scripts.
3. Fret layout sidecars and existing regression gates.
4. Generated parity reports and workstream notes.

## First Sweep Targets

The manifest starts with locked rows for the v1 coverage anchors:

- Select open
- Combobox open desktop/mobile
- Popover command shell
- Drawer bottom sheet
- Calendar Hijri

It then prioritizes new high-risk overlay surfaces:

- Context Menu
- Navigation Menu
- Hover Card
- Tooltip
- Dialog
- Sheet

