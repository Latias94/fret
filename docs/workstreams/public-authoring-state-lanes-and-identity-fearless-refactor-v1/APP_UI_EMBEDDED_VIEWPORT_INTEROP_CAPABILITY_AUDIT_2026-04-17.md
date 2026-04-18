# AppUi Embedded Viewport Interop Capability Audit — 2026-04-17

Status: Landed slice + targeted evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_ROOT_ACCESSOR_AUDIT_2026-04-15.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEFAULT_TEXT_BUILDER_SURFACE_AUDIT_2026-04-17.md`
- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Scope

Decide how the remaining `embedded_viewport_demo` no-`Deref` failures should be classified:

- should this root collapse onto a broad raw `ElementContext` lane,
- should `AppUi` widen again,
- or should the app-facing render root keep ordinary authoring on explicit capability-first
  late-landing and reserve raw `ElementContext` only for the true interop seam?

This note is intentionally about a mixed app-facing/interop root, not a new framework surface.

## Method

1. Inspect the post-advanced-helper no-`Deref` failure map and confirm that
   `apps/fret-examples/src/embedded_viewport_demo.rs` still fails on:
   - ordinary `.into_element(cx)` late-landing,
   - `cx.text(...)` inside `ToggleGroupItem`,
   - and the direct `EmbeddedViewportSurface::panel(...)` call.
2. Keep tracked reads and ordinary root authoring on `AppUi`:
   - `cx.theme_snapshot()`
   - `LocalState::layout_value(...)`
   - model paint reads
3. Migrate ordinary late-landing to the explicit capability lane:
   - import `fret_ui_kit::IntoUiElementInExt as _`
   - use `.into_element_in(cx)` for header/card/container landing
   - replace `cx.text(...)` with `ui::text(...).into_element_in(cx)` in `ToggleGroupItem`
4. Keep the true interop seam explicit instead of widening `AppUi`:
   - `self.embedded.panel(cx.elements(), ...)`
5. Re-run:
   - `cargo test -p fret-examples --lib embedded_viewport_demo_prefers_capability_first_landing_with_explicit_panel_owner`
   - `cargo test -p fret-examples --lib selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`
   - `cargo check -p fret-examples --all-targets --message-format short`
6. Re-run the temporary local no-`Deref` compile audit:
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`

The temporary `AppUi` no-`Deref` patch was reverted immediately after collecting the new failure
map.

## Findings

### 1) This is a mixed app-facing root with one real interop escape hatch

The remaining failures were not evidence that the whole root should move to raw
`ElementContext`.

Most of the failing callsites were ordinary authoring:

- header/card late-landing,
- toggle-label text children,
- card/container landing.

Only the actual embedded viewport panel construction still needed raw `ElementContext`.

### 2) The correct fix is capability-first late-landing plus one explicit interop owner

The owner-correct implementation keeps the root readable and honest:

- tracked reads stay on `AppUi`,
- ordinary builders land through `IntoUiElementInExt::into_element_in(cx)`,
- ordinary labels use `ui::text(...).into_element_in(cx)`,
- and only `EmbeddedViewportSurface::panel(...)` uses `cx.elements()`.

That preserves the app-facing authoring story while keeping the genuine interop seam explicit.

### 3) The follow-on no-`Deref` audit gets materially smaller again

With the paired `date_picker_demo` follow-on landed and the temporary local patch that removes
both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` drops from `67` to `49` previous errors,
- `embedded_viewport_demo` no longer appears in the `fret-examples` output,
- `date_picker_demo` also no longer appears,
- and the current top `fret-examples` clusters are now led by:
  - `todo_demo.rs` (`8`)
  - `async_playground_demo.rs` (`6`)
  - `markdown_demo.rs` (`5`)

That is strong evidence that `embedded_viewport_demo` needed a narrower capability/owner split,
not a wider `AppUi` façade.

## Evidence

- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `cargo test -p fret-examples --lib embedded_viewport_demo_prefers_capability_first_landing_with_explicit_panel_owner`
- `cargo test -p fret-examples --lib selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`
- `cargo check -p fret-examples --all-targets --message-format short`
- temporary local no-`Deref` audit:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `cargo check -p fret-cookbook --all-targets --message-format short`
  - `/tmp/fret_examples_noderef_date_embedded_v2.txt`
  - `/tmp/fret_cookbook_noderef_date_embedded_v2.txt`

## Outcome

The repo now has an explicit owner answer for the embedded viewport app root:

1. ordinary app-facing authoring stays on `AppUi`
2. ordinary late-landing/text children use the existing capability-first lane
3. the raw interop seam stays explicit at `EmbeddedViewportSurface::panel(cx.elements(), ...)`
