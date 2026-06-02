# Material3 Interaction Regression Harness Split v1 Design

Status: Closed
Last updated: 2026-05-31

## Problem

After moving broad headless goldens out of `radio_alignment.rs`, the file still contained dozens of
non-Radio interaction regressions for Snackbar, Navigation, TimePicker, Switch, Checkbox, Menu,
Dialog, Tooltip, Autocomplete, ExposedDropdown, ChipSet, and other surfaces.

That kept Radio ownership blurred even though the broad golden harness had been split.

## Decision

Move historical non-Radio interaction regressions into
`ecosystem/fret-ui-material3/tests/material3_interaction_regressions.rs`.

Keep `radio_alignment.rs` focused on the three Radio-owned tests:

- selected dot geometry;
- pointer-origin ripple geometry;
- pressed scene structure stability.

This lane intentionally creates one intermediate interaction-regression owner file instead of
moving every test directly to a family file. The next split can now happen from a correctly named
surface without continuing to misuse Radio ownership.

## Non-Goals

- Do not rewrite the moved tests or change behavior.
- Do not split the interaction regression file into all family-owned files in this lane.
- Do not move the plain TextInput regression across crate boundaries in this lane.

## Follow-On Shape

Future work should split `material3_interaction_regressions.rs` by component family where an
existing state/behavior file already owns the surface. The plain TextInput regression should be
audited separately because it may belong in `fret-ui` mechanism coverage rather than Material3.
