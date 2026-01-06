# Radix Primitives Audit — Tabs

This audit compares Fret's Radix-aligned tabs substrate against the upstream Radix `@radix-ui/react-tabs`
primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/tabs/src/tabs.tsx`
- Public exports: `repo-ref/primitives/packages/react/tabs/src/index.ts`

Key upstream concepts:

- `Tabs` root provides shared context: `value`, `onValueChange`, `orientation`, `activationMode`.
- `TabsList` installs a roving focus group (`loop` default `true`).
- `TabsTrigger` participates in roving focus and updates value on activation.
- `TabsContent` conditionally mounts/hides based on the selected value.

## Fret mapping

Fret does not use React context. Instead, tabs behavior is composed via:

- Mechanism layer (runtime): `crates/fret-ui` (`RovingFlex`, focus, event dispatch).
- Headless helpers: `ecosystem/fret-ui-kit/src/headless/roving_focus.rs`
- Declarative wiring helpers: `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs`
- Radix-named primitive facades: `ecosystem/fret-ui-kit/src/primitives/tabs.rs`

## Current parity notes

- Pass: Controlled selection via `Model<Option<Arc<str>>>` (Radix `value`).
- Pass: `orientation` and `activationMode` outcomes are modeled as enums.
- Pass: Active index derivation skips disabled items (roving focus parity).
- Pass: `TabsTrigger` selects on left mouse down and suppresses focus for other mouse downs
  (Radix `onMouseDown` behavior).
- Partial: Semantics roles (`TabList` / `Tab` / `TabPanel`) exist in the runtime; the current
  shadcn-aligned `Tabs` recipe wires these roles explicitly.
- Pass: `TabsContent` force-mount semantics are exposed as `tab_panel_with_gate(...)` and the
  composable `TabsContent::force_mount(...)` surface.

## Follow-ups (recommended)

- Done: Added a composable tabs surface (`TabsRoot` / `TabsList` / `TabsTrigger` / `TabsContent`)
  on top of these primitives to match Radix authoring ergonomics without hard-coding a visual skin.
