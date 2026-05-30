# Material 3 Test ID Contract v1

Status: Active
Updated: 2026-05-27
Scope: `ecosystem/fret-ui-material3` recipe-level automation surfaces.

## Purpose

Material 3 recipes must expose intent-level automation anchors so parity packets, diagnostics
scripts, and headless tests can target stable parts without depending on layout position or child
index. These selectors are recipe policy, not `crates/fret-ui` mechanism policy.

This contract covers the first gated Material packets: `Select` and `Switch`.

## Naming Rules

| Surface | Rule | Owner |
| --- | --- | --- |
| Component root | Caller-provided base ID via `.test_id("<base>")` | Call site |
| Visual chrome | `<base>.chrome` | Recipe |
| Popup/listbox | `<base>-listbox` | Recipe |
| Option/item root | Caller-provided option ID via `SelectItem::test_id("<item>")` | Call site |
| Option/item chrome | `<item>.chrome` | Recipe |
| Select trigger trailing affordance | `<base>.trailing-icon` | Recipe |
| Select active indicator | `<base>.active-indicator` | Recipe |
| Select option leading affordance | `<item>.leading-icon` | Recipe |
| Select option trailing affordance | `<item>.trailing-icon` | Recipe |
| Switch track | `<base>.track` | Recipe |
| Switch handle | `<base>.handle` | Recipe |
| Switch selected icon | `<base>.icon-on` | Recipe |
| Switch unselected icon | `<base>.icon-off` | Recipe |

Selectors use semantic part names, not ordinal names such as `child-0` or `item-2`. Indexed IDs are
only acceptable when the caller's data model is naturally indexed and the index is part of the
caller-owned item identity.

## Select Contract

`Select::test_id("m3-select")` must expose:

- `m3-select`: trigger/root semantics, role, focus, activation, and `expanded` state.
- `m3-select.chrome`: visual field chrome used by diagnostics and fill/outline assertions.
- `m3-select.trailing-icon`: trigger dropdown affordance.
- `m3-select.active-indicator`: filled-field active indicator.
- `m3-select-listbox`: popup listbox derived from the trigger base ID.

`SelectItem::test_id("m3-select-item-alpha")` must expose:

- `m3-select-item-alpha`: option root semantics, selection, disabled state, position metadata, and
  focus.
- `m3-select-item-alpha.chrome`: option chrome used by item fill/height assertions.
- `m3-select-item-alpha.leading-icon`: optional leading icon.
- `m3-select-item-alpha.trailing-icon`: optional trailing icon.

Fallback behavior:

- If a Select has no base ID, the listbox keeps the existing `material3-select-listbox` fallback.
- Derived chrome/part selectors are only created when the caller provides the corresponding base
  root/item ID.

Evidence:

- `ecosystem/fret-ui-material3/src/select.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/select_behavior.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-a11y-parity-bundle.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-item-chrome-fill.json`

## Switch Contract

`Switch::test_id("m3-switch")` must expose:

- `m3-switch`: switch root semantics, checked state, disabled state, focus, and activation.
- `m3-switch.chrome`: visual chrome centered inside the minimum interactive target.
- `m3-switch.track`: token-driven track surface.
- `m3-switch.handle`: token-driven handle/thumb surface.
- `m3-switch.icon-on`: selected icon layer when icon rendering is enabled.
- `m3-switch.icon-off`: unselected icon layer when two-icon rendering is enabled.

Fallback behavior:

- Derived part selectors are only created when the caller provides a Switch base ID.
- `m3-switch.icon-off` is not required for selected-only-icon mode because that mode intentionally
  omits the unselected icon.

Evidence:

- `ecosystem/fret-ui-material3/src/switch.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-switch-handle-screenshots.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-switch-icon-motion-timeline-screenshots.json`

## Gate

```powershell
cargo test -p fret-ui-material3 --test automation_surface --no-run
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
```

The first command proves the default test target remains compile-safe without diagnostics enabled.
The second command proves the live diagnostics frame can find the root, chrome, part, popup/listbox,
option/item, and secondary-affordance IDs.

## Follow-Up Rule

Any new Material component packet must record its root, chrome, content/indicator, popup/listbox,
item, and secondary-affordance selector names here or in a component-specific follow-on artifact
before adding new diagnostics predicates.
