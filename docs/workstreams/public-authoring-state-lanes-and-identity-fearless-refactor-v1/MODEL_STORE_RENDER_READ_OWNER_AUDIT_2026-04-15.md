# Model Store Render Read Owner Audit 2026-04-15

Scope:

- `apps/fret-cookbook/examples/virtual_list_basics.rs`
- `apps/fret-examples/src/custom_effect_v2_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs`
- `apps/fret-examples/src/external_texture_imports_demo.rs`
- `apps/fret-examples/src/external_texture_imports_web_demo.rs`
- `apps/fret-examples/src/external_video_imports_avf_demo.rs`
- `apps/fret-examples/src/external_video_imports_mf_demo.rs`

## Question

After the grouped query-maintenance and cookbook theme-owner slices landed, the next remaining
`models().read/get_* / revision(...)` grep surface mixed three different owners:

1. default app-facing render code that already had tracked-read helpers,
2. direct-leaf/manual `UiTree` render roots that already had grouped `cx.data()` selector helpers,
3. pure app/driver loops that intentionally own raw `ModelStore`.

This note records which surfaces should move and which should stay raw.

## Assumptions

1. `virtual_list_basics` is still teaching ordinary `AppUi` authoring, not a driver/runtime
   escape hatch.
   Evidence: [`apps/fret-cookbook/examples/virtual_list_basics.rs`](/Users/frankorz/Documents/projects/rust/fret/apps/fret-cookbook/examples/virtual_list_basics.rs)
   uses `impl View for VirtualListBasicsView`, `cx.state().local_*`, and existing
   `self.items.layout(cx).value_or_default()`.
   Confidence: Confident.
   Consequence if wrong: migrating revisions away from `cx.app.models()` would hide an intentional
   raw-store teaching surface.

2. The Custom Effect V2 web family is a direct-leaf/manual render lane, but render-time grouped
   model reads should still stay on `cx.data()` instead of spelling raw `observe_model +
   app.models().read(...)`.
   Evidence: the files already import `fret::advanced::view::UiCxDataExt as _` and already use
   `cx.data().selector_model_paint(...)` for the control bag.
   Confidence: Confident.
   Consequence if wrong: these files would drift toward two inconsistent render-time model stories
   inside the same root.

3. The external texture/video import demos are also direct-leaf/manual surfaces, but their root
   `show` visibility read is still render-time state and therefore belongs on the grouped selector
   lane, not on raw `ModelStore`.
   Evidence: the render roots are `fn render_view(cx: &mut ElementContext<'_, App>, ...)`,
   while the driver-side `record_engine_frame(...)` functions still need raw `app.models()` for
   frame orchestration outside the render tree.
   Confidence: Confident.
   Consequence if wrong: we would either keep teaching raw store reads in render code, or wrongly
   flatten driver/app loops onto render-only helpers.

4. No new framework API is required for this slice.
   Evidence: `fret::view::TrackedStateExt::{layout, paint}` already exposes `.revision()`, and
   `fret::view::{AppUiData, UiCxData}` already expose `selector_model_layout(...)`.
   Confidence: Confident.
   Consequence if wrong: we would spend a slice adding redundant surface instead of closing
   existing example drift.

## Owner resolution

### 1. Default `AppUi` render surfaces

Owner: tracked builder on the app-facing lane.

Decision:

- `virtual_list_basics` should use `self.items.layout(cx).revision()` and
  `local_state.layout(cx).revision()` instead of reopening `cx.app.models()`.

Why:

- The example already uses `TrackedStateExt` for value reads.
- The remaining raw store access was only revision aggregation, not a real driver/store boundary.

### 2. Direct-leaf/manual `ElementContext` render roots

Owner: grouped selector on `cx.data()`.

Decision:

- The Custom Effect V2 web family should read root visibility with
  `cx.data().selector_model_layout(&show, |show| show)`.
- The external texture/video import demos should read root visibility with
  `cx.data().selector_model_layout(&st.show, |show| show)` or
  `cx.data().selector_model_layout(&show_model, |show| show)`.

Why:

- These surfaces already own explicit `Model<T>` bags.
- `selector_model_layout(...)` is the existing grouped helper for explicit shared models on the
  render lane.
- The render roots stay direct-leaf/manual; only the read path stops teaching raw store access.

### 3. Pure app/driver loops

Owner: raw `app.models()` / `app.models_mut()`.

Decision:

- Keep raw `app.models().read(...)` in driver-side paths such as `record_engine_frame(...)`.

Why:

- Those functions run outside the render tree and do not own `cx.data()`.
- Replacing them would blur the boundary between render helpers and driver/runtime orchestration.

## Landed slice

- `virtual_list_basics` now computes `items_revision` from tracked builders instead of
  `let store = cx.app.models();`.
- `custom_effect_v2_web_demo`, `custom_effect_v2_lut_web_demo`,
  `custom_effect_v2_identity_web_demo`, and `custom_effect_v2_glass_chrome_web_demo` now read
  render-time `show` via `cx.data().selector_model_layout(...)`.
- `external_texture_imports_demo`, `external_texture_imports_web_demo`,
  `external_video_imports_avf_demo`, and `external_video_imports_mf_demo` now read render-time
  `show` via `cx.data().selector_model_layout(...)`.
- Driver-side frame orchestration in those import demos stays on raw `app.models().read(...)`.

## Follow-on implication

The remaining `models().read/get_* / revision(...)` grep surface is no longer one unresolved
render-time authoring seam. It is now more cleanly split across:

- intentional driver/app loops,
- retained/component-layer internal owners,
- and a smaller remaining set of advanced/reference examples that need their own owner audit.

That makes future `AppUi` `Deref` discussion narrower and evidence-based instead of grep-based.
