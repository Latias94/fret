# Material 3 Component Alignment Sweep v1

Status: Active
Last updated: 2026-05-27

## Why This Lane Exists

The previous Material 3 parity harness lane established the operating loop: suite manifest,
Button/Select/Switch packets, reusable test support, focused Select behavior tests, and stable
automation surfaces for Select and Switch.

This lane applies that loop across the whole `fret-ui-material3` component surface. The goal is not
to mechanically port every upstream implementation. The goal is to align every component against
Material outcomes, classify each finding by Fret layer, and refactor shared infrastructure when
multiple components prove the same foundation or policy drift.

## Core Strategy

Material alignment proceeds as a component sweep with foundation escalation:

1. Every component receives a row in the alignment matrix.
2. High-risk components become parity packets with source facts, Fret evidence, owner/layer
   classification, and gates.
3. If two or more components expose the same drift, the fix is considered for shared Material
   foundation or `fret-ui-kit` policy instead of duplicated recipe edits.
4. Mechanism work in `crates/*` is only allowed when a packet proves a real contract gap.
5. Low-interaction components can close with focused goldens/scene assertions when the risk is
   mostly visual and the evidence is stable.

## Source Precedence

Use axis-based precedence, not a single upstream winner:

- Material Design 3 spec: UX intent, taxonomy, state definitions, token direction, density, motion
  intent.
- Compose Material3: toolkit state machines, semantics, touch behavior, motion foundations, and
  renderer-agnostic interaction behavior.
- MUI Material UI: web composition, default props, popup/focus edge cases, and browser-facing
  overlay behavior.
- Base UI: headless accessibility parts and fallback semantics.
- shadcn/Radix in-tree work: Fret-side layering, stable `test_id` naming, and gate design only.

## Layer Ownership

- `crates/fret-ui`: mechanisms and hard contracts only.
- `ecosystem/fret-ui-kit`: design-system-agnostic interaction policy shared by more than one
  design system.
- `ecosystem/fret-ui-material3/src/foundation`: Material-wide tokens, motion, state layers, ripple,
  floating labels, active indicators, elevation, overlay motion, and touch-target helpers.
- `ecosystem/fret-ui-material3/src/<component>.rs`: Material recipe composition, intrinsic chrome,
  slot spacing, and stable component part IDs.
- `apps/fret-ui-gallery`: teaching surfaces and reproducible diagnostic entry points.
- `tools/parity-discovery`: packet joining, suite summaries, and agent repair/hardening queues.

## Component Sweep Waves

Wave order is risk-driven:

1. Evidence stabilization: classify known Material controls golden drift.
2. Navigation indicator packet: Tabs, NavigationBar, NavigationRail, and adjacent navigation
   surfaces.
3. Field-family foundation packet: TextField, Autocomplete, ExposedDropdown, SearchBar/SearchView,
   DatePicker, and TimePicker.
4. Overlay and feedback packet: Menu/DropdownMenu, Dialog, BottomSheet, Tooltip, and Snackbar.
5. Choice controls packet: Checkbox, Radio, Slider, SegmentedButton, chips, and related controls.
6. Surface/data-display audit: Badge, Card, CarouselItem, Divider, FAB, List, ProgressIndicator,
   TopAppBar, and other low-interaction surfaces.

Existing Button/Select/Switch packets remain the seed evidence and should be reused as consumer
anchors when foundation behavior changes.

## In Scope

- Add component parity packets and suite entries.
- Add or harden stable Material `test_id` surfaces.
- Add focused `fretboard diag` scripts when behavior cannot be proven headlessly.
- Split large tests only after evidence is stable.
- Remove stale, duplicated, or wrongly layered Material code when the packet evidence supports it.
- Refactor shared Material foundations when multiple component rows point to the same drift.

## Out Of Scope

- 1:1 source compatibility with Compose, MUI, Material Web, or Base UI.
- Moving Material policy into `crates/fret-ui` without packet evidence.
- Refreshing goldens without classifying whether the drift is real behavior, stale expectation, or
  test instability.
- Styling gallery containers as recipe defaults unless upstream owns that behavior in the component.

## Closeout Condition

This lane can close when:

- all Material components in the alignment matrix have a current classification,
- high-risk components have packet evidence or an explicitly split blocker,
- shared foundation refactors have at least two consumer anchors or a documented exception,
- stable automation surfaces exist for all packeted overlay/field/navigation components,
- stale duplicated recipe code has been removed or isolated behind a follow-on,
- targeted gates and suite regeneration pass,
- remaining work is split into narrow follow-ons instead of hidden in this broad lane.
