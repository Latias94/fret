# Material 3 Visual Behavior Layout Parity v2 - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

The lane is open. M3PV2-010 is complete: the v2 parity-axis matrix exists and covers all 39
components from the closed Material3 component sweep. M3PV2-021 is complete: Material3 Select now
uses dotted `<base>.listbox` ids for the listbox automation surface. M3PV2-022 is complete:
Autocomplete fallback ids now use the same dotted listbox contract, ExposedDropdown proves
combobox/listbox wiring through composition, and live Material3 Select diagnostics have been swept
to the dotted ids. M3PV2-023 is complete: TextInput/TextArea gained labelled/described relation
targets, Material TextField wires visual label/supporting text into those relations, and the filled
chrome test now tracks the current container + active-indicator layer split.
M3PV2-024 is complete: the Material field text-start inset helper moved from Select into
`foundation::field`, TextField uses it for leading-icon input padding, floating label, and
supporting text, and fixed geometry gates now cover idle/focus/populated label positions.
M3PV2-025 is complete: Autocomplete and ExposedDropdown popup geometry now anchors to the
TextField chrome element when available, so icon-bearing fields get menu/listbox width parity with
the full field while the input remains the combobox trigger and keyboard/a11y owner.
M3PV2-026 is complete: Select selected menu items now use selected content colors for label,
leading icon, and trailing icon, and the visible item chrome uses the Material selectable-item inset
inside the listbox while the pressable row keeps the existing behavior contract.
M3PV2-027 is complete: Autocomplete and ExposedDropdown option rows now use the same shared
Material selectable menu item token outcomes as Select, including `4px` option chrome inset and
selected label color.
M3PV2-028 is complete: DatePicker calendar content now uses the Material 12px horizontal inset and
48px interactive weekday/date slots, with visual date chrome centered through the shared
`foundation::interactive_size` helper. Docked and modal automation gates prove weekday/date-cell
column alignment, and DatePicker headless goldens were refreshed for the intentional layout shift.
M3PV2-029 is complete: TimePicker display mode now keeps the period selector in the time display
row with the Material 12px margin, uses fixed 96px selector and 24px separator slots, centers the
clock dial in the picker chrome, and applies the same fixed separator/period-row structure to input
mode. TimePicker headless goldens were refreshed for the intentional layout shift.
M3PV2-031 is complete: full-screen SearchView now renders the header inside the Material 72px
header slot, exposes stable header-slot/divider/body part ids, and places the divider/content after
that slot. SearchView behavior stayed green and headless goldens were refreshed for the intentional
full-screen header/content shift.
M3PV2-032 is complete: ordinary SearchBar now applies Compose's 360..720px default width
constraint while SearchView-controlled headers remain full-width under SearchView overlay layout.
SearchBar/SearchView automation, SearchView behavior, and both headless golden suites stayed green.
M3PV2-033 is complete: SearchView now wires docked inputs to their overlay panel and full-screen
header inputs to their dialog through Fret's existing `controls` relation, while each overlay is
labelled by its controlling input. The gap was recipe wiring, not a core or kit mechanism issue.
M3PV2-034 is complete: multiline TextField now exposes Compose-aligned `min_lines`, `max_lines`,
and `line_limits` builders and maps visible line limits to Material chrome height. This packet also
closed a `fret-ui` TextArea mechanism gap by adding max-height support and measuring bound model
text during declarative layout. TextField headless goldens were refreshed for the intentional
active-indicator layer representation used by the current implementation.
M3PV2-035 is complete: TextField floating-label motion now initializes on the idle frame rather
than snapping on first focus. A shared TextField motion-frame helper drives single-line and
multiline branches, and fixed-frame tests now prove first-frame label movement plus active-indicator
thickness interpolation before settle.
M3PV2-036 is complete: Select trigger field motion now shares the Material field-motion foundation
with TextField. Initially populated Select labels mount at floated geometry, focused Select labels
animate through an intermediate first frame, and Select now exposes `<base>.label` for stable
automation. Select chevron and overlay alpha/scale motion remain separate residual probes.
M3PV2-037 is complete: Select chevron now uses Material `FastSpatial` spring motion and SceneOp
gates prove first-frame open/close chevron rotation plus Select overlay first-frame fade/scale.
Together with M3PV2-036, Select motion is now classified as v2-covered.
M3PV2-038 is complete: SearchView now uses dedicated Material search motion instead of generic
overlay scale. Docked overlays fade and vertically expand/shrink, full-screen overlays animate from
collapsed input geometry toward the viewport, and initially open SearchViews start settled.
M3PV2-039 is complete: ordinary standalone SearchBar indication motion is now v2-covered. The
packet found and fixed a recipe bug where ink was constrained to the padded content box and presses
starting over the editable text area did not start SearchBar ripple. SearchBar now keeps
component-local pointer-down interaction state, feeds it into the shared Material ink runtime, and
separates outer chrome from inner padded content.
M3PV2-041 is complete: Autocomplete and ExposedDropdown popup/trigger motion is now v2-covered.
Popup alpha/scale already used `foundation::overlay_motion`; the bug was Autocomplete's old
duration/easing chevron animator. The shared trigger now uses scoped Material `FastSpatial` spring
motion like Select, and ExposedDropdown inherits the fix through composition. Autocomplete headless
goldens were refreshed for the current active-indicator and selectable option row signatures.
M3PV2-042 is complete: DatePicker modal motion is now v2-covered for the current docked/modal
recipe surface. The packet extracted `foundation::modal_motion`, kept Dialog on the same modal
fade/rise/scale transform, and moved DatePickerDialog off its old pure-scale panel animation.
M3PV2-043 is complete: TimePickerDialog modal motion now uses the same shared modal helper for
both initial dial and input modes. That packet left the clock-face selector movement and
hour/minute crossfade as a separate motion gap.

