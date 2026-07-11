# P3 Design Surface Readiness - 2026-05-06

Status: design/theme readiness audit; no style-system refactor opened yet
Last updated: 2026-05-06

## Decision

Do not copy Dear ImGui's mutable `ImGuiStyle` / `PushStyleVar` / `PushStyleColor` stack into Fret.

Fret's current design direction is the right one:

- generic IMUI chrome stays small and theme-derived in `fret-ui-kit::imui`,
- editor-grade density and visual policy live in `fret-ui-editor`,
- demos that need imgui-class density opt into `EditorThemePreset::ImguiLikeDense`,
- per-widget tuning should use typed options or editor token patches, not frame-local global style
  mutation.

This is enough for the active editor proof. A future design-system follow-on should start from
visual evidence, not from mirroring Dear ImGui's style API surface.

## Current Design Layers

| Layer | Current source | Purpose | Keep / avoid |
| --- | --- | --- | --- |
| Generic IMUI chrome | `ecosystem/fret-ui-kit/src/imui/control_chrome.rs`, `text_controls.rs`, `slider_controls.rs` | Compact immediate widget baseline, response-driven visuals, theme-derived text/input chrome | Keep as low-level policy; avoid turning it into a broad design system |
| IMUI disabled/hover timing | `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`, `interaction_runtime/disabled.rs`, `response/hover.rs` | Dear ImGui-style disabled alpha and hover-delay semantics | Keep in kit policy; do not widen `crates/fret-ui` |
| Editor tokens | `ecosystem/fret-ui-editor/src/primitives/tokens.rs`, `primitives/style.rs`, `primitives/chrome.rs` | Shared density, text-field chrome, popup, inspector/property-grid metrics | Keep as editor-owned design vocabulary |
| Editor theme presets | `ecosystem/fret-ui-editor/src/theme.rs` | `Default` plus `ImguiLikeDense`, layered as `ThemeConfig` patches over host themes | Keep opt-in; avoid defaulting all app UI to dense editor chrome |
| Teaching/proof usage | `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs` | Cookbook/editor proof opt into `ImguiLikeDense` | Keep as proof-local design intent |

## Source-Backed Facts

- `EditorThemePreset::ImguiLikeDense` exists in `fret-ui-editor` and changes row height,
  padding, hit thickness, text-field radius, popup radius/shadow, property-grid metrics, checkbox
  metrics, and editor color tokens.
- `install_editor_theme_preset(...)` stores the selected preset so it can be reapplied after a
  host theme reset.
- `apps/fret-cookbook/examples/imui_editor_controls_basics.rs` installs
  `EditorThemePreset::ImguiLikeDense`.
- `apps/fret-examples/src/imui_editor_proof_demo.rs` defaults its editor proof theme to
  `EditorThemePreset::ImguiLikeDense` unless `FRET_IMUI_EDITOR_PRESET` selects another preset.
- Generic IMUI text input/textarea chrome is compact and intentionally avoids focus-ring styling
  inside the immediate chrome.
- Dear ImGui exposes style as mutable runtime state via `ImGuiStyle`, style colors, style vars, and
  `ShowStyleEditor()`. Fret should translate the useful outcome into tokens/tools, not stack
  mutation.

## Follow-On Threshold

Open a design-specific follow-on only when one of these happens:

- two first-party IMUI/editor proofs need the same new density/chrome token,
- a screenshot or diagnostics visual gate shows the dense editor preset drifting,
- a user-facing theme/preset selector is needed in DevTools or a demo shell,
- a component needs a typed visual option that cannot be represented by existing editor tokens,
- style-editor functionality becomes a real product/debugging workflow rather than parity pressure.

Suggested follow-on names:

- `imui-editor-density-visual-gate-v1`
- `imui-editor-theme-preset-selector-v1`
- `imui-kit-chrome-tokenization-v1`
- `imui-style-editor-devtools-v1`

## Non-Goals

- Do not add `push_style_color(...)` / `pop_style_color(...)` or style-var stacks from this lane.
- Do not put editor density tokens in `crates/fret-ui`.
- Do not force `ImguiLikeDense` onto general-purpose Fret apps.
- Do not treat Dear ImGui's `ShowStyleEditor()` as a generic component backlog item.

## Gates

Suggested audit/gate commands:

```powershell
rg -n "EditorThemePreset|ImguiLikeDense|install_editor_theme_preset|reapply_installed_editor_theme_preset" ecosystem/fret-ui-editor/src/theme.rs apps/fret-cookbook/examples/imui_editor_controls_basics.rs apps/fret-examples/src/imui_editor_proof_demo.rs
rg -n "component\\.imui\\.disabled_alpha|imui_text_input_style_from_theme|input_text_model_uses_compact_imui_chrome_without_focus_ring|textarea_model_uses_compact_imui_chrome_without_focus_ring|hovered_like_imgui|ImUiHoveredFlags" ecosystem/fret-ui-kit/src/imui
rg -n "ShowStyleEditor|ImGuiStyle|PushStyleColor|PushStyleVar|StyleColorsDark|StyleColorsLight|StyleColorsClassic" repo-ref/imgui/imgui.h repo-ref/imgui/imgui_demo.cpp
cargo nextest run -p fret-ui-editor default_preset_keeps_existing_editor_patch_baseline imgui_like_dense_preset_overrides_density_and_field_chrome installed_preset_can_be_reapplied_after_base_theme_reset --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui input_text_model_uses_compact_imui_chrome_without_focus_ring textarea_model_uses_compact_imui_chrome_without_focus_ring --no-fail-fast
```

## Gate Results

2026-05-06 local results:

- `rg -n "EditorThemePreset|ImguiLikeDense|install_editor_theme_preset|reapply_installed_editor_theme_preset" ecosystem/fret-ui-editor/src/theme.rs apps/fret-cookbook/examples/imui_editor_controls_basics.rs apps/fret-examples/src/imui_editor_proof_demo.rs`
  passed and found the editor theme preset, cookbook opt-in, and editor proof default preset anchors.
- `rg -n "component\\.imui\\.disabled_alpha|imui_text_input_style_from_theme|input_text_model_uses_compact_imui_chrome_without_focus_ring|textarea_model_uses_compact_imui_chrome_without_focus_ring|hovered_like_imgui|ImUiHoveredFlags" ecosystem/fret-ui-kit/src/imui`
  passed and found the generic IMUI chrome, disabled alpha, hover flag, and focused text-control
  test anchors.
- `rg -n "ShowStyleEditor|ImGuiStyle|PushStyleColor|PushStyleVar|StyleColorsDark|StyleColorsLight|StyleColorsClassic" repo-ref/imgui/imgui.h repo-ref/imgui/imgui_demo.cpp`
  passed and found the Dear ImGui mutable style/editor reference anchors.
- `cargo nextest run -p fret-ui-editor default_preset_keeps_existing_editor_patch_baseline imgui_like_dense_preset_overrides_density_and_field_chrome installed_preset_can_be_reapplied_after_base_theme_reset --no-fail-fast`
  passed: 3 tests run, 3 passed.
- `cargo nextest run -p fret-ui-kit --features imui input_text_model_uses_compact_imui_chrome_without_focus_ring textarea_model_uses_compact_imui_chrome_without_focus_ring --no-fail-fast`
  passed: 2 tests run, 2 passed.

Rerun the gates above when editor density presets, generic IMUI chrome, or style-tooling follow-ons
change.
