# shadcn/ui new-york-v4 Alignment Audit (Fret)

This audit tracks visual/behavior alignment gaps between:

- Upstream baseline: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/*.tsx`
- Fret recipes: `ecosystem/fret-ui-shadcn/src/*.rs`

Goal: align **default outcomes** (spacing, sizing, truncation, focus ring, indicator slots, overlay chrome)
for the `new-york-v4` preset, without expanding `fret-ui` mechanism scope.

## How to validate

- Run the component gallery: `cargo run -p fret-demo --bin components_gallery`
- Validate controls at multiple DPIs and with a “weird metrics” UI font (e.g. a Nerd Font) to catch
  baseline/centering issues.

## Global baseline rules (new-york-v4)

These patterns appear repeatedly across upstream components:

- Control height: `h-9` (default), `h-8` (sm), `h-10` (lg) for buttons; select trigger uses `h-9`/`h-8`.
- Radius: `rounded-md` for controls; `rounded-lg` for larger containers (e.g. dialog).
- Truncation: value/title areas use `min-w-0` + `overflow-hidden` + `whitespace-nowrap` + `truncate`
  (or `line-clamp-1`), especially for trigger/value slots inside a `justify-between` row.
- Icon sizing: default `size-4`, often with `opacity-50` for down chevrons.
- Indicator slots: checkbox/radio/select indicators are positioned via `absolute` + reserved padding
  (`pl-8` or `pr-8`) so layout does not jitter when toggling selection.
- Focus ring: `focus-visible:ring-[3px]` with `ring/50` and `border-ring`.

Fret mapping intent:

- Use `fret-ui-kit::recipes::input::resolve_input_chrome` (and/or shadcn theme presets) for shared
  control chrome and ring behavior.
- Use recipe-level layout helpers to guarantee “min width 0 + truncation” on value/title slots.
- Prefer stable indicator slot layout (reserve space, avoid `SpaceBetween` relying on icon presence).

## Component checklist (high impact)

### `Select`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/select.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/select.rs`
- Gaps to check:
  - Trigger: ensure value slot truncation is always enabled (no premature ellipsis).
  - Trigger: chevron icon size/opacity and gap to value.
  - Content: `p-1`, `rounded-md`, `border`, `shadow-md`, max-height behavior.
  - Items: `py-1.5`, `pl-2`, `pr-8`, `gap-2`, `rounded-sm`.
  - Selected indicator: absolute slot (`right-2`, `size-3.5`) + reserve `pr-8`.
  - Scroll buttons: `py-1` with centered `size-4` chevrons (only visible when scrollable).

### `DropdownMenu`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dropdown-menu.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- Gaps to check:
  - Content: `p-1`, `rounded-md`, `border`, `shadow-md`, max-height behavior.
  - Item row: `px-2 py-1.5`, `gap-2`, `rounded-sm`, destructive focus background tint.
  - Checkbox/radio indicators: absolute left slot (`left-2`, `size-3.5`) + reserve `pl-8`.
  - SubTrigger: right chevron `ml-auto size-4`, `data-[state=open]` accent background.
  - Shortcut: `ml-auto text-xs tracking-widest` alignment.

### `Input`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/input.rs`
- Gaps to check:
  - Ensure `min-w-0` equivalent for flex layouts.
  - Focus ring thickness (`3px`) and border color keys.
  - Placeholder color and selection colors.

### `Button`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/button.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/button.rs`
- Gaps to check:
  - Size: `h-9` baseline, icon-only sizing (`size-9`) behavior.
  - Variant mapping: outline uses border + shadow-xs; destructive uses dedicated ring color.
  - Focus ring thickness (`3px`) and ring/border keys.

### `Tabs`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tabs.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/tabs.rs`
- Gaps to check:
  - TabsList: `h-9`, `rounded-lg`, `p-[3px]`, `bg-muted`.
  - Trigger: `flex-1`, `h-[calc(100%-1px)]`, active background/border behavior.

### `Tooltip`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/tooltip.rs`
- Gaps to check:
  - Content chrome: `bg-foreground text-background`, `rounded-md`, `px-3 py-1.5`, `text-xs`.
  - Arrow: diamond rotated 45deg, size `2.5`, minor translate.

### `Dialog`

- Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx`
- Fret: `ecosystem/fret-ui-shadcn/src/dialog.rs`
- Gaps to check:
  - Overlay: `bg-black/50` (not fully opaque).
  - Content: centered, `rounded-lg`, `border`, `p-6`, `shadow-lg`, close button slot.

