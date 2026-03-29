# shadcn/ui v4 Audit - Tabs


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- MUI Base UI: https://github.com/mui/base-ui
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Tabs` against upstream shadcn/ui v4 recipes and
Base UI `Tabs.Root` behavior.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/base/tabs.mdx`
- shadcn registry (new-york-v4): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tabs.tsx`
- shadcn demo: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/tabs-demo.tsx`
- Base UI root contract: `repo-ref/base-ui/packages/react/src/tabs/root/TabsRoot.tsx`

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/tabs.rs`
- Primitive semantics: `ecosystem/fret-ui-kit/src/primitives/tabs.rs`
- Roving/APG helpers: `ecosystem/fret-ui-kit/src/declarative/collection_semantics.rs`

## Audit checklist

### Composition & control model

- Pass: Exposes composable `TabsRoot` / `TabsList` / `TabsTrigger` / `TabsContent`, and keeps
  `Tabs` + `TabsItem` recipe builder for convenience.
- Note: Fret intentionally does not add a separate generic `compose()` builder for `Tabs` because
  the composable part surface already matches upstream authoring directly; the builder is purely a
  convenience layer, not a contract gap.
- Pass: Supports controlled selection via `Model<Option<Arc<str>>>` and uncontrolled `default_value`.
- Pass: Aligns with Base UI `onValueChange` intent via
  `Tabs::on_value_change(...)` and `TabsRoot::on_value_change(...)`.
- Pass: Adds source-aware callback parity layer via
  `Tabs::on_value_change_with_source(...)` and `TabsRoot::on_value_change_with_source(...)`
  to expose change origin (`RovingActiveChange` / `PointerDown` / `Activate`).
- Pass: Callback only emits when value actually changes (no duplicate emission on same selection).

### Keyboard & selection semantics

- Pass: Arrow roving + APG navigation are wired through `RovingFlex` + `cx.roving_nav_apg()`.
- Pass: `TabsActivationMode::Automatic` and `TabsActivationMode::Manual` map to expected behavior.
- Pass: `TabsOrientation::{Horizontal, Vertical}` and `loop_navigation(true)` are supported.
- Pass: `force_mount_content(true)` preserves inactive panels while gating layout/paint/semantics.

### Visual defaults (new-york-v4 parity)

- Pass: Root/list/trigger tokens align with shadcn v4 defaults (`h-9`, `p-[3px]`, muted list chrome,
  active trigger background/border/shadow conventions).
- Note: Default-style ownership remains split on purpose: root width constraints such as upstream
  `className="w-[400px]"` and the demo shell `w-full max-w-sm` stay caller-owned, while
  list/trigger/content chrome remains recipe-owned.
- Pass: `TabsContent` defaults to filling the remaining main-axis space (shadcn `flex-1` intent).
- Pass: Trigger content remains rich (icons/badges/custom children).

### Docs surface & composable authoring

- Pass: The UI Gallery page now mirrors the upstream docs path first:
  `Demo`, `Usage`, `Line`, `Vertical`, `Disabled`, `Icons`, `RTL`, and `API Reference`.
- Pass: The lead UI Gallery `Demo` snippet now keeps the upstream `w-full max-w-sm` shell and no
  longer forces a full-width `TabsList`.
- Pass: `Line`, `Vertical`, and `Disabled` now keep the same label/value shapes as the upstream
  docs examples instead of reusing gallery-only trigger content.
- Pass: `Icons` now demonstrates caller-owned icon + label trigger composition through
  `TabsItem::trigger_children(...)`, proving the existing composable trigger-children lane without
  widening the root API.
- Pass: `TabsRoot` / `TabsList` / `TabsTrigger` / `TabsContent` already cover the composable
  compound-parts lane, so no separate root `children([...])` API is needed for Tabs.
- Pass: The gallery keeps a copyable `Composable Parts (Fret)` follow-up after `API Reference` so
  custom trigger children stay discoverable without displacing the docs path.

## Known gaps

- Partial: Base UI `onValueChange` supports cancelation (`eventDetails.isCanceled`). Fret supports
  canceling a pending selection write via `Tabs::on_value_change_with_event_details(...)` /
  `TabsRoot::on_value_change_with_event_details(...)`, but older callbacks remain
  notification-only.

## Validation

- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn tabs_root_on_value_change_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_fires_once_when_selection_changes`
- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_with_source_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn tabs_root_on_value_change_with_source_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_with_source_reports_pointer_down`
- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_with_source_reports_roving_active_change`
- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_with_details_reports_activation_direction_on_pointer_down`
- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_with_details_reports_activation_direction_on_roving_active_change`
- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_with_details_reports_left_activation_direction_in_rtl_on_pointer_down`
- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_with_details_reports_left_activation_direction_in_rtl_on_roving_active_change`
- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_with_event_details_can_cancel_model_update`
- `cargo nextest run -p fret-ui-gallery tabs_page_uses_typed_doc_sections_for_app_facing_snippets`
- `cargo nextest run -p fret-ui-gallery tabs_demo_snippet_keeps_upstream_demo_width_lane_and_intrinsic_list`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-docs-smoke.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-icons-screenshots-zinc-light-dark.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-vertical-list-grows.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
- Web layout gates remain covered in `web_vs_fret_layout` tabs assertions.
