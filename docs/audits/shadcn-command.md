# shadcn/ui v4 Audit - Command (cmdk)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- cmdk: https://github.com/pacocoursey/cmdk
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Command` surface (backed by cmdk-style behavior) against
the upstream shadcn/ui v4 docs and registry implementations, focusing on **behavioral outcomes**
instead of API compatibility.

## Upstream references (source of truth)

- shadcn docs surface: `repo-ref/ui/apps/v4/content/docs/components/{base,radix}/command.mdx`
- shadcn registry recipe surface: `repo-ref/ui/apps/v4/registry/bases/{base,radix}/ui/command.tsx`
- shadcn docs demo surface: `repo-ref/ui/apps/v4/registry/bases/{base,radix}/examples/command-example.tsx`
- Base UI headless/docs reference:
  `repo-ref/base-ui/docs/src/app/(docs)/react/components/autocomplete/demos/command-palette/tailwind/index.tsx`
- cmdk repo: `repo-ref/cmdk`

Key upstream semantics:

- `Command` is a composite (input + results list) where focus typically stays in the input.
- Highlight moves via `aria-activedescendant` (active descendant); `Enter` activates the highlighted item.
- Filtering/ranking is fuzzy-scored; `value`/`keywords` participate in matching.
- Structure primitives exist: `CommandGroup` (optional heading), `CommandSeparator`, `CommandEmpty`.
- Visual conventions commonly used by shadcn: optional check indicator and right-side shortcut text.

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/command.rs`
- Headless scoring/selection: `ecosystem/fret-ui-kit/src/headless/cmdk_score.rs`,
  `ecosystem/fret-ui-kit/src/headless/cmdk_selection.rs`
- Active descendant contract: `docs/adr/0073-active-descendant-and-composite-widget-semantics.md`

## Audit checklist

### Composition & navigation (cmdk-style)

- Pass: `CommandPalette` is a unified implementation (input + list) keeping focus in the input and driving highlight via active-descendant.
- Pass: `ArrowUp/Down`, `Home/End`, `PageUp/PageDown` update highlight; `Enter` triggers `on_select` / `command`.
- Pass: Hover moves highlight without stealing focus (matches cmdk expectations).

### Filtering / ranking semantics

- Pass: `CommandPalette` provides built-in filtering/ranking via `fret-ui-kit::headless::cmdk_score`.
- Pass: `CommandItem.value` participates as an alias when `value != label`.
- Pass: `CommandItem.keywords([...])` aligns with cmdk's `keywords` semantics (matching can hit non-label strings).

### Structure primitives (Group / Separator / Empty)

- Pass: Supports `CommandGroup` (optional heading) and `CommandSeparator`, and trims leading/trailing/consecutive separators after filtering.
- Pass: Shows `empty_text` when no results; provides `CommandEmpty` as a convenience (`CommandPalette::empty(...)`).

### Layout & geometry (docs surface + registry defaults)

- Pass: `CommandDemo` matches the public shadcn docs example surface for the copyable Gallery lane:
  `max-w-sm`, rounded border, and the icon/disabled/shortcut content shape used by
  `examples/{base,radix}/command-demo.tsx`.
- Pass: `CommandDemo` still matches the upstream split sizing: input wrapper uses `h-9` while the
  input uses `h-10` (the input overflows the wrapper slightly in the web golden).
- Pass: `CommandDialog` matches the upstream overrides (`h-12` wrapper + `h-12` input, and `pt-0` for sibling groups).
- Pass: `Command` root chrome (rounded border + popover background) stays recipe-owned because upstream defines it in the component source.
- Pass: `CommandPalette` defaults to a `w-full` root layout to avoid cmdk listbox width collapse when embedded in recipes (e.g. `Combobox`).
- Note: Width caps such as upstream `max-w-sm` remain caller-owned; gallery `Usage` applies that at the call site rather than baking it into the recipe root.
- Note: the newer `registry/new-york-v4/examples/command-demo.tsx` currently adds sink/demo-local
  chrome overrides (`shadow-md`, `md:min-w-[450px]`). Treat that as a separate example-surface
  choice, not as the default docs-surface width contract for the copyable Gallery page.

### Visual/content conventions (shadcn)

