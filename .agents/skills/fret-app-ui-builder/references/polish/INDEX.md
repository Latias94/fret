# Polish pass (style-agnostic)

This folder is a **style-agnostic “polish pass”** for making Fret UIs look coherent and feel good to use.

It intentionally avoids “pick one of 67 styles” and instead focuses on universal, high-leverage moves:

- visual hierarchy (type + contrast)
- spacing rhythm (layout + density)
- consistent states (hover/pressed/disabled/error/focus-visible)
- empty/loading/error states
- overlays (menus/dialogs/popovers) that behave predictably
- small, purposeful motion

## How to use

Use this late in `fret-app-ui-builder` (after the shell works) or as the rule source for `fret-ui-review`.

Recommended workflow:

1. Choose one surface (settings form, data table, command palette, workspace shell).
2. Pick a baseline theme + density (`new-york-v4` + small token overrides).
3. Apply the checklist in `polish-pass.md` and stop when the UI is cohesive.
4. Add stable `test_id` anchors and at least one diag script for the most fragile interaction.

## Files

- `polish-pass.md`: the checklist + rules (`rule_id`) to apply to one screen at a time.

## Notes

- App authors should not worry about upstream parity (Radix/shadcn/Base UI). If a component behavior seems wrong,
  leave a minimal diag repro and hand it to the framework/eco owners.