M3PV2-044 is complete: TimePicker clock-face motion now models Compose's analog dial behavior with
an angle spring, a face-alpha crossfade, and a separate selector chrome layer. The motion axis is
now v2-covered, and the headless TimePicker suite was refreshed for the intentional selector chrome
split.

M3PV2-045 is complete: TimePicker 24h hour mode now renders Compose-aligned outer `00..11` and
inner `12..23` rings, uses the Compose ring split for pointer selection, and keeps selector radius
in the spatial motion runtime. TimePicker layout remains v2-covered with the 24h ring gap closed.

M3PV2-046 is complete: BottomSheet modal motion and semantics are now v2-covered for the current
recipe surface. The packet found a recipe gap where the sheet slid by viewport height, faded the
panel, and exposed the modal surface as `Group`; ModalBottomSheet now uses Material
`DefaultSpatial` / `DefaultEffects` motion, translates by the sheet surface's own height, keeps
the panel opaque, and exposes dialog/scrim/drag-handle semantics.

M3PV2-047 is complete: standalone Button style/layout/accessibility/motion axes are now v2-covered.
The packet found a Material recipe/token wiring gap, not a core or kit mechanism gap: Button had
state-layer/ripple support but did not feed stateful elevation into the existing Material
surface/elevation foundation. Button now paints default Elevated shadows, animates Filled/Tonal
hover elevation, snaps disabled elevation, exposes role/label/disabled semantics in a focused gate,
and uses Compose-aligned `DefaultEffects` for pressed-shape motion.

M3PV2-048 is complete: standalone Badge style/layout/accessibility axes are now v2-covered. The
packet found a Material recipe gap, not a core or kit mechanism gap: Badge collapsed BadgedBox root,
anchor, and visual badge identity into one semantics wrapper, and that wrapper masked text-badge
intrinsic width. Badge now exposes `base`, `base.anchor`, and `base.badge`, puts author label/value
semantics on the badge part, and refreshed Badge goldens for Material-aligned text badge expansion.

M3PV2-049 is complete: standalone Card style/accessibility/motion axes are now v2-covered while
layout remains caller-owned. The packet found a Card recipe gap and a Material foundation gap:
static cards leaked disabled button semantics through the reused pressable wrapper, and Button's
Compose-like elevation animation had not yet been promoted to shared foundation. Static cards now
present as non-disabled groups with no invoke action, interactive cards keep button semantics, and
Button/Card share `foundation::elevation` animation behavior.

## Decisions

- This lane is about shadcn-level proof density, not shadcn visual styling.
- Material spec, Compose Material3, MUI Material UI, and Base UI are axis-specific references.
- Stable Fret-side shadcn components are exemplars for layering and gates only.
- Layout defaults must be classified as intrinsic recipe defaults or caller-owned before edits.
- Shared foundation refactors require multiple component proofs.

## Next Recommended Action

Continue with the next uncovered Material3 packet from the matrix. Good M5 candidates are
CarouselItem, FAB, List, and ProgressIndicator; higher-priority choice-control candidates remain
Checkbox and Radio.

## Useful Gates

```powershell
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test select_behavior
```
