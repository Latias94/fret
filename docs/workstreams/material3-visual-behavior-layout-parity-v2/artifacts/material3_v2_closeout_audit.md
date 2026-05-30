# Material3 v2 Component-Axis Closeout Audit

Date: 2026-05-29
Tasks: M3PV2-020, M3PV2-030, M3PV2-040, M3PV2-050, M3PV2-060, M3PV2-085, M3PV2-080

## Target State

- The v2 matrix covers all 39 Material3 components from the v1 sweep.
- No component axis is left as `seeded` or `needs_v2_packet`.
- Every completed packet records source truth, layer ownership, artifacts, proof gates, and
  residual risk.
- Remaining work is future API breadth, exhaustive style matrices, or test-maintenance scope rather
  than an unclosed current-surface parity gap.

## Family Closure

- Field family: M3PV2-021 through M3PV2-045 closed listbox id continuity, field relations, floating
  label geometry, popup width/option styling, picker layout, SearchBar/SearchView accessibility and
  motion, multiline line limits, and fixed-frame trigger/overlay motion.
- Navigation/app chrome: M3PV2-073, M3PV2-074, M3PV2-075, M3PV2-076, and M3PV2-084 closed Tabs,
  NavigationBar, NavigationRail, NavigationDrawer, ModalNavigationDrawer, and TopAppBar current
  layout/accessibility/motion axes.
- Choice controls and chips: M3PV2-065 through M3PV2-071 closed Checkbox, Radio, Switch, Slider,
  SegmentedButton, IconButton, Chip, and ChipSet geometry, explicit checked-state semantics, stable
  parts, and current state-layer/selection motion gates.
- Overlay and feedback: M3PV2-046, M3PV2-077, M3PV2-078, M3PV2-079, M3PV2-081, and M3PV2-083
  closed BottomSheet, Dialog, Snackbar, Tooltip, Menu, DropdownMenu, and standalone Menu item
  current-surface layout/accessibility/motion axes.
- Surface and data display: M3PV2-047, M3PV2-048, M3PV2-049, M3PV2-061, M3PV2-062, M3PV2-063,
  M3PV2-064, and M3PV2-082 closed Button, Badge, Card, CarouselItem, FAB, List, and
  ProgressIndicator proof density. Divider remains low-risk v1 coverage in the matrix.

## Layer Findings

- Core mechanism gaps were real but narrow: text control labelled/described relations, TextArea
  max-height, and state-description semantics were added only where the Material packets proved a
  portable contract need.
- Kit policy gaps were similarly bounded: Switch checked-state helper output and Snackbar toast
  surface scale/min-height channels were shared behavior, not Material-only chrome.
- Material foundation extraction was justified where multiple consumers proved the need: field
  insets, modal motion, elevation animation, active-indicator painting, interactive sizing, and
  state-layer/ripple indication.
- Most remaining fixes stayed in `ecosystem/fret-ui-material3` recipes and tests, preserving the
  mechanism-vs-policy boundary.

## Harness Audit

The original harness-consolidation task id duplicated the IconButton packet id (`M3PV2-070`), so
the closeout renames the harness audit to `M3PV2-085`.

No broad test deletion was made in this closeout:

- `radio_alignment` remains referenced by closed packet evidence for headless goldens and behavior
  gates.
- headless golden fixtures remain historical proof for intentional scene signatures.
- helper duplication can be reduced later, but only with replacement gates in a dedicated
  test-maintenance workstream.

## Follow-On Boundaries

Start new workstreams for these items instead of reopening this lane:

- adaptive NavigationSuite, wide rails, dismissible drawer gestures, RTL drawer slide, and
  predictive-back drawer/dialog/sheet choreography,
- richer Menu item families, submenu triggers, shortcut text, and scroll/max-height behavior,
- rich tooltip action rows or a hit-testable rich-tooltip/popover API,
- DatePicker year/input-mode breadth and TimePicker advanced crossfades,
- full carousel container behavior, keyline sizing, masking, parallax, and carousel semantics,
- exhaustive visual matrices for style variants, themes, expressive sizes, and exact token
  conflicts between Compose and Material Web aliases,
- test helper consolidation and fixture-driven reductions for broad headless/golden suites.

## Proof

Closeout gates:

```powershell
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test select_behavior
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
git diff --check
```

The matrix query used for closeout returned no `seeded` or `needs_v2_packet` rows.
