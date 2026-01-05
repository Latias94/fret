# shadcn/ui v4 Audit — Tabs

This audit compares Fret’s shadcn-aligned `Tabs` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/tabs.mdx`
- Component wrapper (Radix Tabs skin): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tabs.tsx`
- Demo usage: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/tabs-demo.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/tabs.rs`
- Key building blocks:
  - Roving focus utilities: `ecosystem/fret-ui-kit/src/headless/roving_focus.rs`
  - APG navigation hooks: `ecosystem/fret-ui-kit/src/declarative/collection_semantics.rs`

## Audit checklist

### Composition surface

- Partial: Fret exposes a `Tabs` builder with `TabsItem` entries (label + content) rather than a
  fully composable `TabsList` / `TabsTrigger` / `TabsContent` surface like Radix/shadcn.
- Pass: Supports a controlled selection model via `Model<Option<Arc<str>>>`.

### Keyboard & selection behavior

- Pass: Arrow-key roving navigation is implemented via `RovingFlex` + `cx.roving_nav_apg()`.
- Pass: Supports Radix `activationMode` outcomes:
  - `TabsActivationMode::Automatic` updates the model while roving.
  - `TabsActivationMode::Manual` keeps selection stable until activation.
- Pass: Supports Radix `orientation` outcomes (`TabsOrientation::Horizontal` / `Vertical`).
- Pass: `loop_navigation(true)` defaults to looping behavior (Radix `loop` default).

### Visual defaults (shadcn parity)

- Pass: Root layout matches shadcn’s default (`flex` column + `gap-2`) via `component.tabs.gap`.
- Pass: List styling aligns with the wrapper:
  - Height (`h-9`) via `component.tabs.list_height` (fallback `36px`)
  - Padding (`p-[3px]`) via `component.tabs.list_padding` (fallback `3px`)
  - Background uses `muted` and inactive foreground uses `muted-foreground`.
- Pass: Trigger styling aligns with the wrapper’s active state defaults:
  - Active background uses `background`
  - Active border uses `input`/`border`
  - Active shadow uses the standard shadcn-ish `shadow-sm`
- Partial: Trigger content is label-only; upstream supports arbitrary children (icons, badges).

## Validation

- `cargo test -p fret-ui-shadcn --lib tabs`

## Follow-ups (recommended)

- Consider adding a composable surface (`TabsList` / `TabsTrigger` / `TabsContent`) to better match
  Radix/shadcn authoring ergonomics.
