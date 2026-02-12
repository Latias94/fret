# Shadcn Style Overrides & `*Style` Patterns (v1)

This document defines a shared, reusable pattern for exposing per-component style overrides in
shadcn/ui-aligned Fret component libraries.

Primary references:

- State-driven resolution primitive: `docs/adr/0219-state-driven-style-resolution-v1.md`
- Token naming vocabulary: `docs/shadcn-style-token-conventions.md`
- Focus-visible semantics: `docs/adr/0061-focus-rings-and-focus-visible.md`

## Goals

- Provide a consistent user-facing override API shape across interactive controls.
- Keep “mechanism” (`WidgetStates` / `WidgetStateProperty<T>`) in `fret-ui-kit`, and keep “policy”
  (defaults/variants) in component crates (`fret-ui-shadcn`, future Material 3, etc.).
- Avoid expensive or surprising deep-merge behavior.

Non-goals:

- A complete design system specification.
- A CSS-like selector language.

## Recommended `*Style` Shape

Expose a per-control `*Style` struct that:

- uses `Option<...>` for each overrideable slot,
- is cheap to clone and merge,
- is stable enough to become a public API surface for ecosystem crates.

### 1) Per-state values (interactive chrome)

For interactive controls, prefer:

- `Option<WidgetStateProperty<Option<T>>>`
- Prefer using the alias `fret_ui_kit::OverrideSlot<T>` in Rust code for readability.

The inner `Option<T>` enables **partial overrides**:

- `Some(T)`: override the slot for the current state
- `None`: do not override; fall back to the widget's default style for this state

Component code should apply this at resolve-time (resolve override → `Option<T>`; if `None`, use
the default property's resolved value).

### 2) Optional surfaces (background may be absent)

If the final widget output is “optional” (e.g. background can be absent), still prefer the same
shape and treat “no background” as a policy choice:

- use theme tokens and fallbacks for defaults,
- use transparent colors when you need an explicit “clear” outcome (avoid `Option<Option<T>>`).

### 3) Non-stateful values

For values that do not vary by widget state, prefer:

- `Option<T>`

Examples: fixed metrics, text style overrides, constant colors, feature flags.

## Merge Semantics (`merged`)

Every `*Style` should provide:

- `fn merged(self, other: Self) -> Self`

Rules:

- Right-biased: fields in `other` override fields in `self` when `other.<field>.is_some()`.
- No deep merge: a `WidgetStateProperty<T>` is treated as an atomic value (replace as a whole).

Rationale:

- Predictable precedence (no “partial” merging surprises).
- Cheap merges (no per-field allocation or per-state reconciliation).

## Component Integration (`.style(...)`)

Controls should expose a builder method:

- `fn style(mut self, style: *Style) -> Self`

Implementation pattern:

- store a `style: *Style` field on the component,
- merge on every call: `self.style = self.style.merged(style)`,
- apply overrides by merging into a variant/recipe-derived default:
  `let style = default_style.merged(style_override);`

This makes multiple `.style(...)` calls composable and keeps “default vs override” explicit.

## Computing `WidgetStates` (don’t duplicate logic)

Compute `WidgetStates` once per widget and reuse it across slots.

Recommended baseline:

- `WidgetStates::from_pressable(cx, pressable_state, enabled)` (includes focus-visible policy)
- then set additional semantic bits as needed:
  - `selected` (toggles, tabs, current rows)
  - `open` (menus, submenus, expanded disclosure)

## Examples (existing v1 surfaces)

- `ButtonStyle`: `ecosystem/fret-ui-shadcn/src/button.rs`
- `CheckboxStyle`: `ecosystem/fret-ui-shadcn/src/checkbox.rs`
- `RadioGroupStyle`: `ecosystem/fret-ui-shadcn/src/radio_group.rs`
- `SelectStyle`: `ecosystem/fret-ui-shadcn/src/select.rs`
- `SliderStyle`: `ecosystem/fret-ui-shadcn/src/slider.rs`
- `SwitchStyle`: `ecosystem/fret-ui-shadcn/src/switch.rs`
- `ToggleStyle`: `ecosystem/fret-ui-shadcn/src/toggle.rs`
- `ToggleGroupStyle`: `ecosystem/fret-ui-shadcn/src/toggle_group.rs`
- `TabsStyle`: `ecosystem/fret-ui-shadcn/src/tabs.rs`
- `InputStyle` (focus-specific v0 shape): `ecosystem/fret-ui-shadcn/src/input.rs`

## Notes / Pitfalls

- Gate ring/border accents on `focus_visible` (not `focused`) to match ADR 0061.
- In `WidgetStateProperty<T>`, “last matching override wins”; place more specific overrides later.
- Keep the number of style slots small. When in doubt, prefer theme tokens + documented fallbacks
  over adding per-component fields.
