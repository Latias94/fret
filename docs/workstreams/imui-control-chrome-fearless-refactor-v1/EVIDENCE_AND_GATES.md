# ImUi Control Chrome Fearless Refactor v1 - Evidence & Gates

Goal: keep the IMUI control-surface rewrite tied to one proof surface, one gate package, and one
shared owner split instead of letting demos paper over broken defaults.

## Evidence anchors (current)

- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/M0_BASELINE_AUDIT_2026-04-14.md`
- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/M1_IMUI_VS_IMGUI_COMPONENT_AUDIT_2026-04-14.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/DESIGN.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md`
- `docs/workstreams/control-chrome-normalization-audit-v1/control-chrome-normalization-audit-v1.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/standalone/ui-editor-egui-imgui-gap-v1.md`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/bullet_text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/control_chrome.rs`
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_store.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-authoring/src/lib.rs`
- `ecosystem/fret-ui-kit/tests/imui_bullet_text_smoke.rs`
- `ecosystem/fret-ui-kit/tests/imui_button_smoke.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`
- `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-control-discoverability.json`
- `tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-interaction-smoke.json`
- `tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-compact-shell-smoke.json`
- `tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-layout-compact-screenshot.json`
- `repo-ref/imgui/imgui_widgets.cpp`
- `repo-ref/imgui/imgui_demo.cpp`
- `repo-ref/egui/crates/egui/src/widgets/button.rs`
- `repo-ref/egui/crates/egui/src/widgets/slider.rs`
- `repo-ref/egui/crates/egui/src/containers/combo_box.rs`
- `repo-ref/egui/crates/egui/src/widgets/text_edit/mod.rs`

## First-open repro surfaces

Use these when reopening the lane:

1. Immediate interaction showcase
   - `cargo run -p fret-demo --bin imui_interaction_showcase_demo`
2. Immediate adapter proof
   - `cargo run -p fret-demo --bin imui_shadcn_adapter_demo`
3. Immediate interaction behavior proof
   - `cargo run -p fret-demo --bin imui_response_signals_demo`

These three surfaces together answer:

- does the control look interactive,
- does the shared surface still feel coherent in a compact editor rail,
- and did the visual rewrite accidentally break the interaction lifecycle surface?

## Current focused gates

### IMUI behavior and adapter gates

- `cargo nextest run -p fret-imui`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_combo_smoke`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_adapter_seam_smoke --test imui_combo_smoke`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_bullet_text_smoke --test imui_separator_text_smoke --test imui_button_smoke`
- `cargo nextest run -p fret-imui button_family_variants_and_radio_mount_with_expected_bounds`
- `python tools/gate_imui_shadcn_adapter_control_discoverability_source.py`
- `cargo nextest run -p fret-imui bullet_text_helper_renders_indicator_before_wrapped_label separator_text_helper_renders_label_with_trailing_rule button_family_variants_and_radio_mount_with_expected_bounds`
- `cargo nextest run -p fret-imui button_activate_shortcut_is_scoped_to_focused_button button_lifecycle_edges_follow_press_session long_press_sets_long_pressed_true_once_and_reports_holding checkbox_lifecycle_reports_edit_and_deactivated_after_edit right_click_sets_context_menu_requested_true_once`

This package protects:

- direct immediate interaction responses,
- IMUI adapter seams,
- combo-related shared helper behavior while the trigger surface is being rewritten,
- and the new Dear ImGui-style button-family/radio helpers.

### Showcase layout / screenshot gates

- `cargo nextest run -p fret-examples --lib showcase_responsive_layout`
- `python tools/gate_imui_facade_teaching_source.py`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-control-discoverability.json --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-interaction-smoke.json --launch -- cargo run -p fret-demo --bin imui_interaction_showcase_demo --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-compact-shell-smoke.json --launch -- cargo run -p fret-demo --bin imui_interaction_showcase_demo --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-layout-compact-screenshot.json --launch -- cargo run -p fret-demo --bin imui_interaction_showcase_demo --release`

This package protects:

- the current showcase responsive-layout expectations,
- the absence of the old fixed-width compact-lab workaround now that shared control chrome owns the
  compact rail,
- compact control discoverability and non-overlap on the adapter proof surface,
- compact-shell reachability,
- and the screenshot proof that the showcase remains reviewable at the default compact window.

## Executed on 2026-04-14

These commands were run after the shared control-chrome landing:

