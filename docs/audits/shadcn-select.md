# shadcn/ui v4 Audit — Select


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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
- Pass: Label association remains caller-owned via `FieldLabel::for_control(...)` +
  `Select::control_id(...)`, and the real UI Gallery gate now passes under the shared
  `FieldLabel` forwarding fix: clicking the label focuses the trigger and opens the popup even
  inside the gallery's ambient pressable shell.
- Pass: root-level disabled gate now forces closed-state render semantics (content hidden and trigger
  not exposed as expanded), even when the controlled `open` model is `true`.
- Pass: Open lifecycle callbacks are available via `Select::on_open_change` and
  `Select::on_open_change_complete` (Base UI `onOpenChange` + `onOpenChangeComplete`).

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

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream docs path first: `Demo`, `Usage`, `Align Item With Trigger`,
  `Groups`, `Scrollable`, `Disabled`, `Invalid`, `RTL`, and `API Reference`.
- Pass: `Label Association`, `Field Builder Association`, and `Diag Surface` stay as explicit Fret follow-ups after
  `API Reference`, so first-party docs stay source-aligned without dropping the existing stable
  `test_id` surfaces used by `tools/diag-scripts/ui-gallery/select/*`.
- Pass: the default copyable lane stays on `Select::new(...)` / `new_controllable(...)` plus the
  direct builder chain, while `Select::into_element_parts(...)` + `SelectContent::with_entries(...)`
  remains the focused composable parts adapter when callers want the upstream nested call-site
  shape.
- Pass: no extra generic arbitrary-children API is warranted for `Select`; the option tree is
  intentionally typed as `SelectEntry` / `SelectGroup` / `SelectItem` / `SelectLabel` /
  `SelectSeparator`, and the mechanism/runtime work was already in place before this docs-surface
  alignment pass.
- Pass: richer item rows are now documented via typed `SelectItemText` / `SelectTextRun` surfaces in
  UI Gallery, which is sufficient for the current shadcn-aligned select lane without widening items
  to arbitrary child trees.

### Base UI advanced examples (non-blocking gaps)

- Not implemented by current public surface: Base UI's `multiple` example. Fret's `Select`
  remains a single-select `Model<Option<Arc<str>>>` recipe today; this is a public-surface choice,
  not an overlay/runtime defect.
- Not implemented by current public surface: Base UI's object-value + custom trigger renderer
  example (`defaultValue={plans[0]}`, `itemToStringValue(...)`, custom `SelectValue` children).
  Current Fret `Select` is intentionally text-keyed and does not yet expose object-valued items or
  trigger-side custom value rendering.
- Recommendation: treat both as a separate public-surface workstream. Do not widen the current
  shadcn-aligned `Select` recipe ad hoc while doing docs or mechanism parity. Use `Combobox` for
  multi-select/chips-style authoring today.

## Validation

- `cargo test -p fret-ui-shadcn --lib select`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app select_page_uses_typed_doc_sections_for_app_facing_snippets --status-level fail`
- `CARGO_TARGET_DIR=target-codex-fretboard cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/select/ui-gallery-select-label-click-focus.json --dir target/fret-diag-select-label-focus-20260320-1 --launch -- env CARGO_TARGET_DIR=target-codex-ui-gallery-select cargo run -p fret-ui-gallery`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/select/ui-gallery-select-docs-screenshots.json --dir target/fret-diag-select-docs-rich-items --session-auto --timeout-ms 240000 --launch -- cargo run -p fret-ui-gallery`
- Contract test: `select_disabled_hides_content_even_when_open_model_true`
- Contract test: `select_open_change_events_emit_change_and_complete_after_settle`
- Contract test: `select_open_change_events_complete_without_animation`
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

- Authoring-lane classification: keep `Select` on the direct recipe root/bridge lane.
  The default copyable path should now be `Select::new(...)` / `new_controllable(...)` plus the
  compact direct chain `.trigger(...).value(...).content(...).entries(...)`;
  `into_element_parts(...)` should remain the focused upstream-shaped adapter seam on that same
  lane, and Fret should not add a generic `compose()` root just for taxonomy symmetry.
- Public-surface workstream boundary: if Fret decides to pursue Base UI's `multiple` or object-value
  examples, land that as a separate surface-design task with explicit ownership and regression gates
  rather than folding it into routine shadcn docs parity work.
- Consider exposing a Radix-named `SelectContent`-style wrapper that defaults to item-aligned but
  still allows opting into popper mode for arrow-based skins.