- Pass: `CommandItem.checkmark(bool)` + `CommandShortcut` support the common "left check + right shortcut" row layout.
- Pass: `CommandItem.children(...)` allows rich custom row content.
- Pass: Row-level children composability is already sufficient for upstream item chrome/custom layout
  parity; no additional root-level children API is required just to match shadcn row content.
- Pass: Highlighted rows use `accent` background and `accent-foreground` text (cmdk `data-[selected=true]` parity).
- Pass: Default item icons stay `muted-foreground` even when the row is highlighted (aligns shadcn `[_svg:not([class*='text-'])]:text-muted-foreground`).
- Pass: Default `CommandPalette` rows can render cmdk-style match highlighting (matched characters use `foreground`; non-matched characters use `muted-foreground`).

### CommandDialog

- Pass: `CommandDialog` supports `entries(...)`, so `CommandGroup/Separator/...` can be used inside a dialog.
- Pass: `CommandDialog` exposes open lifecycle callbacks via
  `CommandDialog::on_open_change`, `CommandDialog::on_open_change_with_reason`, and
  `CommandDialog::on_open_change_complete`
  (delegated to the aligned `Dialog` lifecycle semantics).
- Pass: `CommandPalette::list_viewport_test_id(...)` and `CommandDialog::list_viewport_test_id(...)`
  now expose the internal scroll viewport semantics surface, so diagnostics can target actual
  scroll state instead of the outer listbox wrapper.

## Validation

- `cargo test -p fret-ui-shadcn --lib command::tests`
- Contract test: `command_dialog_open_change_builders_set_handlers`
- Contract test: `command_dialog_test_id_builders_forward_to_palette_semantics`
- Contract test: `command_item_children_surface_renders_custom_row_while_preserving_option_label`
- Contract test: `command_palette_list_viewport_test_id_mounts_scroll_semantics_surface`
- Reason mapping test: `command_dialog_open_change_reason_maps_dismiss_reasons`
- Reason behavior test: `command_dialog_open_change_with_reason_reports_item_press_when_close_on_select`
- Curated stable subset suite entry (docs screenshots + keybindings + RTL):
  - `cargo run -p fretboard -- diag suite ui-gallery-command --dir target/fret-diag-suite-ui-gallery-command --session-auto --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery`
- UI Gallery docs-surface screenshot scripts (existing gate; keep navigation on `click_stable` so the run lands on the `Command` page deterministically):
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/command/ui-gallery-command-docs-demo-icons-screenshots.json --dir target/fret-diag-command-docs-icons-light --session-auto --pack --ai-packet --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/command/ui-gallery-command-docs-demo-icons-screenshots-zinc-dark.json --dir target/fret-diag-command-docs-icons-dark --session-auto --pack --ai-packet --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/command/ui-gallery-command-docs-demo-shortcuts-screenshots.json --dir target/fret-diag-command-docs-shortcuts-light --session-auto --pack --ai-packet --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/command/ui-gallery-command-docs-demo-shortcuts-screenshots-zinc-dark.json --dir target/fret-diag-command-docs-shortcuts-dark --session-auto --pack --ai-packet --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery`
- UI Gallery behavior/diagnostics scripts:
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/command/ui-gallery-command-scrollable-filter-clamps-scroll.json --dir target/fret-diag-command-scrollable --session-auto --pack --ai-packet --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/command/ui-gallery-command-palette-force-mount-item-visible.json --dir target/fret-diag-command-force-mount --session-auto --pack --ai-packet --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/command/ui-gallery-command-palette-separator-always-render-visible.json --dir target/fret-diag-command-separator --session-auto --pack --ai-packet --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery`
- shadcn-web golden + gates:
  - Golden: `goldens/shadcn-web/v4/new-york-v4/command-demo.json`
  - Layout gates:
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout -- web_vs_fret_layout_command_demo_input_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout -- web_vs_fret_layout_command_demo_listbox_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout -- web_vs_fret_layout_command_demo_listbox_option_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout -- web_vs_fret_layout_command_demo_listbox_option_insets_match`
  - Golden: `goldens/shadcn-web/v4/new-york-v4/command-dialog.open.json`
  - Chrome gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome -- web_vs_fret_command_dialog_panel_chrome_matches`
  - Placement gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement -- web_vs_fret_command_dialog_overlay_center_matches`
  - List metrics gates (v4 dialog surface overrides):
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement -- web_vs_fret_command_dialog_input_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement -- web_vs_fret_command_dialog_listbox_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement -- web_vs_fret_command_dialog_listbox_option_height_matches`
    - `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement -- web_vs_fret_command_dialog_listbox_option_insets_match`
    - Tiny viewport variants: `*_tiny_viewport` (ensures the web-style “centered even when overflowing” behavior stays aligned)
  - UI Gallery docs-surface gate:
    - `cargo nextest run -p fret-ui-gallery --lib gallery_command_core_examples_keep_upstream_aligned_targets_present`
    - `cargo nextest run -p fret-ui-gallery --lib gallery_command_docs_demo_keeps_upstream_max_width --features gallery-chart`
    - `cargo nextest run -p fret-ui-gallery --lib gallery_command_basic_opens_dialog_with_default_recipe_a11y_label --features gallery-chart`
    - `cargo nextest run -p fret-ui-gallery --lib gallery_command_follow_up_sections_remain_explicit_after_docs_aligned_examples --features gallery-chart`

