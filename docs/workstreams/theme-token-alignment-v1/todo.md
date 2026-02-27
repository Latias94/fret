# Theme Token Alignment v1 — TODO + Tracker

This file is the living execution tracker for `theme-token-alignment-v1`.

Read first:

- Plan/rules: `docs/workstreams/theme-token-alignment-v1/design.md`
- Milestones: `docs/workstreams/theme-token-alignment-v1/milestones.md`

## How to audit (repeatable workflow)

1) Identify the upstream intent.

- Literal Tailwind class (e.g. `text-white`, `bg-black/50`) => use named literal colors.
- Semantic class (e.g. `text-muted-foreground`) => use semantic palette tokens.
- Variant rule with `dark:*` deltas (e.g. `dark:bg-destructive/60`) => use component-derived tokens seeded by the preset.

2) Decide the owning layer.

- Mechanism/token resolution => `crates/fret-ui`
- Authoring glue (`ColorRef`, token read surfaces) => `ecosystem/fret-ui-kit`
- Recipes/variants => `ecosystem/*` (e.g. `ecosystem/fret-ui-shadcn`)
- Preset seeding for `component.*` => the ecosystem preset builder (e.g. `shadcn_themes.rs`)

3) Add a gate.

- Prefer a Rust test for token seeding outcomes.
- Add `fretboard diag` screenshot scripts for high-signal visual outcomes under zinc/dark.

## Useful scans (fast inventory)

- Find literal Tailwind uses in repo refs (source of truth):
  - `rg -n "text-white|bg-white|bg-black/|text-black" repo-ref/ui/apps/v4/registry/new-york-v4/ui -S`
- Find likely problematic patterns in our recipes:
  - `rg -n "destructive-foreground|color_token\\(\"white\"\\)|color_token\\(\"black\"\\)" ecosystem -S`
  - `rg -n "ThemeNamedColorKey|ColorRef::Named\\(" ecosystem -S` (already-migrated areas)

## Status legend

- `[ ]` Not started
- `[~]` In progress
- `[x]` Done
- `[?]` Needs triage / unclear ownership

## Tracker table (ecosystem-wide)

