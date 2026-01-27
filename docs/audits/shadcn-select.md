# shadcn/ui v4 Audit — Select

This audit compares Fret’s shadcn-aligned `Select` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/select.mdx`
- Component wrapper (Radix Select skin): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/select.tsx`
- Demo usage: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/select-demo.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/select.rs`
- Key building blocks:
  - Overlay orchestration: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
  - Anchored placement: `ecosystem/fret-ui-kit/src/primitives/popper.rs`
  - Radix facade: `ecosystem/fret-ui-kit/src/primitives/select.rs`
  - Dismissal policy: `ecosystem/fret-ui-kit/src/primitives/dismissable_layer.rs`
  - Focus policy: `ecosystem/fret-ui-kit/src/primitives/focus_scope.rs`

## Audit checklist

### Placement & alignment

- Pass: Exposes `side`/`align` and both `side_offset(...)` + `align_offset(...)`, mapping to
  `PopperContentPlacement`.
- Pass: Exposes `position(...)` to switch between Radix `item-aligned` and `popper` positioning.
  Default is `SelectPosition::ItemAligned` to match the upstream wrapper default (`position="item-aligned"`).
- Pass: Uses per-window overlay roots (portal-like) via `OverlayController`.
- Pass: Optional diamond arrow rendering (`Select::arrow(true)`).

### Keyboard & roving navigation

- Pass: Trigger `Enter` / `Space` / `ArrowDown` / `ArrowUp` opens the popover on key down when
  closed (Radix `OPEN_KEYS` outcome; prevents the key-up activation from toggling it closed).
- Pass: Trigger typeahead while closed updates the selection without opening (Radix trigger
  typeahead behavior); `Space` is ignored while typing ahead.
- Pass: Listbox navigation uses roving focus + typeahead hooks (`cx.roving_nav_apg()` and
  `cx.roving_typeahead_prefix_arc_str(...)`).
- Pass: `loop_navigation(true)` defaults to looping behavior (Radix `loop` default).

### Selection & dismissal

- Pass: Selecting an item commits `model` and closes the overlay.
- Pass: Outside press dismissal is delegated to the shared dismissible overlay infra (ADR 0069).
- Pass: Select behaves like a Radix-style menu overlay: outside pointer-down is consumed (non-click-through).

### Visual parity (shadcn)

- Pass: Selected option shows a trailing checkmark (`ids::ui::CHECK`) and selection background.
- Pass: Overlay content animates with shadcn’s motion taxonomy (fade + zoom, plus side-based slide-in on enter).
- Pass: Content width behavior matches upstream: min width tracks the trigger, but long items can expand
  the listbox wider (item-aligned mode uses a longest-item width probe).
- Pass: Structural rows are supported via `SelectEntry` (`SelectLabel`, `SelectGroup`,
  `SelectSeparator`) rendered inside the listbox.
- Pass: Scroll buttons (`SelectScrollUpButton` / `SelectScrollDownButton`) are rendered for
  overflowing lists and scroll the viewport without dismissing the overlay.
- Pass: Supports both Radix positioning modes (`SelectPosition::ItemAligned` and
  `SelectPosition::Popper`). Arrow rendering is only available in popper mode.
- Pass: `aria-invalid=true` border and focus ring (including shadcn's invalid ring color overrides) match
  shadcn-web (`select-demo.invalid`, `select-demo.invalid-focus`).

## Validation

- `cargo test -p fret-ui-shadcn --lib select`
- Trigger chrome + focus ring gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome`
  (`web_vs_fret_select_scrollable_trigger_chrome_matches`, `web_vs_fret_select_demo_aria_invalid_border_color_matches`,
  `web_vs_fret_select_demo_focus_ring_matches`, `web_vs_fret_select_demo_aria_invalid_focus_ring_matches`).
- Overlay surface/shadow gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_select_scrollable_surface_colors_match_web`, `web_vs_fret_select_scrollable_surface_colors_match_web_dark`,
  `web_vs_fret_select_scrollable_shadow_matches_web`, `web_vs_fret_select_scrollable_shadow_matches_web_dark`,
  `web_vs_fret_select_scrollable_small_viewport_surface_colors_match_web`, `web_vs_fret_select_scrollable_small_viewport_surface_colors_match_web_dark`,
  `web_vs_fret_select_scrollable_small_viewport_shadow_matches_web`, `web_vs_fret_select_scrollable_small_viewport_shadow_matches_web_dark`,
  `web_vs_fret_select_scrollable_tiny_viewport_surface_colors_match_web`, `web_vs_fret_select_scrollable_tiny_viewport_surface_colors_match_web_dark`,
  `web_vs_fret_select_scrollable_tiny_viewport_shadow_matches_web`, `web_vs_fret_select_scrollable_tiny_viewport_shadow_matches_web_dark`).
- Highlighted option chrome gates (hover/highlight background + text color): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_select_demo_highlighted_option_chrome_matches_web`, `web_vs_fret_select_demo_highlighted_option_chrome_matches_web_dark`,
  `web_vs_fret_select_scrollable_highlighted_option_chrome_matches_web`, `web_vs_fret_select_scrollable_highlighted_option_chrome_matches_web_dark`).
- Web goldens (placement + scroll-button geometry): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  - Evidence:
    - `goldens/shadcn-web/v4/new-york-v4/select-demo.open.json`
    - `goldens/shadcn-web/v4/new-york-v4/select-scrollable.open.json`
    - Highlight-state goldens:
      - `goldens/shadcn-web/v4/new-york-v4/select-demo.highlight-first.open.json`
      - `goldens/shadcn-web/v4/new-york-v4/select-scrollable.highlight-first.open.json`
  - Gates:
    - `web_vs_fret_select_demo_overlay_placement_matches`
    - `web_vs_fret_select_demo_open_option_metrics_match`
    - `web_vs_fret_select_scrollable_listbox_width_matches`
    - `web_vs_fret_select_scrollable_listbox_option_insets_match` (plus small/tiny viewport variants)

## Follow-ups (recommended)

- Consider exposing a Radix-named `SelectContent`-style wrapper that defaults to item-aligned but
  still allows opting into popper mode for arrow-based skins.
