# shadcn/ui v4 Audit - Alert Dialog (new-york)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `AlertDialog` surface against the upstream shadcn/ui v4
docs/examples surfaces and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs pages:
  - `repo-ref/ui/apps/v4/content/docs/components/base/alert-dialog.mdx`
  - `repo-ref/ui/apps/v4/content/docs/components/radix/alert-dialog.mdx`
- Docs examples:
  - `repo-ref/ui/apps/v4/examples/base/alert-dialog-*.tsx`
  - `repo-ref/ui/apps/v4/examples/radix/alert-dialog-*.tsx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert-dialog.tsx`
- Underlying primitive concept: Radix `@radix-ui/react-alert-dialog` (dialog + safety defaults)

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`
- Depends on overlay policy/infra:
  - `ecosystem/fret-ui-kit/src/window_overlays/*` (modal overlays, focus restore/initial focus)
  - `ecosystem/fret-ui-kit/src/overlay_controller.rs` (overlay requests)
  - `ecosystem/fret-ui-kit/src/primitives/alert_dialog.rs` (Radix-aligned cancel focus policy)
  - `ecosystem/fret-ui-kit/src/primitives/dialog.rs` (Radix-aligned modal request facade)
  - `crates/fret-ui/src/tree/*` (modal barrier scoping + focus traversal contract, ADR 0068)

## What upstream exports (new-york)

Upstream shadcn/ui exports a thin wrapper around Radix:

- `AlertDialog`
- `AlertDialogTrigger`
- `AlertDialogContent`
- `AlertDialogHeader`
- `AlertDialogFooter`
- `AlertDialogTitle`
- `AlertDialogDescription`
- `AlertDialogAction`
- `AlertDialogCancel`

## Audit checklist

### Composition surface

- Pass: Fret exposes `AlertDialog` as a recipe driven by a `Model<bool>` open state.
- Pass: Trigger/content composition matches the shadcn mental model.
- Pass: `AlertDialogAction` / `AlertDialogCancel` now provide `from_scope(...)` authoring helpers for
  content-local composition while preserving the explicit `new(label, open)` constructors.
- Pass: `AlertDialog::children([...])` now provides a root-level part-children surface that is
  closer to upstream nested children authoring while still lowering into the existing slot-based
  recipe.
- Pass: `AlertDialogPart::content_with(...)` together with `AlertDialogContent::with_children(...)`,
  `AlertDialogHeader::with_children(...)`, and `AlertDialogFooter::with_children(...)` now forms
  the default copyable content-side composition lane for first-party examples, so
  `AlertDialogAction::from_scope(...)` / `AlertDialogCancel::from_scope(...)` are rendered inside
  the alert-dialog content scope instead of being taught through an eager array path.
- Pass: `AlertDialogContent::new([...])`, `AlertDialogHeader::new([...])`, and
  `AlertDialogFooter::new([...])` remain available for already-materialized children and composed
  sections, but they are no longer treated as the default teaching lane for scope-sensitive parts.
- Pass: `AlertDialogPart` is now reachable from the curated `facade` import lane, so the default
  `use fret_ui_shadcn::{facade as shadcn, prelude::*};` teaching path can author
  `AlertDialog::children([...])` without dropping to `raw::*`.
- Pass: `AlertDialog::compose()` provides a recipe-level builder for part assembly without pushing
  shadcn-specific composition concerns into the lower-level mechanism contract.
- Pass: `AlertDialogContent::build(...)` remains the typed content-side companion when staged or
  conditional assembly is clearer than deferred `with_children(...)`.
- Pass: `AlertDialogDescription::new_selectable(...)` /
  `AlertDialogDescription::new_selectable_with(...)` now provide a first-class recipe seam for
  inline interactive spans (for example the upstream destructive `Settings` link) without forcing
  callers back through the generic `new_children([...])` escape hatch.
- Note: Public-surface drift still exists compared with full React/JSX nesting. Fret now supports
  root-level part children, but it still collects recipe parts explicitly instead of accepting
  arbitrary nested JSX-like children.

### Dismissal behavior (safety defaults)

- Pass: Escape dismiss is supported (shared dismissible root).
- Pass: Overlay click-to-dismiss is **disabled by default** (Radix/shadcn safety behavior).
- Pass: Open lifecycle callbacks are available via `AlertDialog::on_open_change` and
  `AlertDialog::on_open_change_complete` (Base UI `onOpenChange` + `onOpenChangeComplete`).

### Focus behavior (safety defaults)

- Pass: Modal barrier scoping prevents underlay focus traversal (ADR 0068).
- Pass: On open, default initial focus prefers the first `AlertDialogCancel` button when present
  (Radix outcome).
- Pass: On close, focus restoration is deterministic to the trigger.

### Visual parity (new-york)

- Pass: Motion matches shadcn's `fade` + `zoom-in-95` / `zoom-out-95` outcomes (best-effort, tick
  driven).
- Pass: `AlertDialogCancel::variant(...)` now supports the non-default cancel styling used by
  upstream destructive example surfaces without requiring a lower-level escape hatch.
- Pass: The first-party UI Gallery examples now follow the docs-page teaching surface (`Show
  Dialog`, `Share Project`, dual-example RTL) instead of the older registry-base example labels.
- Note: The current `repo-ref/ui` routed web-golden surfaces only expose `alert-dialog-demo`.
  Docs-only examples such as `alert-dialog-small`, `alert-dialog-media`,
  `alert-dialog-small-media`, and `alert-dialog-destructive` are present in source/docs, but they
  are not currently emitted as routable `view`/`preview` items in the local snapshot. Visual
  evidence for those examples therefore relies on first-party UI Gallery diagnostics until upstream
  routeable goldens become available.

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo check -p fret-ui-gallery`
- `cargo test -p fret-ui-shadcn --lib alert_dialog::tests`
- `cargo test -p fret-ui-shadcn --lib alert_dialog::tests::alert_dialog_content_build_accepts_builder_first_sections -- --exact`
- `cargo test -p fret-ui-shadcn --lib alert_dialog::tests::alert_dialog_content_new_accepts_composed_sections_and_test_id -- --exact`
- Contract test: `alert_dialog_open_change_events_emit_change_and_complete_after_settle`
- Contract test: `alert_dialog_open_change_events_complete_without_animation`
- Shadcn Web chrome gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_alert_dialog_demo_panel_chrome_matches`).
- Shadcn Web placement gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_alert_dialog_demo_overlay_center_matches`).
- Radix Web overlay geometry gate: `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
  (`radix_web_alert_dialog_open_geometry_matches_fret`).
- UI Gallery docs-example screenshot gate: `tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-docs-example-open-screenshots.json`
  (covers `Small`, `Media`, `Small with Media`, and `Destructive` on the first-party docs page).

## Authoring note: `from_scope(...)`

Fret now exposes `AlertDialogAction::from_scope(...)` and `AlertDialogCancel::from_scope(...)` as
recipe-layer sugar.

- Scope: only for parts whose semantic job is “close the current alert dialog”.
- Layering: this does **not** change the underlying mechanism contract; it only reads the current
  alert dialog content scope while rendering the recipe.
- Escape hatch: `new(label, open)` remains the explicit constructor and should be preferred when
  building the part outside the alert dialog content subtree.
- Failure mode: `from_scope(...)` panics when rendered outside alert-dialog content so misuse is
  caught early during development.

## Authoring note: `children([...])`

`AlertDialog::children([...])` is now the default copyable root path for first-party examples.

- Scope: ergonomics only; it lowers into `AlertDialog::into_element_parts(...)`.
- Shape: root-level `AlertDialogPart::{trigger, portal, overlay, content}` adapters keep the part
  collection explicit while still reading closer to upstream nested children.
- Default teaching path: first-party examples now prefer
  `AlertDialog::new_controllable(cx, None, false).children([AlertDialogPart::trigger(...), AlertDialogPart::content_with(|cx| { ... })])`.
- Limitation: this is still not a full React-style nested children API; Fret stores deferred recipe
  parts and assembles them at the final call site.

## Authoring note: deferred content children

First-party examples now pair the root `children([...])` path with deferred content assembly:

- `AlertDialogPart::content_with(|cx| { ... })`
- `AlertDialogContent::new([]).with_children(cx, |cx| ...)`
- `AlertDialogHeader::new([]).with_children(cx, |cx| ...)`
- `AlertDialogFooter::new([]).with_children(cx, |cx| ...)`
- `AlertDialogDescription::new_selectable_with(props, Some(handler))`

Why this is the default teaching path:

- it keeps `AlertDialogAction::from_scope(...)` / `AlertDialogCancel::from_scope(...)` inside the
  alert-dialog content scope where they are valid,
- it gives inline links / interactive description spans an explicit recipe seam instead of teaching
  them through generic child escape hatches,
- it stays close to the upstream nested-children mental model without widening the mechanism
  contract,
- it keeps automation-friendly hooks available through `AlertDialogContent::test_id(...)`.

The eager `new([...])` constructors remain useful when children are already materialized `AnyElement`
values or when a section is being assembled outside the scope-sensitive lane. `build(...)` remains
the typed fallback when the content really is easier to assemble in stages.

## Authoring note: `compose()`

`AlertDialog::compose()` is a recipe-layer bridge for authors who want a more composable part-based
style than the raw closure root.

- Scope: ergonomics only; it lowers into `AlertDialog::into_element_parts(...)`.
- Default teaching path: `compose()` is now the explicit builder/escape hatch when callers prefer
  chained configuration over root-level part collection.
- Focused follow-up lanes: explicit root-part ownership stays documented through `Parts`, while
  `Detached Trigger` and `Rich Content` remain additional Fret-specific follow-ups instead of
  replacing the default copyable path.
- Layering: it does **not** change the underlying overlay/focus/dismiss mechanism.
- Limitation: it is still a builder/closure bridge, so `children([...])` is the closer upstream
  mental model for first-party teaching.

## Follow-ups (recommended)

- Add optional presence/motion (if desired) while preserving safety semantics.
