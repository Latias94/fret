# App builder engineering notes

Use this note when the visual direction is already chosen and the remaining work is about making the UI robust, scriptable, and architecture-aligned.

## 1) Theme and tokens

- Prefer theme-first: one preset + small overrides
- Avoid per-component magic numbers
- Generator helper:
  - `python3 .agents/skills/fret-app-ui-builder/scripts/stylegen.py --suggest "<keywords>"`
- High-leverage knobs:
  - `theme/token-groups.md`
  - `theme/editor-presets.md`
  - `mind-models/mm-theme-and-tokens.md`

## 2) Layout and overflow

- Use token-driven layout/chrome via `UiBuilder` (declarative-only)
- When porting from shadcn/Tailwind, align layout constraints first:
  - `mind-models/mm-layout-and-sizing.md`
- Common mappings:
  - `w-full` / `h-full` → `.ui().w_full()` / `.ui().h_full()`
  - `flex-1` → `.ui().flex_1()`
  - pair text-heavy rows with `.ui().min_w_0()`
  - `items-stretch` → explicit flex container stretch
  - `truncate` / `overflow-hidden` → `.ui().truncate()` / `.ui().overflow_hidden()`
- Don’t clip focus rings by accident: keep the pressable/root overflow visible; clip only inside chrome

## 3) Interaction policy and overlays

- `crates/fret-ui` is mechanism-only; policy belongs in components via action hooks
- Pressable activate/toggle: prefer `fret-ui-kit` helpers such as `cx.pressable_toggle_bool(&open)`
- Dismiss policy for overlays should stay component-owned
- Pick the correct family: menu vs popover vs modal
- Menus are usually non-click-through on outside press
- Focus restores to the trigger on close unless explicitly overridden

See also:

- `mind-models/mm-layering.md`
- `mind-models/mm-overlays-and-focus.md`

## 4) Commands, keymaps, and text input

- Treat `CommandId` as stable contracts
- Always add explicit `when` gating for global shortcuts
- Keep text channels separate: `KeyDown` vs `TextInput` vs `ImeEvent`
- While composing, IME gets first refusal on Tab/Escape/arrows
- Provide caret rect feedback for candidate window placement when needed

See also:

- `mind-models/mm-models-actions-and-commands.md`

## 5) Virtualization and scheduling

- Use keyed virtualization; keys must come from the model, never row index
- Prefer fixed row heights when possible in editor UIs
- Tie continuous frame leases to element lifetime
- Prefer runner-owned timers/effects so behavior stays deterministic and diagnosable

## 6) Regression gates and automation surfaces

- Add stable `test_id` to interactive affordances early
- Leave one diag script per interaction state machine when behavior is non-trivial
- Use a style-agnostic polish pass one surface at a time:
  - `polish/polish-pass.md`
- Diagnostics and gate heuristics:
  - `mind-models/mm-diagnostics-and-regression-gates.md`
- A11y and `test_id` guidance:
  - `mind-models/mm-a11y-and-testid.md`
