# shadcn/ui v4 Audit - Tabs

This audit compares Fret's shadcn-aligned `Tabs` against upstream shadcn/ui v4 recipes and
Base UI `Tabs.Root` behavior.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/tabs.mdx`
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
- Pass: Supports controlled selection via `Model<Option<Arc<str>>>` and uncontrolled `default_value`.
- Pass: Aligns with Base UI `onValueChange` intent via
  `Tabs::on_value_change(...)` and `TabsRoot::on_value_change(...)`.
- Pass: Callback only emits when value actually changes (no duplicate emission on same selection).

### Keyboard & selection semantics

- Pass: Arrow roving + APG navigation are wired through `RovingFlex` + `cx.roving_nav_apg()`.
- Pass: `TabsActivationMode::Automatic` and `TabsActivationMode::Manual` map to expected behavior.
- Pass: `TabsOrientation::{Horizontal, Vertical}` and `loop_navigation(true)` are supported.
- Pass: `force_mount_content(true)` preserves inactive panels while gating layout/paint/semantics.

### Visual defaults (new-york-v4 parity)

- Pass: Root/list/trigger tokens align with shadcn v4 defaults (`h-9`, `p-[3px]`, muted list chrome,
  active trigger background/border/shadow conventions).
- Pass: Trigger content remains rich (icons/badges/custom children).

## Known gaps

- Gap: Base UI `onValueChange` provides richer `eventDetails` (cancelation + activation metadata);
  Fret currently exposes only the next value callback.

## Validation

- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn tabs_root_on_value_change_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn tabs_on_value_change_fires_once_when_selection_changes`
- Web layout gates remain covered in `web_vs_fret_layout` tabs assertions.