| Area | Component | Upstream evidence (repo-ref) | Literal/derived rule | Target tokens | Gate | Status | Evidence anchors |
|---|---|---|---|---|---|---|---|
| shadcn | Button (destructive) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/button.tsx` | `text-white`, `dark:bg-destructive/60` | `named:white`, `component.button.destructive.bg` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/button.rs`, `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`, `tools/diag-scripts/ui-gallery-button-variants-width-zinc-dark.json` |
| shadcn | Button (outline, dark deltas) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/button.tsx` | `dark:bg-input/30`, `dark:border-input`, `dark:hover:bg-input/50` | `component.button.outline.*` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/button.rs`, `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`, `tools/diag-scripts/ui-gallery-button-outline-screenshot-zinc-dark.json` |
| shadcn | Badge (destructive) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/badge.tsx` | `text-white`, `dark:bg-destructive/60` | `named:white`, `component.badge.destructive.bg` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/badge.rs`, `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`, `tools/diag-scripts/ui-gallery-badge-destructive-screenshot-zinc-dark.json` |
| shadcn | Dialog / Sheet / AlertDialog (scrim) | `repo-ref/ui` overlay recipes | `bg-black/50` | `named:black` + alpha | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/dialog.rs`, `ecosystem/fret-ui-shadcn/src/sheet.rs`, `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` |
| shadcn | Drawer (scrim) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/drawer.tsx` | `bg-black/50` | `named:black` + alpha | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/drawer.rs`, `ecosystem/fret-ui-shadcn/src/sheet.rs` |
| shadcn | Slider (thumb) | `repo-ref/ui` slider recipe | `bg-white` | `named:white` | Rust test or web-vs-fret layout | [x] | `ecosystem/fret-ui-shadcn/src/slider.rs` |
| shadcn | Input (dark bg) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input.tsx` | `dark:bg-input/30` | `component.input.bg` (seeded by preset) | Rust test (preset seeding) | [x] | `ecosystem/fret-ui-kit/src/recipes/input.rs`, `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs` |
| shadcn | Textarea (dark bg) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/textarea.tsx` | `dark:bg-input/30` | `component.input.bg` (via input-family resolver) | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/textarea.rs`, `ecosystem/fret-ui-kit/src/recipes/input.rs` |
| shadcn | Select (trigger dark bg) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/select.tsx` | `dark:bg-input/30`, `dark:hover:bg-input/50` | `component.input.bg` + `component.input.bg_hover` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/select.rs`, `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`, `tools/diag-scripts/ui-gallery-select-trigger-hover-screenshot-zinc-dark.json` |
| shadcn | NativeSelect (trigger dark bg) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/native-select.tsx` | `dark:bg-input/30`, `dark:hover:bg-input/50` | `component.input.bg` + `component.input.bg_hover` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/native_select.rs`, `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`, `tools/diag-scripts/ui-gallery-native-select-trigger-hover-screenshot-zinc-dark.json` |
| shadcn | Input OTP (cells dark bg) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input-otp.tsx` | `dark:bg-input/30`, `ring/50` | `component.input.bg`, `ring/50` | Rust test (ring/50 seeded) | [x] | `ecosystem/fret-ui-shadcn/src/input_otp.rs`, `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs` |
| shadcn | Input Group (dark bg) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input-group.tsx` | `dark:bg-input/30` | `component.input.bg` (via input-family resolver) | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/input_group.rs`, `ecosystem/fret-ui-kit/src/recipes/input.rs` |
| shadcn | Checkbox (unchecked bg) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/checkbox.tsx` | `dark:bg-input/30` | `component.input.bg` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/checkbox.rs`, `tools/diag-scripts/ui-gallery-checkbox-basic-screenshots-zinc-dark.json` |
| shadcn | RadioGroup (unchecked bg) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/radio-group.tsx` | `dark:bg-input/30` | `component.input.bg` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/radio_group.rs`, `tools/diag-scripts/ui-gallery-field-radio-screenshot-zinc-dark.json` |
| shadcn | Combobox (input-like chrome) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/combobox.tsx` | `dark:bg-input/30` | `component.input.bg` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/combobox.rs`, `tools/diag-scripts/ui-gallery-combobox-trigger-screenshot-zinc-dark.json` |
| shadcn | NavigationMenu (trigger open bg) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/navigation-menu.tsx` | `data-[state=open]:bg-accent/50` | `component.navigation_menu.trigger.bg_open` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`, `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`, `tools/diag-scripts/ui-gallery-navigation-menu-open-bg-screenshot-zinc-dark.json`, `target/fret-diag/share/1772163781950.zip` |
| shadcn | Menubar (trigger open bg) | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/menubar.tsx` | `data-[state=open]:bg-accent` | `accent` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/menubar.rs`, `tools/diag-scripts/ui-gallery-menubar-open-bg-screenshot-zinc-dark.json`, `target/fret-diag/share/1772166152176.zip` |
| shadcn | Popover | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/popover.tsx` | `bg-popover`, `text-popover-foreground` | `popover`, `popover-foreground` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/popover.rs`, `ecosystem/fret-ui-shadcn/src/ui_builder_ext/surfaces.rs`, `tools/diag-scripts/ui-gallery-popover-basic-open-screenshot-zinc-dark.json`, `target/fret-diag/share/1772167841043.zip` |
| shadcn | DropdownMenu | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dropdown-menu.tsx` | `data-[variant=destructive]:focus:bg-destructive/10`, `dark:* /20` | `component.menu.destructive_focus_bg` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`, `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`, `tools/diag-scripts/ui-gallery-dropdown-menu-demo-open-screenshot-zinc-dark.json`, `target/fret-diag/share/1772172354054.zip` |
| shadcn | Tooltip / HoverCard | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/hover-card.tsx` | `bg-popover`, `text-popover-foreground` | `popover`, `popover-foreground` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/tooltip.rs`, `ecosystem/fret-ui-shadcn/src/hover_card.rs`, `tools/diag-scripts/ui-gallery-tooltip-demo-open-arrow-screenshot-zinc-dark.json`, `tools/diag-scripts/ui-gallery-hovercard-demo-screenshot-zinc-dark.json`, `target/fret-diag/share/1772167862626.zip`, `target/fret-diag/share/1772167887013.zip` |
| shadcn | Command | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/command.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/command-demo.tsx` | `bg-popover text-popover-foreground`, input icon `opacity-50` | `popover`, `popover-foreground` | diag screenshot (zinc/dark) | [x] | `ecosystem/fret-ui-shadcn/src/command.rs`, `tools/diag-scripts/ui-gallery-command-docs-demo-icons-screenshots-zinc-dark.json`, `tools/diag-scripts/ui-gallery-command-docs-demo-shortcuts-screenshots-zinc-dark.json`, `target/fret-diag/share/1772179040044.zip`, `target/fret-diag/share/1772179062288.zip` |
| shadcn-ai | AI Elements (messages + code blocks) | TODO: pin ai-elements upstream under `repo-ref/` | bubble `bg-secondary` should imply `text-secondary-foreground` | `secondary`, `secondary-foreground` | diag screenshot (zinc/dark) | [~] | `ecosystem/fret-ui-ai/src/elements/message.rs`, `tools/diag-scripts/ui-gallery-ai-message-demo-screenshot-zinc-dark.json`, `tools/diag-scripts/ui-gallery-ai-code-block-demo-screenshot-zinc-dark.json`, `target/fret-diag/share/1772180842135.zip`, `target/fret-diag/share/1772181265718.zip` |
| ecosystem | Markdown (inline code + links) | (ecosystem) | audit semantic vs literal color rules | tbd | diag screenshot (zinc/dark) | [ ] | `ecosystem/fret-markdown/src/theme.rs`, `ecosystem/fret-markdown/src/components.rs` |
| ecosystem | CodeView / Syntax (tokens contrast) | (ecosystem / vscode-theme) | audit syntax token palette contrast | tbd | diag screenshot (zinc/dark) | [ ] | `ecosystem/fret-code-view/src/syntax.rs`, `ecosystem/fret-vscode-theme/src/lib.rs` |
| material3 | Overlay scrim + surface contrast | (spec / repo-ref/material-ui / compose) | audit literal/derived assumptions | tbd | tbd | [ ] |  |
| charts/plot | Chart chrome + legend contrast | (ecosystem ports) | audit literal/derived assumptions | tbd | tbd | [ ] |  |

## Open questions / decision gates

- Do we need additional named literal colors beyond `white` and `black`?
  - Default: no. Only add with upstream evidence + multi-ecosystem need.
- Do we standardize a cross-ecosystem namespace for “surface-on-accent” like `on_destructive`?
  - Default: prefer semantic palette keys where the intent is semantic; avoid duplicating roles.
