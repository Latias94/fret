# Control Chrome Normalization Audit v1 - TODO

Status: Active

Workstream overview:

- `docs/workstreams/control-chrome-normalization-audit-v1.md`
- `docs/workstreams/control-chrome-normalization-audit-v1-milestones.md`

---

## TODO (prioritized)

### Contract + tests (M0)

- [ ] Expand unit tests for `normalize_control_chrome_sizing`:
  - [ ] `pressable.layout.size.width = Fill` -> chrome width Fill
  - [ ] `pressable.layout.size.height = Fill` -> chrome height Fill
  - [ ] `pressable.layout.size.width = Px(_)` -> chrome width Fill (border-box semantics)
  - [ ] `pressable.layout.size.height = Px(_)` -> chrome height Fill (border-box semantics)
  - [ ] `min/max` shrink-by `(padding + border)` is correct for both axes
  - [ ] icon-button square sizing: outer box square -> chrome Fill + content centered

### Audit + migration (M1/M2)

- [ ] Populate the audit table below (start with high-impact + likely-to-stretch components).
- [ ] For every “At risk” row:
  - [ ] Decide migration strategy: adopt `control_chrome_*` vs explicit normalization helper.
  - [ ] Add at least one evidence anchor: unit test, diag script, or focused integration test.

### Diagnostics gates (M3)

- [ ] Add a `fretboard diag` scenario that exercises:
  - Tabs triggers with `flex-1`
  - ButtonGroup/ToggleGroup with stretched items
  - Dialog trigger in a stretched row

---

## Audit table

Legend:

- **Pattern**
  - `ControlChrome`: uses `control_chrome_*` helper
  - `ManualFill`: child chrome explicitly sets `w/h = Fill`
  - `AdHocChrome`: pressable composes a “chrome” child but does not enforce Fill invariants
- **Risk**
  - `OK`: safe by construction
  - `At risk`: outer can stretch but inner chrome can remain shrink-wrapped
  - `Unknown`: needs inspection

| Area | Component / Element | Pattern | Outer can stretch? | Chrome fills? | Risk | Migration target | Evidence | Notes |
|---|---|---|---|---|---|---|---|---|
| `ecosystem/fret-ui-kit/src/declarative/chrome.rs` | `control_chrome_pressable_with_id_props` | `ControlChrome` | Yes | Yes | OK | N/A | Unit tests in-file | Canonical helper; expand matrix tests. |
| `ecosystem/fret-ui-shadcn/src/button.rs` | shadcn Button | `ControlChrome` | Yes | Yes | OK | N/A | Uses helper | Prefer keeping all shadcn controls on this path. |
| `ecosystem/fret-ui-shadcn/src/tabs.rs` | Tabs triggers | `ManualFill` | Yes | Yes | OK | Optional: adopt helper | N/A | Inner container sets `w/h = Fill`. |
| `ecosystem/fret-ui-shadcn/src/menubar.rs` | Menubar trigger | `ControlChrome` | Possible (caller-dependent) | Yes | OK | N/A | Uses helper | Migrated to `control_chrome_pressable_with_id_props`. |
| `ecosystem/fret-ui-shadcn/src/menubar.rs` | Menubar items (overlay) | `ManualFill` | Yes (`w = Fill`) | Yes (`w = Fill`) | OK | Optional: adopt helper | N/A | `menu_row_children` chrome container sets `w = Fill`. |
| `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` | Dropdown menu items | `ManualFill` | Yes (`w = Fill`) | Yes (`w = Fill`) | OK | Optional: adopt helper | N/A | Item row background container sets `w = Fill`. |
| `ecosystem/fret-ui-shadcn/src/context_menu.rs` | Context menu items | `ManualFill` | Yes (`w = Fill`) | Yes (`w = Fill`) | OK | Optional: adopt helper | N/A | `menu_row_children` chrome container sets `w = Fill`. |
| `ecosystem/fret-ui-material3/src/segmented_button.rs` | Segmented button segments | `ManualFill` | Yes | Yes | OK | Optional: adopt helper | N/A | `material_segment_chrome` sets `w = Fill`. |
| `ecosystem/fret-ui-material3/src/button.rs` | Material button | `ControlChrome` | Future (if layout becomes patchable) | Yes | OK | N/A | Uses helper | Migrated to `control_chrome_pressable_with_id_props`. |
| `ecosystem/fret-ui-material3/src/card.rs` | Material card | `ControlChrome` | Future (if layout becomes patchable) | Yes | OK | N/A | Uses helper | Migrated to `control_chrome_pressable_with_id_props`. |
| `ecosystem/fret/src/workspace_menu.rs` | Menubar trigger | `ControlChrome` | Possible (caller-dependent) | Yes | OK | N/A | Uses helper | Migrated to `control_chrome_pressable_with_id_props`. |
| `ecosystem/fret/src/workspace_menu.rs` | Menubar item | `ControlChrome` | Yes (`w = Fill`) | Yes | OK | N/A | Uses helper | Migrated to `control_chrome_pressable_with_id_props`. |
| `ecosystem/fret-code-view/src/copy_button.rs` | Copy button | `AdHocChrome` | Possible (caller-dependent) | No | At risk | Adopt helper | N/A | Chrome container does not set `w/h = Fill`; safe today but fragile in flex/grid. |

Add rows as audit progresses. The key question for each row is:

1) Can the outer pressable receive a definite box from layout (now or in future composition)?
2) If yes, does the chrome child fill both axes?