## Follow-ups (non-P0)

- Authoring-lane classification: keep `Command` on the direct recipe root/bridge lane.
  `command(...)` / `CommandPalette` remain the default story; if split `CommandInput/List/Item`
  authoring is ever promoted, it should happen behind an explicit shared context contract rather
  than ad-hoc surface growth.
- Docs surface: UI Gallery now mirrors the upstream docs path after skipping `Installation`
  (`Demo`, `About`, `Usage`, `Basic`, `Shortcuts`, `Groups`, `Scrollable`, `RTL`, `API Reference`)
  before Fret-only `Loading` / `Action-first` follow-ups; the default copyable `Usage` lane stays
  explicit about the root-story divergence (`CommandPalette` instead of literal split children).
- Manual shell lane: UI Gallery now also exposes an explicit `Composable Shell (Fret)` follow-up
  that demonstrates the current lower-level `Command` + `CommandInput` + `CommandList` split lane
  with a shared query model for filtering/highlighting, while keeping the page explicit that this is
  not yet a full cmdk-equivalent shared-context API.
- Source axes: the Gallery page now states explicitly that the lead `Demo` follows the public docs
  example surface, while recipe-owned chrome/defaults stay aligned and tested against the registry
  source separately. This avoids conflating docs-example width caps with sink/demo-local registry
  example overrides.
- Demo surface: the lead `Demo` snippet now keeps the upstream `max-w-sm` width cap and stops
  overriding root chrome with a gallery-local shadow; recipe-owned border/radius remain on the
  component default lane.
- Basic surface: the `Basic` snippet now mirrors the upstream minimal dialog teaching lane more
  closely by using an explicit `Open Menu` trigger, a `Suggestions` group with the three upstream
  rows, and the recipe-default dialog a11y label (`Command palette`) instead of a gallery-local
  label override.
- Example surface split: `Shortcuts`, `Groups`, `Scrollable`, and `RTL` now stay on the upstream
  docs lane, while the previous Fret-only cmdk behavior demos (`disablePointerSelection`,
  controlled active value, `shouldFilter=false`, and `forceMount`) move to an explicit
  `Behavior Demos` follow-up section instead of overloading the docs examples.
- Recipe surface: `CommandDialog` now forwards the same palette test-id builders
  (`test_id_input`, `list_test_id`, `test_id_item_prefix`, `test_id_heading_prefix`) so
  docs-aligned dialog examples can remain automation-friendly without dropping back to embedded
  `CommandPalette` just for selectors.
- Diagnostics surface: `CommandPalette` / `CommandDialog` now also forward an explicit
  `list_viewport_test_id(...)` seam for the internal scroll viewport; scripts that assert scroll
  position should target that viewport semantics node rather than the outer listbox wrapper.
- Composability: if split authoring (`CommandInput`/`CommandList`/`CommandItem`) becomes a goal, introduce a shared context model (query + active + selection) with an explicit contract/ADR first.
- Composability status: `CommandInput` / `CommandList` remain available as lower-level shell pieces
  and for legacy roving-list use, but they are not a promoted cmdk-equivalent split children API.
- Composability status: treat row-level and root-level composability as separate concerns.
  `CommandItem::children(...)` already ships and is enough for item-local composition; the deferred
  gap is only the shared root context that would let split `CommandInput` / `CommandList` /
  `CommandEmpty` / `CommandGroup` authoring behave like upstream cmdk without manual wiring.
