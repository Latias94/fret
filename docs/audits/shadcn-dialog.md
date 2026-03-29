# shadcn/ui v4 Audit - Dialog (new-york)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Dialog` surface against the upstream shadcn/ui v4 docs
and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/dialog.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx`
- Underlying primitive concept: Radix `@radix-ui/react-dialog` (portal + dismiss + focus management)

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/dialog.rs`
- Depends on overlay policy/infra:
  - `ecosystem/fret-ui-kit/src/window_overlays/*` (modal overlays, focus restore/initial focus)
  - `ecosystem/fret-ui-kit/src/overlay_controller.rs` (overlay requests + presence)
  - `ecosystem/fret-ui-kit/src/primitives/dialog.rs` (Radix-aligned trigger a11y + request facade)
  - `crates/fret-ui/src/tree/*` (modal barrier scoping + focus traversal contract, ADR 0068)

## What upstream exports (new-york)

Upstream shadcn/ui exports a thin wrapper around Radix:

- `Dialog`
- `DialogTrigger`
- `DialogContent`
- `DialogHeader`
- `DialogFooter`
- `DialogTitle`
- `DialogDescription`

## Audit checklist

### Composition surface

- Pass: Fret exposes `Dialog` as a recipe driven by a `Model<bool>` open state.
- Pass: Trigger/content composition matches the shadcn mental model.
- Pass: Content is rendered via a per-window overlay root (portal-like), so it is not clipped by
  underlay layout/overflow.
- Pass: `DialogContent` renders a default close affordance and exposes
  `show_close_button(false)`, matching upstream `showCloseButton={false}`.
- Pass: `DialogClose` is available as an explicit close affordance recipe (close button parity).
- Pass: `DialogClose::from_scope()` is available as recipe-layer sugar for content-local close
  buttons while preserving `DialogClose::new(open)` as the explicit constructor.
- Pass: `Dialog::children([...])` provides a root-level part-children builder that is closer to
  upstream nested children composition while still lowering into the existing recipe-layer slots.
- Pass: `DialogPart::content_with(...)` plus `DialogContent::with_children(...)`,
  `DialogHeader::with_children(...)`, and `DialogFooter::with_children(...)` keep the default
  copyable lane close to upstream nested children composition while preserving dialog-scope access
  for `DialogClose::from_scope()`.
- Pass: `Dialog::compose()` provides a recipe-level builder for part assembly without pushing
  shadcn-specific composition concerns into the lower-level mechanism contract.
- Pass: `DialogContent::build(...)` is the typed content-side companion on that same recipe lane,
  but it is now the secondary builder-first lane rather than the default copyable path.

### Dismissal behavior

- Pass: Escape dismiss is handled by the shared dismissible root (Radix-aligned outcome).
- Pass: Overlay click-to-dismiss is implemented by rendering a full-window barrier behind the
  content (default on).
- Pass: Base UI-compatible convenience alias `disable_pointer_dismissal(bool)` is provided and
  maps directly to `overlay_closable(!disable)`.
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) via
  `Dialog::on_dismiss_request(...)`. When set, Escape and overlay-click dismissal route through the
  handler and do not automatically close `open`.
- Pass: Open lifecycle callbacks are available via `Dialog::on_open_change` and
  `Dialog::on_open_change_complete` (Base UI `onOpenChange` + `onOpenChangeComplete`).

### Focus behavior

- Pass: Modal barrier scoping prevents underlay focus traversal (ADR 0068).
- Pass: On open, initial focus falls back to the first focusable descendant within the modal root
  (via `window_overlays` focus helpers).
- Pass: On close, focus restoration is deterministic to the trigger (modal close unmount path).

### Visual parity (new-york)

- Pass: Motion matches shadcn's `fade` + `zoom-in-95` / `zoom-out-95` outcomes (best-effort, tick
  driven).
- Pass: Default sizing matches the upstream `w-full max-w-[calc(100%-2rem)] sm:max-w-lg` intent via
  a padded center layout + `DialogContent` max-width.
