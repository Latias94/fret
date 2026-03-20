# Selector Borrowed-Input Audit — 2026-03-20

This audit records the first selector-side follow-up pass for the selector/query density lane.

Goal:

- determine whether the current LocalState-first selector surface now needs a borrowed-input
  follow-on,
- or whether the remaining pressure is still too narrow to justify a new public API.

## Evidence surfaces used in this pass

Higher-pressure app-facing surfaces:

- `apps/fretboard/src/scaffold/templates.rs`
- `apps/fret-examples/src/async_playground_demo.rs`

Cross-check surfaces using the same selector lane without obvious pressure:

- `apps/fret-examples/src/hello_counter_demo.rs`
- `apps/fret-cookbook/examples/text_input_basics.rs`
- `apps/fret-cookbook/examples/markdown_and_code_basics.rs`
- `apps/fret-cookbook/examples/virtual_list_basics.rs`
- `apps/fret-examples/src/postprocess_theme_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_demo.rs`
- `apps/fret-examples/src/custom_effect_v3_demo.rs`
- `apps/fret-examples/src/liquid_glass_demo.rs`
- `apps/fret-examples/src/launcher_utility_window_demo.rs`

## Findings

### 1. Most current `selector_layout(...)` sites are small settings projections, not evidence for another API

The majority of current selector call sites follow this shape:

- read a few scalar or small option-like locals,
- fold them into a `*Settings` / `*Stats` struct,
- then use that struct to drive layout or rendering.

Representative examples:

- `TextInputStats`
- `PreviewSettings`
- `VirtualListViewSettings`
- `ThemePostprocessViewSettings`
- `CustomEffectV2ViewSettings`
- `CustomEffectV3ViewSettings`
- `LiquidGlassVisibilitySettings`
- `LiquidGlassModeSettings`
- `LauncherUtilityWindowViewSettings`

These are already:

- explicit,
- easy to read,
- and not obviously noisy because of missing borrowed selector inputs.

Conclusion:

- these surfaces do not justify a new borrowed-compute selector API.

### 2. The current real pressure is narrow and concentrated in only two app-facing patterns

#### A. Third-rung Todo scaffold

`apps/fretboard/src/scaffold/templates.rs` still builds:

- `TodoDerived`
- `TodoRowSnapshot`

inside `selector_layout((&todos_state, &filter_state), ...)`.

This is the strongest current example of a LocalState-first selector paying an "owned values"
cost, because the input `todos` collection is cloned into the selector compute and then partially
projected again into row snapshots.

#### B. Async playground query helpers

`apps/fret-examples/src/async_playground_demo.rs` uses selector structs such as:

- `QueryPolicySettings`
- `QueryKeyInputs`

Those shapes do create some owned intermediate state, but they are also quite intentional:

- the query key needs owned strings,
- the policy builder needs a compact typed projection anyway,
- and the code remains readable.

Conclusion:

- only the Todo scaffold currently reads like a strong ergonomics pressure point,
- while the async playground looks more like an intentional typed projection surface.

### 3. There is not yet enough cross-surface evidence for a new selector borrowed-input surface

Current evidence does **not** show the same pressure repeating across:

- the first-contact default path,
- generic app surfaces,
- and additional non-Todo app-facing surfaces.

What it shows instead:

- one heavier third-rung scaffold (`todo`),
- one medium app-facing helper surface (`async_playground`) that is still arguable as intentional,
- and many selector sites that are already fine on the current owned-input surface.

Conclusion:

- the current repo state does not yet justify a public borrowed-input selector follow-on.

## Ownership reminder

If this question is ever reopened later:

- the follow-on must stay on the app-facing layer,
- it must not teach `fret-selector` about `LocalState<T>`,
- and it must not widen `fret::app::prelude::*`.

## Decision from this audit

Current verdict:

- **no new selector API for now**

Read the selector side this way:

- the current LocalState-first `selector_layout(...)` surface remains the shipped default,
- the remaining pressure is real but still too narrow for another public helper family,
- and future reopening requires fresh proof beyond the Todo scaffold.

## Immediate execution consequence

For this lane:

1. keep the query semantic projection batch as the shipped code change,
2. treat selector borrowed-input work as deferred pending fresh cross-surface evidence,
3. and do not mint a borrowed selector surface from Todo-only pressure.
