# Material reference stack and Fret-side exemplars

Use this note when you need to choose **which upstream truth to trust first** and **which in-tree Fret exemplar to inspect before coding**.

Goal: keep Material parity work consistent, reviewable, and architecture-aligned.

## 1) Default source-of-truth ordering

Use this ordering unless the component clearly demands a different one:

1. **Material Design 3 spec/guidelines**
   - UX intent, tokens, density, motion intent, and naming.
2. **Compose Material3**
   - toolkit-style state machines, semantics, motion/touch behavior, and non-DOM interaction patterns.
3. **MUI Material UI**
   - web-specific composition, default props, portal/focus edge cases, and browser-facing outcomes.
4. **Base UI**
   - accessibility-first headless part composition and fallback a11y patterns.

If sources disagree, document the chosen ordering in the change and keep the disagreement local to the component.

Treat this as an axis-based ordering, not a blanket “one source wins everything” rule:

- visual chrome / defaults / web composition → MUI tends to be the best first implementation reference,
- semantics / state machine / touch / motion foundations → Compose Material3 tends to be the best first implementation reference,
- taxonomy / UX intent / token direction → Material spec stays above both,
- headless parts / a11y fallback patterns → Base UI stays supporting, not taxonomy-defining.


## 1.5) Default-style ownership (recipe vs call site)

Before turning a visual/style mismatch into a new default, inspect *where upstream applies it*: 

- If the behavior or styling is encoded in the upstream component implementation or default API, it is a candidate recipe default in Fret.
  - Typical examples: intrinsic container height, shape, elevation, outline/filled chrome, active indicator, default content padding, state layer / ripple behavior.
- If the behavior or styling is encoded in the upstream sample/container code, keep it caller-owned in Fret.
  - Typical examples: `fillMaxWidth`, `widthIn`, `maxWidth`, page centering, surrounding `Box` / `Row` / `Column` constraints, grid placement, doc/demo wrappers.

Source-specific hints:

- **Compose Material3**: `Modifier.fillMaxWidth()` or surrounding `Box/Row/Column` layout code is usually caller-owned unless the composable itself applies it internally.
- **MUI Material UI**: `sx`, wrapper `Box`, layout grid props, and demo container classes are usually caller-owned unless the component default props/style overrides own them.
- **Base UI**: layout wrappers in examples are caller-owned; headless parts/semantics belong closer to the recipe/policy layer.

Heuristic:

- recipe default = intrinsic to the component across most placements
- caller-owned = negotiated by the page/container composition

When in doubt, prefer caller-owned for width/flex/grid negotiation. It is easier to opt in locally than to unwind an over-opinionated default later.

## 2) What each source is best at

### Material spec

Best for:

- visual and interaction intent,
- token naming and fallback direction,
- component taxonomy,
- state definitions,
- density and touch-target expectations.

Do **not** treat the spec as a sufficient implementation guide for:

- overlay dismissal/focus restore edge cases,
- field-family state ownership,
- web-specific portal behavior,
- exact motion/state machine wiring.

### Compose Material3

Best for:

- non-DOM state machines,
- semantics and focus outcomes,
- motion/timing expectations in a toolkit renderer,
- touch-centric behavior and minimum target sizing,
- shared foundation patterns (`MotionScheme`, indication, tokens-as-typed-access).

Use Compose when the question is “How should this feel in a retained/custom renderer?”

### MUI Material UI

Best for:

- web-facing composition and defaults,
- menu/select/dialog interplay,
- anchored overlay behavior in browser-like environments,
- common field-family ergonomics.

Use MUI when the question is “How should this Material component compose on the web?”

### Base UI

Best for:

- headless part composition,
- accessibility-first structure,
- fallback patterns when Material/MUI sources are visually noisy but the a11y contract is still clear.

Use Base UI to clarify parts and semantics, not to override Material taxonomy.

## 3) Fret-side exemplars to inspect first

Before implementing a Material fix, inspect at least one mature in-tree exemplar.

### Highest-value exemplar for overlay/listbox families

- Audit note: `docs/audits/shadcn-select.md`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/select.rs`
- shared primitive/policy surface: `ecosystem/fret-ui-kit/src/primitives/select.rs`

What this exemplar teaches well:

- mechanism vs policy vs recipe boundaries,
- stable `test_id` derivation for trigger/listbox/items,
- semantics outcomes (`ComboBox`, `ListBox`, `active_descendant`, `controls`, `expanded`),
- overlay sizing and width-probe policy,
- how to leave a small repro + gate + evidence trail.

This is **not** the Material visual truth. It is a strong Fret-side exemplar for how parity work should be structured.

### Current Material field-family exemplars

- Material Select: `ecosystem/fret-ui-material3/src/select.rs`
- Material Exposed Dropdown: `ecosystem/fret-ui-material3/src/exposed_dropdown.rs`
- Material Autocomplete: `ecosystem/fret-ui-material3/src/autocomplete.rs`
- Material Text Field: `ecosystem/fret-ui-material3/src/text_field.rs`

What these exemplars teach well:

- field-family ownership boundaries,
- floating-label choreography,
- active indicator / outline / supporting text outcomes,
- query ↔ selected value synchronization,
- Material-specific `test_id` stamping and gallery coverage.

### Shared Material foundation exemplars

- tokens and resolver: `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`
- motion scheme: `ecosystem/fret-ui-material3/src/foundation/motion_scheme.rs`
- floating label: `ecosystem/fret-ui-material3/src/foundation/floating_label.rs`
- indication: `ecosystem/fret-ui-material3/src/foundation/indication.rs`
- overlay motion: `ecosystem/fret-ui-material3/src/foundation/overlay_motion.rs`
- state layer: `ecosystem/fret-ui-material3/src/interaction/state_layer.rs`
- ripple: `ecosystem/fret-ui-material3/src/interaction/ripple.rs`

Inspect these first when the same drift appears across multiple Material components.

## 4) Optional local snapshots and upstream repos

Use local snapshots only as optional convenience.

Available pinned/local mirrors in this repo may include:

- `repo-ref/material-ui`
- `repo-ref/compose-multiplatform-core` (or an equivalent local Compose Multiplatform core mirror under a different folder name)
- `repo-ref/base-ui`
- `repo-ref/ui`
- `repo-ref/primitives`

Notes:

- `repo-ref/ui` and `repo-ref/primitives` are useful when a Material component reuses the same overlay/listbox/menu semantics already proven in shadcn/Radix work.
- If the Compose mirror is present locally, prefer it over remote browsing for exact Compose behavior.
- If the Compose mirror is absent in the current checkout, fall back to upstream source/docs and say so explicitly in the change notes.

## 5) Quick decision matrix

### If the mismatch is mostly about...

- **tokens / density / visual intent** → start with Material spec.
- **motion / touch / toolkit semantics / non-DOM behavior** → start with Compose Material3.
- **web composition / portal / popup behavior** → start with MUI Material UI.
- **parts / headless accessibility patterns** → start with Base UI.
- **Fret-side layering / gating / stable automation surfaces** → inspect the shadcn Select audit plus current Material recipes.

When the change touches shared Material foundation code, inspect both the Compose-side and MUI-side references before choosing the abstraction boundary.

## 6) Questions to answer before coding

- Which source defines the UX truth for this specific mismatch?
- Which source explains the state machine most clearly?
- Is there already a Fret-side exemplar that solved the same mechanism split?
- Does the problem belong in shared Material foundation, shared kit policy, or one component recipe?
- What is the smallest gate that would fail before the fix and stay stable after refactors?