- Pass: `DialogHeader` matches upstream `gap-2` and `text-center sm:text-left` outcomes without
  adding extra padding that would double-count the `DialogContent` `gap-4` grid spacing.

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn dialog::tests`
- `cargo test -p fret-ui-shadcn --lib dialog::tests::dialog_content_build_accepts_builder_first_sections_surface -- --exact`
- Contract test: `dialog_disable_pointer_dismissal_alias_maps_overlay_closable`
- Contract test: `dialog_open_change_events_emit_change_and_complete_after_settle`
- Contract test: `dialog_open_change_events_complete_without_animation`
- Contract test: `dialog_content_default_close_button_closes`
- Contract test: `dialog_content_show_close_button_false_hides_default_close`
- Shadcn Web chrome gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_dialog_demo_panel_chrome_matches`).
- Shadcn Web placement gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_dialog_demo_overlay_center_matches`).
- Radix Web overlay geometry gate: `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
  (`radix_web_dialog_open_geometry_matches_fret`).
- Gallery diag gate: `tools/diag-scripts/ui-gallery/dialog/ui-gallery-dialog-default-close-click.json`

## Authoring note: `from_scope()`

Fret now exposes `DialogClose::from_scope()` as recipe-layer sugar.

- Scope: only for parts whose semantic job is “close the current dialog”.
- Layering: this does **not** change the underlying mechanism contract; it only reads the current
  dialog content scope while rendering the recipe.
- Relationship to defaults: `DialogContent` now owns the upstream-style default corner close
  affordance; use `show_close_button(false)` when an explicit `DialogClose` should replace it.
- Escape hatch: `DialogClose::new(open)` remains the explicit constructor and should be preferred
  when building the part outside the dialog content subtree.
- Rollout: `SheetClose` and `DrawerClose` now reuse the same pattern via wrappers over
  `DialogClose::from_scope()`.
- Failure mode: `from_scope()` panics when rendered outside dialog content so misuse is caught
  early during development.

## Authoring note: `compose()`

`Dialog::children([...])` is now the default copyable root path for first-party docs/examples.

- Scope: root-level part collection that stays closer to upstream nested children composition.
- Parts: use `DialogPart::trigger(...)`, `DialogPart::content(...)`, and optional
  `DialogPart::portal(...)` / `DialogPart::overlay(...)`.
- Layering: this still lowers into the same recipe-layer trigger/content slots and does **not**
  change the underlying overlay/focus/dismiss mechanism.

The default content lane now prefers deferred child composition:

- `DialogPart::content_with(...)`
- `DialogContent::with_children(...)`
- `DialogHeader::with_children(...)`
- `DialogFooter::with_children(...)`

This keeps dialog-local close affordances on the same copyable lane as upstream nested JSX while
leaving `DialogContent::build(...)` / `DialogHeader::build(...)` / `DialogFooter::build(...)`
available for builder-first code that already has section builders.

`Dialog::compose()` remains a recipe-layer bridge for authors who want a more explicit builder
style than the raw closure root.

- Scope: ergonomics only; it lowers into `Dialog::into_element_parts(...)`.
- Default teaching path: first-party examples now prefer
  `Dialog::new_controllable(cx, None, false).children([DialogPart::trigger(...), DialogPart::content(...)])`.
- Focused follow-up lane: explicit `DialogTrigger` / `DialogPortal` / `DialogOverlay` ownership
  still stays documented through a dedicated `Parts` example rather than replacing the default
  copyable path.
- Layering: it does **not** change the underlying overlay/focus/dismiss mechanism.
- Limitation: this is still not a full React-style nested children API; Fret stores already-built
  elements and assembles them at the final call site.

## Follow-ups (recommended)

- Consider porting Base UI-style detached trigger handles (`Dialog.createHandle()` / detached
  trigger focus-restore ownership) if the dialog authoring surface needs to mirror the richer
  Base UI docs lane beyond the current shadcn page parity target.
- Consider exposing optional per-surface motion variants if recipes need diverging durations/easing.
