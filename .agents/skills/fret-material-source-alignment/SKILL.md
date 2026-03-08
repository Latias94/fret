---
name: fret-material-source-alignment
description: "This skill should be used when the user asks to \"align Material 3 components\", \"port Material 3 (Expressive)\", \"match MUI/Compose Material behavior\", or \"build a Material design-system layer\" for Fret. It provides a source-of-truth workflow (Material spec + MUI + Compose Material3 + Base UI) that maps changes to the correct Fret layer and locks outcomes with targeted tests and `fretboard diag` scripted repros."
---

# Material 3 (Expressive) source alignment

## When to use

- A Material-ish component does not match expected Material 3 behavior (states, focus, dismiss, keyboard nav, motion, density, or field choreography).
- You want to port a Material 3 (or “Material 3 Expressive”) component recipe into Fret without leaking policy into `crates/*`.
- You need a repeatable parity workflow: upstream reference → Fret layer mapping → regression gate.
- The task touches Material-specific foundations such as state layers, ripple, floating labels, motion schemes, active indicators, or touch-target sizing.

## Choose this vs adjacent skills

- Use this skill when the goal is **Material parity** (spec-driven outcomes) plus a regression gate.
- Use `fret-shadcn-source-alignment` for shadcn/ui v4 + Radix parity work.
- Use `fret-app-ui-builder` when the goal is “ship a good UI” (not strict upstream parity).
- Use `fret-diag-workflow` when the primary deliverable is a scripted repro/bundle (parity work is secondary).
- Use `fret-ui-review` when the request is an audit of an app UI’s layering/UX hygiene.
- If a mature in-tree shadcn component already solved the same mechanism split (for example Select/listbox overlays), use this skill **together with** `fret-shadcn-source-alignment`: shadcn is not the visual truth for Material, but it is a strong Fret-side exemplar for layering, `test_id` stamping, and gate design.

## Inputs to collect (ask the user)

- Which component(s)? (for example button, checkbox/switch, text field, menu, select, exposed dropdown, autocomplete, dialog, snackbar, tabs, navigation drawer)
- What mismatch class?
  - interaction policy (dismiss/focus restore/keyboard nav)
  - layout/density/sizing
  - tokens (colors/typography/shape/elevation/state layers)
  - motion (durations/easing/springs, interruption rules)
  - accessibility semantics (roles, names, focus order, active-descendant, relations)
- Which Material family is it closest to?
  - field family (`TextField`, `Select`, `Autocomplete`, exposed dropdown)
  - choice controls (`Checkbox`, `Radio`, `Switch`, chips)
  - navigation (`Tabs`, navigation bar/rail/drawer)
  - overlays/feedback (`Menu`, `Dialog`, `BottomSheet`, `Snackbar`, `Tooltip`)
- Which interaction substrate is likely involved?
  - state layer / ripple
  - floating label / active indicator / outline
  - overlay placement / collision / focus restore
  - motion scheme / spring timing
  - touch target / hit-testing
- What is the upstream source of truth for this case?
  - Material 3 guidelines/spec, MUI behavior, Compose Material3 behavior, and/or Base UI headless patterns
- Which Fret layer should own the change (mechanism vs shared policy/foundation vs recipe)?
- What regression protection is required: unit test, parity harness case, and/or diag script?
- Do we need stable `test_id` targets for automation?

Defaults if unclear:

- Treat Material spec as the primary UX truth.
- Treat Compose Material3 as the primary reference for non-DOM state machines, semantics, and motion/touch behavior.
- Treat MUI Material UI as the primary reference for web-facing defaults and composition details.
- Treat Base UI as an additional headless reference for accessibility-first part composition.
- When a mature in-tree shadcn component already solved the same Fret-side mechanism split, use it as an implementation exemplar for layering and gates, not as the visual/taxonomy truth.
- Treat policy/state machines as `ecosystem/*` unless it is a true mechanism/contract.
- Add at least one gate (test or diag script) for any interaction/motion change.

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin components_gallery`

## Quick start

1. Decide the source-of-truth ordering before touching code.
2. Inspect one mature in-tree exemplar before inventing a new parity workflow.
3. Read the two reference notes listed below and map the mismatch to the correct Fret layer.
4. Land a gate (test and/or `tools/diag-scripts/*.json`) with stable `test_id`.

## Workflow

### 0) Read the two reference notes first

Use these notes to keep the main workflow lean:

- Reference stack + in-tree exemplars:
  - `.agents/skills/fret-material-source-alignment/references/material-reference-stack.md`
- Field-family high-ROI checklist:
  - `.agents/skills/fret-material-source-alignment/references/material-field-family-checklist.md`

When the component belongs to the field family (`TextField`, `Select`, `Autocomplete`, `ExposedDropdown`, `DatePicker`, `TimePicker`), read both before coding.

### 1) Pick the upstream reference stack explicitly

Use the right source for the right kind of parity:

- **Material Design 3 spec/guidelines**: visual intent, tokens, density, motion intent, interaction outcomes.
- **Compose Material3**: non-DOM state machines, semantics, touch targets, motion/foundation patterns.
- **MUI Material UI**: web defaults, field/menu/select composition, portal/dismiss/focus edge cases.
- **Base UI**: accessibility-first headless part composition.

If sources disagree, document the chosen ordering in the change.

### 2) Map the work to the right Fret layer (non-negotiable)

- `crates/fret-ui` (and other `crates/*`): mechanisms/contracts (focus primitives, semantics routing, overlay roots, layout and hit-testing primitives, renderer-facing scene contracts).
- `ecosystem/fret-ui-kit`: design-system-agnostic headless policy + reusable infra (state machines, roving/typeahead, overlay dismiss/focus restore rules, motion helpers) when multiple design systems should share the behavior.
- `ecosystem/fret-ui-material3/src/foundation`: Material-wide infrastructure that must stay consistent across many components.
  - `foundation::token_resolver` / `tokens/*`
  - `foundation::motion_scheme`
  - `foundation::floating_label`
  - `foundation::overlay_motion`
  - `foundation::elevation`
  - `foundation::interactive_size`
  - `foundation::indication`
  - `interaction::state_layer`
  - `interaction::ripple`
- `ecosystem/fret-ui-material3/src/<component>.rs`: component recipes (composition + styling + stable `test_id` surfaces).

If the mismatch is “interaction policy” (dismiss rules, focus restore, keyboard nav, listbox behavior), it almost never belongs in `crates/fret-ui`.

If the mismatch is “every Material component needs this to feel coherent” (ripple, state layers, floating labels, motion scheme, elevation mapping), it usually belongs in `ecosystem/fret-ui-material3` foundation rather than in one component file.

### 2.5) Translate platform assumptions into GPU-first outcomes

Material upstreams often assume DOM/CSS or toolkit-managed rendering. In Fret:

- Treat upstream APIs as **spec**, not an implementation to port 1:1.
- Make hit-testing semantics explicit when motion or transforms are involved.
- Keep motion tunable via theme tokens (durations/easing/spring params), not hard-coded numbers.
- Prefer shared Material foundation helpers over per-component math for:
  - ripple/state-layer ink,
  - floating label transitions,
  - overlay open/close motion,
  - elevation/shadow/tonal overlay mapping,
  - touch-target expansion vs visual bounds.
- Use layout probes/derived state deliberately when a component needs measured widths/heights instead of depending on fragile frame timing.

Only add a new renderer/mechanism primitive if the outcome cannot be expressed cleanly at the Material foundation layer without correctness/perf drift.

### 2.75) Accessibility parity means semantics outcomes, not DOM attributes

Prefer semantics-snapshot outcomes and focused invariants:

- roles (`ComboBox`, `ListBox`, `ListBoxOption`, button-like roles, tabs/navigation roles),
- relations (`labelled_by`, `described_by`, `controls`),
- state flags (`expanded`, `selected`, `disabled`, `checked`),
- composite widget behavior (`active_descendant`, roving focus, focus order),
- collection metadata when applicable.

For field-family overlays, always verify the trigger/input ↔ popup relationship and the focused/active option semantics.

### 3) Lock behavior with regression gates (goldens are not enough)

Pick the smallest gate that locks the invariant:

- Deterministic logic/invariants → unit tests near the owning component/foundation module.
- Scene/chrome outcomes (indicator geometry, state-layer fill, centered chrome, shape/elevation invariants) → use focused scene/geometry assertions and reuse the Material interaction harness patterns under `ecosystem/fret-ui-material3/tests/`.
- Interaction state machines (dismiss/focus restore/keyboard nav/motion interruptions/filtering/typeahead) → `tools/diag-scripts/*.json` + `fretboard diag run` with stable `test_id`.
- Motion-sensitive work → run diag scripts with a fixed timestep (`--fixed-frame-delta-ms 16`) so results are deterministic.
- Accessibility-sensitive work → keep or add an a11y bundle/assertion gate when possible.

Always leave the 3-pack:

- Repro (smallest surface),
- Gate (test/script),
- Evidence (anchors + exact commands).

### 3.5) Prefer existing script/test patterns over inventing new ones

Material 3 already has useful gate shapes. Reuse them before creating a new style:

- overlay/menu/select bounds and collision scripts,
- item chrome fill / centered chrome scripts,
- focus-visible and icon-motion timeline scripts,
- a11y bundle scripts for select/dialog surfaces,
- targeted test binaries under `ecosystem/fret-ui-material3/tests/`.

Good starting places:

- `tools/diag-scripts/ui-gallery/material3/`
- `ecosystem/fret-ui-material3/tests/text_field_hover.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `ecosystem/fret-ui-material3/tests/environment_query_adoption_smoke.rs`

### 4) High-value regression targets (start here)

- Field family: `Select`, `ExposedDropdown`, `Autocomplete`, `TextField`, `DatePicker`, `TimePicker`.
- Choice controls: `Switch`, `Checkbox`, `Radio`, `Slider`, chips.
- Navigation: `Tabs`, `NavigationBar`, `NavigationRail`, `NavigationDrawer`, `ModalNavigationDrawer`, `TopAppBar`.
- Overlays/feedback: `Menu`, `Dialog`, `BottomSheet`, `Snackbar`, `Tooltip`.
- Foundations: state layer, ripple, motion scheme, floating label, active indicator, elevation.

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro, Gate, Evidence.
- A clear layer mapping in the change (no Material policy pushed into `crates/*` unless it is truly a mechanism).
- At least one regression artifact:
  - **state machine / overlay / motion** mismatch ⇒ diag script with stable `test_id`,
  - **layout / token / scene** mismatch ⇒ deterministic invariant test or scene/geometry assertion,
  - **a11y** mismatch ⇒ semantics/bundle assertion when feasible.
- Stable automation surfaces for field/overlay components: trigger/input, popup/listbox/sheet, and at least one option/item anchor.
- If the change adds or refactors Material foundation code, leave at least one consumer-level usage anchor proving the shared primitive is adopted.

## Practical gates (what to actually run)

Prefer bounded, fast gates that catch regressions without compiling the entire world:

- Targeted component tests:
  - `cargo nextest run -p fret-ui-material3 --lib <filter>`
  - `cargo nextest run -p fret-ui-material3 --test <target>`
- Scripted repros:
  - `fretboard diag run` against the relevant script under `tools/diag-scripts/ui-gallery/material3/`
  - add `--fixed-frame-delta-ms 16` for motion-sensitive checks
- When updating diag coverage, keep script names and stable `test_id` surfaces aligned with existing Material 3 gallery conventions.

## Evidence anchors

- Layering and contracts: `docs/architecture.md`, `docs/runtime-contract-matrix.md`
- Material workstream context: `docs/workstreams/material3-refactor-plan.md`, `docs/workstreams/material3-todo.md`
- Material mechanism/foundation ADR: `docs/adr/0226-material3-state-layer-and-ripple-primitives.md`
- Reference notes:
  - `.agents/skills/fret-material-source-alignment/references/material-reference-stack.md`
  - `.agents/skills/fret-material-source-alignment/references/material-field-family-checklist.md`
- Material foundation code:
  - `ecosystem/fret-ui-material3/src/foundation/`
  - `ecosystem/fret-ui-material3/src/interaction/`
  - `ecosystem/fret-ui-material3/src/tokens/`
- Material components:
  - `ecosystem/fret-ui-material3/src/select.rs`
  - `ecosystem/fret-ui-material3/src/exposed_dropdown.rs`
  - `ecosystem/fret-ui-material3/src/autocomplete.rs`
  - `ecosystem/fret-ui-material3/src/text_field.rs`
  - `ecosystem/fret-ui-material3/src/switch.rs`
  - `ecosystem/fret-ui-material3/src/icon_button.rs`
  - `ecosystem/fret-ui-material3/src/tabs.rs`
  - `ecosystem/fret-ui-material3/src/navigation_bar.rs`
  - `ecosystem/fret-ui-material3/src/navigation_rail.rs`
  - `ecosystem/fret-ui-material3/src/navigation_drawer.rs`
- Mature Fret-side parity exemplar:
  - `docs/audits/shadcn-select.md`
  - `ecosystem/fret-ui-shadcn/src/select.rs`
  - `ecosystem/fret-ui-kit/src/primitives/select.rs`
- Diagnostics + scripts: `tools/diag-scripts/ui-gallery/material3/`, `docs/ui-diagnostics-and-scripted-tests.md`
- Gallery harness/pages:
  - `apps/fret-ui-gallery/src/ui/pages/material3/`
  - `apps/fret-ui-gallery/src/ui/snippets/material3/`
- Optional upstream snapshots: `repo-ref/material-ui`, `repo-ref/base-ui`, `repo-ref/ui`, `repo-ref/primitives`

## Examples

- Example: align a Material 3 field-family overlay
  - User says: "Make this Material exposed dropdown feel like Material Web / Compose Material3."
  - Actions:
    - inspect the reference stack note plus the field-family checklist,
    - inspect one Fret-side exemplar before changing code,
    - choose a source-of-truth ordering (spec + Compose + MUI),
    - keep overlay/listbox policy in the right ecosystem layer,
    - lock width floor, filtering/typeahead, a11y, and bounds with targeted tests/scripts.
  - Result: parity improvement with regression protection and Fret-consistent layering.

## Common pitfalls

- Porting Material policy into `crates/*` (wrong layer, hard-to-change).
- Re-implementing shared Material foundations inside one component instead of using `foundation/*` / `interaction/*`.
- Hard-coding numbers (durations/radius/elevation/opacity) instead of tokenizing them.
- Treating screenshots/goldens as sufficient for state machines, ripple, or motion.
- Forgetting stable `test_id` surfaces for trigger/input, popup/listbox, and option/item nodes.
- Ignoring field-family specifics such as menu width floor, query sync-on-blur, or typeahead delay.
- Mixing Material and shadcn taxonomies in one recipe layer (keep design systems separate).
- Copying shadcn visuals into Material work just because the mechanism happens to be similar.

## Troubleshooting

- Symptom: spec and implementation references disagree.
  - Fix: document the chosen source-of-truth ordering and add evidence anchors for the decision.
- Symptom: the component is visually close but interaction still feels wrong.
  - Fix: compare against a mature in-tree exemplar first, then gate semantics/focus/dismiss/typeahead before chasing tokens.
- Symptom: a Material field overlay regresses repeatedly.
  - Fix: add stable trigger/listbox/item `test_id`s and a diag script for bounds + interaction flow.
- Symptom: ripple/state layer parity keeps drifting across components.
  - Fix: move the behavior into shared Material foundation/interaction modules and prove it with at least one consumer anchor.

## Related skills

- `fret-skills-playbook`
- `fret-diag-workflow`
- `fret-ui-review`
- `fret-app-ui-builder`
- `fret-shadcn-source-alignment`