- `cargo check -p fret-ui-kit --features imui`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_bullet_text_smoke --test imui_separator_text_smoke --test imui_button_smoke`
- `cargo nextest run -p fret-imui bullet_text_helper_renders_indicator_before_wrapped_label separator_text_helper_renders_label_with_trailing_rule button_family_variants_and_radio_mount_with_expected_bounds`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_combo_smoke`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_adapter_seam_smoke --test imui_combo_smoke`
- `cargo nextest run -p fret-imui button_family_variants_and_radio_mount_with_expected_bounds`
- `cargo nextest run -p fret-imui button_activate_shortcut_is_scoped_to_focused_button button_lifecycle_edges_follow_press_session long_press_sets_long_pressed_true_once_and_reports_holding checkbox_lifecycle_reports_edit_and_deactivated_after_edit right_click_sets_context_menu_requested_true_once`
- `cargo nextest run -p fret-examples --lib showcase_responsive_layout`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-layout-compact-screenshot.json --launch -- cargo run -p fret-demo --bin imui_interaction_showcase_demo --release`
- `git diff --check -- ecosystem/fret-ui-kit/src/imui/options.rs ecosystem/fret-ui-kit/src/imui/control_chrome.rs ecosystem/fret-ui-kit/src/imui.rs ecosystem/fret-ui-kit/src/imui/button_controls.rs ecosystem/fret-ui-kit/src/imui/boolean_controls.rs ecosystem/fret-ui-kit/tests/imui_button_smoke.rs ecosystem/fret-imui/src/tests/composition.rs apps/fret-examples/src/imui_interaction_showcase_demo.rs`

## Executed on 2026-04-20

- `cargo nextest run -p fret-examples --lib imui_shadcn_adapter_demo_keeps_control_discoverability_proof_surface`
- `python tools/gate_imui_facade_teaching_source.py`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-control-discoverability.json --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-layout-compact-screenshot.json --launch -- cargo run -p fret-demo --bin imui_interaction_showcase_demo --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-compact-shell-smoke.json --launch -- cargo run -p fret-demo --bin imui_interaction_showcase_demo --release`
- `git diff --check -- apps/fret-examples/src/imui_interaction_showcase_demo.rs apps/fret-examples/src/lib.rs`

Artifacts left behind by the 2026-04-20 adapter discoverability gate:

- run result:
  `target/fret-diag/1776672988456/script.result.json`
- before layout sidecar:
  `target/fret-diag/1776672988679-imui-shadcn-adapter.discoverability.before/layout.taffy.v1.json`
- before bundle:
  `target/fret-diag/1776672988687-imui-shadcn-adapter.discoverability.before/bundle.schema2.json`
- before screenshot:
  `target/fret-diag/screenshots/1776672988691-imui-shadcn-adapter.discoverability.before/window-4294967297-tick-21-frame-22.png`
- after layout sidecar:
  `target/fret-diag/1776672988815-imui-shadcn-adapter.discoverability.after/layout.taffy.v1.json`
- after bundle:
  `target/fret-diag/1776672988824-imui-shadcn-adapter.discoverability.after/bundle.schema2.json`
- after screenshot:
  `target/fret-diag/screenshots/1776672988827-imui-shadcn-adapter.discoverability.after/window-4294967297-tick-36-frame-37.png`

Artifacts left behind by the 2026-04-20 compact showcase fixed-rail re-audit:

- run result:
  `target/fret-diag/1776674580589/script.result.json`
- comparison bundle before the re-audit:
  `target/fret-diag/1776673936986-imui-interaction-showcase.compact/bundle.schema2.json`
- compact layout sidecar after the re-audit:
  `target/fret-diag/1776674580966-imui-interaction-showcase.compact-layout/layout.taffy.v1.json`
- compact bundle after the re-audit:
  `target/fret-diag/1776674580976-imui-interaction-showcase.compact/bundle.schema2.json`
- compact screenshot after the re-audit:
  `target/fret-diag/screenshots/1776674580981-imui-interaction-showcase.compact/window-4294967297-tick-39-frame-38.png`

This artifact set captures the main conclusion of the re-audit:

- the old comparison bundle still shows the workaround-era compact shell/lab semantic width near
  the old `320px` rail (`318px` bounds at the card level, `270px` on representative controls),
- while the new compact layout sidecar shows the shared rail hitting its `352px` cap (`704px` at
  the x2 layout capture scale),
- and the new compact bundle shows the semantic shell/lab width widening to `350px` with inner
  controls widening to `302px`.

Artifacts left behind by the 2026-04-20 compact-shell reachability gate:

- run result:
  `target/fret-diag/1776674616593/script.result.json`
- compact-shell bundle:
  `target/fret-diag/1776674617108-imui-interaction-showcase.compact-shell/bundle.schema2.json`
- compact-shell screenshot:
  `target/fret-diag/screenshots/1776674617117-imui-interaction-showcase.compact-shell/window-4294967297-tick-52-frame-52.png`

Artifacts left behind by the 2026-04-14 compact showcase screenshot gate:

- session:
  `target/fret-diag/sessions/1776138817580-24424`
- bundle dir:
  `target/fret-diag/sessions/1776138817580-24424/1776138987735-imui-interaction-showcase.compact`
- screenshot:
  `target/fret-diag/sessions/1776138817580-24424/screenshots/1776138987735-imui-interaction-showcase.compact/window-4294967297-tick-40-frame-39.png`

## Missing gates that should become real

### Narrow-width field regression gate

If the shared field-width policy changes while the lane is active, add a dedicated gate that proves
slider/combo/input surfaces stay within the compact lab rail without clipping or accidental
zero-width layout.
