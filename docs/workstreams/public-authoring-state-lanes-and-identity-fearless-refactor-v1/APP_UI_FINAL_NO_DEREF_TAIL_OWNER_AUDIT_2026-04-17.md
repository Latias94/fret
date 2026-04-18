# AppUi Final No-Deref Tail Owner Audit — 2026-04-17

Status: Landed slice + tail-closed evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_ADVANCED_RAW_BUILDER_OWNER_AUDIT_2026-04-17.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/COMPONENTS_GALLERY_OWNER_SPLIT_AUDIT_2026-04-16.md`
- `apps/fret-examples/src/components_gallery.rs`
- `apps/fret-examples/src/custom_effect_v1_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_demo.rs`
- `apps/fret-examples/src/custom_effect_v3_demo.rs`
- `apps/fret-examples/src/genui_demo.rs`
- `apps/fret-examples/src/liquid_glass_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Classify and land the final `fret-examples` no-`Deref` tail that remained after the advanced raw
builder triplet closed:

- `apps/fret-examples/src/components_gallery.rs` (`2`)
- `apps/fret-examples/src/custom_effect_v1_demo.rs` (`1`)
- `apps/fret-examples/src/custom_effect_v2_demo.rs` (`1`)
- `apps/fret-examples/src/custom_effect_v3_demo.rs` (`1`)
- `apps/fret-examples/src/genui_demo.rs` (`1`)
- `apps/fret-examples/src/liquid_glass_demo.rs` (`1`)

The question is whether this remaining tail proves more `AppUi` forwarding should exist, or
whether the correct fix is still explicit owner landing at the callsite.

## Method

1. Re-read the remaining no-`Deref` failure map from the previous spot-check.
2. Separate the tail into two owner classes:
   - advanced/reference view-entry helpers already typed as `ElementContext`
   - `components_gallery` normal-branch raw builder/theme-name late landing
3. Land the narrow fixes:
   - change `view(cx, ...)` to `view(cx.elements(), ...)` in
     `custom_effect_v1_demo`, `custom_effect_v2_demo`, `custom_effect_v3_demo`,
     `genui_demo`, and `liquid_glass_demo`
   - keep `components_gallery` tracked reads on `AppUi`, then enter
     `let cx = cx.elements();` before `theme_name` and the surrounding raw builder phase
4. Lock the owner split with source-policy gates in `apps/fret-examples/src/lib.rs`.
5. Re-run targeted proof commands:
   - `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
   - `cargo test -p fret-examples --lib manual_components_gallery_uses_app_ui_render_root_bridge`
   - `cargo check -p fret-examples --all-targets --message-format short`
6. Re-run a temporary no-`Deref` spot-check:
   - locally disable `impl Deref` and `impl DerefMut` for `AppUi` in `ecosystem/fret/src/view.rs`
   - run `cargo check -p fret-examples --all-targets --message-format short`
   - restore `ecosystem/fret/src/view.rs` immediately after collecting evidence

The temporary `AppUi` no-`Deref` patch was reverted immediately after the audit.

## Findings

### 1) The remaining advanced/reference errors were stale callsites, not missing `AppUi` surface

`custom_effect_v1_demo`, `custom_effect_v2_demo`, `custom_effect_v3_demo`, `genui_demo`, and
`liquid_glass_demo` already had the correct helper ownership:

- their `view(...)` functions intentionally accept `&mut ElementContext<'_, KernelApp>`
- the `render(...)` entrypoints still passed `&mut AppUi` only because the temporary compatibility
  bridge existed

The correct fix is therefore explicit owner landing at the entrypoint:

- keep `AppUi` for state/actions/effects orchestration
- call `view(cx.elements(), ...)` once the render entry crosses into the advanced raw helper lane

### 2) `components_gallery` normal-branch ownership matches the earlier manual/raw-builder slices

The table-torture retained branch was already explicit, but the normal branch still mixed:

- tracked `layout(...)` reads on `AppUi`
- a theme-name read that needs raw theme ownership
- outer raw late landing through `.into_element(cx)`

The correct fix is the same split used in `form_demo`, `date_picker_demo`, `postprocess_theme_demo`,
and the advanced raw-builder triplet:

- keep theme snapshot plus tracked model reads on `AppUi`
- then enter `let cx = cx.elements();`
- then perform the theme-name read and the surrounding raw builder phase

That keeps the owner boundary explicit without pretending `theme().name` belongs on the ordinary
app-facing façade.

### 3) The `fret-examples` no-`Deref` tail closes completely

With the temporary local patch that disables both `impl Deref` and `impl DerefMut` on `AppUi`,
the follow-on spot-check now finishes cleanly:

- previous `fret-examples` tail: `7` errors
- current `fret-examples` tail: `0` errors

That is the key evidence that this slice was entirely owner-callsite cleanup rather than fresh
framework API pressure.

## Evidence

- `apps/fret-examples/src/components_gallery.rs`
- `apps/fret-examples/src/custom_effect_v1_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_demo.rs`
- `apps/fret-examples/src/custom_effect_v3_demo.rs`
- `apps/fret-examples/src/genui_demo.rs`
- `apps/fret-examples/src/liquid_glass_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib selected_raw_owner_examples_keep_escape_hatches_explicit`
- `cargo test -p fret-examples --lib manual_components_gallery_uses_app_ui_render_root_bridge`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary no-`Deref` spot-check:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_tail_closed_v10.txt`

## Outcome

The repo now has an explicit owner answer for the final `fret-examples` no-`Deref` tail:

1. Advanced/reference render entrypoints keep `AppUi` for orchestration and enter raw helper
   ownership through `view(cx.elements(), ...)`.
2. `components_gallery` keeps tracked reads on `AppUi`, then explicitly enters `cx.elements()`
   for the raw theme-name + builder phase.
3. A temporary local no-`Deref` spot-check now leaves `fret-examples` clean, so the remaining
   cleanup can focus on actually deleting the compatibility bridge and shrinking the residual
   taught surface rather than hunting more example-level tail errors.
