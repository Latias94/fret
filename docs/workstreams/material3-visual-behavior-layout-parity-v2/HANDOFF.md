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

M3PV2-061 is complete: standalone CarouselItem style/accessibility/motion axes are now v2-covered
while layout is classified as caller-owned. The packet found a Material recipe gap: static carousel
items leaked default button semantics through `Pressable`, and interactive items snapped hover
elevation instead of animating through the shared Material elevation runtime. Static items now
present as non-disabled groups with no invoke action, explicit width/height is proven against root
and `.chrome` bounds, and interactive items share `foundation::elevation` animation behavior with
Button/Card. Full carousel container scrolling, keyline sizing, masking/parallax, and
`Role.Carousel` semantics remain outside this standalone item recipe.

M3PV2-062 is complete: standalone FAB style/layout/accessibility/motion axes are now v2-covered
for the current recipe surface. The packet found Material recipe/token wiring gaps: FAB chrome was
using min-size layout and could stretch to the touch target, default 56px FABs were conflated with
80px medium FABs, extended FAB ignored `size`, primary lowered elevation missed Material Web alias
tokens, and hover elevation snapped instead of animating. FAB now separates touch target from
visual chrome, has explicit default `Regular` and Material `Medium` sizes, applies size-specific
extended FAB tokens, resolves lowered elevation aliases, and shares `foundation::elevation` with
Button/Card/CarouselItem. Show/hide and extended collapsed/expanded choreography remain residual
because the current Fret FAB API has no visibility/expanded state surface.

M3PV2-063 is complete: standalone List style/layout/accessibility axes are now v2-covered for the
current selectable recipe surface. The packet found a Material recipe/API completeness gap, not a
core or kit mechanism gap: ListItem exposed only a one-line headline + icon MVP while Compose
Material3 ListItem supports headline, overline, supporting, leading, and trailing slots with
56/72/88px density outcomes. ListItem now exposes overline/supporting/trailing-supporting text
builders, resolves one-line/two-line/three-line heights, exposes stable slot part ids, refreshes
List headless goldens for multi-line rows, and proves List/ListItem role, selected state, disabled
state, and collection metadata. Roving keyboard behavior remains seeded rather than v2-closed, and
reorder/reveal, avatars/images/video, segmented list items, and multiline wrapped supporting text
remain residual.

M3PV2-064 is complete: ProgressIndicator accessibility and current indeterminate draw-region
motion axes are now v2-covered. The packet found a Material recipe/diagnostics wiring gap, not a
core or kit mechanism gap: `fret-core` already exposed `ProgressBar`, numeric range metadata, and
busy flags, but Material progress indicators wrapped their canvas as generic semantics nodes.
Linear and circular indicators now expose progressbar semantics, optional accessible labels,
determinate 0..1 numeric metadata, indeterminate busy state, circular `.track` / `.active-track`
part ids, and a fixed-frame test proves indeterminate draw regions move across frames. Determinate
value interpolation, linear default width policy, and wavy progress indicators remain residual.

M3PV2-065 is complete: Checkbox layout, accessibility, and current checked-mark motion are now
v2-covered. The packet found a Material recipe wiring gap, not a core mechanism gap:
`SemanticsCheckedState::Mixed` and the kit checkbox a11y helper already existed, but Material
Checkbox bypassed them and only wrote legacy binary `checked`. Checkbox now writes binary and
tri-state checked metadata, exposes `.chrome`, `.box`, and `.mark` part ids, proves 48/40/18px
touch/state-layer/box/mark geometry, and animates mark visibility with the Material
`DefaultSpatial` motion scheme. Exact Compose path-draw geometry and a public error-state variant
remain residual.

M3PV2-066 is complete: Radio layout, accessibility, and current selected-dot motion are now
v2-covered. The packet found a Material recipe wiring/proof-density gap, not a core or kit
mechanism gap: `SemanticsCheckedState::{True,False}` and the kit radio a11y helper already existed,
and Radio already had roving/typeahead behavior, collection metadata, 48/40/20/10px geometry, and
Material ripple/state-layer wiring. Radio now writes explicit checked-state metadata, exposes
`.chrome`, `.icon`, and `.dot` part ids, starts initially selected radios settled, and animates
selected-dot growth with the Material `FastSpatial` motion scheme. Color interpolation and
form-registration parity remain residual.

M3PV2-067 is complete: Switch layout, accessibility, and current handle motion are now v2-covered.
The packet found a shared kit primitive gap plus Material recipe wiring/proof-density gap, not a
core mechanism gap: `SemanticsCheckedState::{True,False}` already existed, but
`fret-ui-kit::primitives::switch::switch_a11y` only wrote legacy binary `checked`, and Material
Switch bypassed that helper. Switch now writes explicit checked-state metadata through the kit
helper, automation asserts it, and the new focused test proves 52/48/40/32/24px geometry plus
fixed-frame handle movement. Existing pointer/keyboard/icon ripple and headless controls gates
stayed green. Drag/swipe gestures and exact Compose `FastSpatial` replacement for the current
Material Web-aligned motion remain residual.

M3PV2-068 is complete: Slider layout, accessibility, and current state-layer/draw-region motion
are now v2-covered. The packet found a Material recipe accessibility/proof-density gap, not a core
or kit mechanism gap: core already exposes numeric metadata and derives set-value support when
the numeric contract is complete, while Material Slider already had pointer/keyboard/RTL/range
behavior and state-layer animation. Slider now publishes keyboard-aligned continuous step/jump
metadata, RangeSlider thumbs publish peer-constrained min/max semantics, focused tests prove
active-track draw-region updates, and fixed-frame scene evidence proves pressed state-layer
animation. Vertical slider, dedicated value-indicator choreography, and advanced track
gap/corner-shrinking parity remain residual.

M3PV2-069 is complete: SegmentedButton layout, accessibility, and current state-layer motion are
now v2-covered. The packet found a Material recipe wiring/proof-density gap, not a core or kit
mechanism gap: explicit checked-state semantics already existed and SegmentedButton already had
roving focus, RTL-aware horizontal arrows, 48/40px touch/chrome split, joined borders, ripple, and
state-layer animation. SegmentedButton now writes explicit binary checked-state metadata, exposes
`.chrome`, `.icon`, and `.label` part ids, proves joined 48px touch targets with centered 40px
chrome, and proves pressed state-layer animation. Exact Compose check-icon scale/content-offset
motion and selected-interaction z-index draw-order assertions remain residual.

M3PV2-070 is complete: IconButton layout, accessibility, and current state-layer motion are now
v2-covered. The packet found a Material recipe wiring/proof-density gap, not a core or kit
mechanism gap: explicit checked-state semantics already existed and IconButton already had 48px
minimum touch targets, 40px chrome, 24px icon tokens, shape spring motion, ripple, and state-layer
wiring. IconButton now writes explicit binary checked-state metadata for toggle surfaces, exposes
`.chrome` and `.icon` part ids, proves 48/40/24px geometry, and proves pressed state-layer
animation. Larger expressive icon-button sizes and dedicated corner-radius timeline assertions
remain residual.

M3PV2-071 is complete: Chip, SuggestionChip, FilterChip, InputChip, and ChipSet layout, behavior,
accessibility, and current state-layer motion are now v2-covered. The packet found a Material
recipe wiring/proof-density gap, not a core or kit mechanism gap: explicit checked-state semantics
already existed and chips already had Material 48px interactive sizing, 32px chrome, 18px icons,
ripple, and state-layer wiring. Assist/Suggestion chips now expose stable `.label` and
`.leading-icon` content anchors, Filter/Input chips expose `Checkbox` semantics plus explicit
checked state, actionable trailing icons split the 34x48 action slot from the 18px glyph, and
focused tests prove primary/trailing activation routing, ChipSet 8px gap/wrap layout, and pressed
state-layer animation. Exact selectable icon expand/shrink/fade, InputChip avatar slots, and
expressive corner morphing remain residual.

M3PV2-072 is complete: standalone SearchBar accessibility is now v2-covered. The packet found both
a core mechanism gap and a Material recipe gap. Core semantics had role descriptions but no
state-description field, so Compose's expanded suggestions phrase could not be represented without
overloading label/value. Material SearchBar also made its accessible label entirely caller-owned,
while Compose always publishes a default search label. `SemanticsNodeExtra::state_description` now
flows through `fret-ui` declarative authoring surfaces and AccessKit; Material SearchBar now uses
localized default strings for `Search` and expanded `Suggestions below` while preserving explicit
label overrides, placeholder semantics, and SearchView relation wiring. Diagnostics JSON export of
state descriptions remains additive future work.

M3PV2-073 is complete: text-only primary Tabs layout and accessibility axes are now v2-covered.
The packet found a Material recipe gap plus a shared Material foundation gap, not a core or kit
mechanism gap. Core already had tab roles, orientation, selected state, collection metadata, and
relations; kit already had richer headless Tabs helpers. Material Tabs now writes horizontal
`TabList` orientation, exposes per-tab `.label` anchors, uses Compose-aligned content-sized primary
indicator geometry with the 24px minimum, and applies the 52px edge padding / 90px minimum tab
width for scrollable rows. The shared active-indicator canvas now explicitly fills its parent so a
live indicator test id also produces a painted indicator quad. Leading-icon tabs, secondary tabs,
panel-owning Material Tabs, and scroll-to-selected overflow behavior remain residual API/follow-on
work.

M3PV2-074 is complete: NavigationBar and NavigationRail layout/accessibility are now v2-covered
for the current collapsed destination recipes. Compose Material3 was the primary source for this
packet: NavigationBar uses an 80dp container, 8dp destination gap, 64x32dp active indicator, and
12dp top indicator offset; NavigationRail uses an 80dp collapsed width, 56dp item height, and 4dp
vertical rail/item spacing. Fret core already had `TabList` / `Tab` orientation, selected,
disabled, and collection metadata mechanisms, and the existing Material roving behavior remained
valid. The packet found Material recipe/token-accessor gaps: both roots omitted orientation,
NavigationBar had no item gap and centered the destination stack instead of the top-icon lane,
NavigationRail applied 4dp padding on all sides and let item chrome collapse to the interactive
minimum, and active-indicator target fallback used the wrong bounds. Adaptive NavigationSuite, wide
rails, modal rails, headers, and dedicated motion diagnostics remain residual/future API work.

M3PV2-075 is complete: NavigationDrawer layout/accessibility and ModalNavigationDrawer
layout/accessibility/current motion are now v2-covered for the current drawer recipes. Compose
Material3 was the primary source: Drawer items use `Role.Tab`, 56dp height, 12dp external item
padding, 16/24dp content padding, and 12dp icon/badge gaps; ModalNavigationDrawer uses
`NavigationMenu` pane semantics, `CloseDrawer` scrim description, a 360dp sheet, full-height panel,
and a negative sheet-width closed anchor. Fret core already had orientation, tab, dialog, label, and
layout-transparent semantics mechanisms, and kit overlay focus/dismiss policy remained green. The
packet found Material recipe/proof-density gaps: NavigationDrawer omitted vertical orientation,
ModalNavigationDrawer panel exposed only generic test-id semantics, and the scrim had no close
label. Dismissible drawer gestures, predictive-back scaling, RTL slide direction, permanent drawer
insets, headers, and adaptive NavigationSuite remain residual/future API work.

M3PV2-076 is complete: TopAppBar layout and current scroll/collapse motion are now v2-covered for
the current recipe surface. Compose Material3 was the primary source: app bars use 4dp horizontal
padding, a 12dp title inset under the 16dp content start, Medium/Large collapse to 64dp from
112/152dp, the two-row top title uses `TopTitleAlphaEasing`, and container color transitions with
a FastOutLinearIn-style scroll fraction. The packet found Material recipe/token proof-density
gaps, not core or kit mechanism gaps: TopAppBar already had scroll behavior state and toolbar
semantics, but lacked stable chrome/title probes, used boolean container color switching, and used
linear collapsed-title alpha. TopAppBar now exposes `.chrome`, `.title`, `.collapsed-title`, and
`.expanded-title`, proves Large 152/108/64dp collapse geometry, keeps explicit `.scrolled(true)`
Medium/Large bars on the fully scrolled color path when no scroll behavior is attached, and uses
fractional color only for scroll-behavior-driven layouts. Flexible app bars, subtitles,
predictive-back integration, RTL-specific title placement, and exhaustive variant/theme visual
matrices remain residual/future API work.

M3PV2-077 is complete: Dialog layout/accessibility/current modal motion are now v2-covered for the
current AlertDialog-style recipe surface. Compose Material3 and Base UI were the primary sources:
dialogs constrain panel width to 280..560dp, use 24dp panel padding, 16dp title-to-text spacing,
24dp text-to-actions spacing, 8dp action spacing, and expose the panel as a labelled/described
dialog. The packet found Material recipe/proof-density gaps, not core or kit overlay policy gaps:
focus containment/restore, scrim dismissal, and the shared modal fade/rise/scale foundation were
already green, but Dialog lacked stable headline/supporting/actions parts, did not wire
labelled/described relations, did not clamp the semantic panel to 560dp in the harness, and stacked
content directly in the container instead of a vertical content column. Dialog now exposes
`.headline`, `.supporting-text`, and `.actions`, centers a tokenized 280..560dp panel, proves
spacing/relations/motion in `dialog_state`, and refreshes menu-dialog headless goldens for the
intentional centered vertical content signature. Full-screen dialogs, compact pointer padding,
predictive-back choreography, and broader overlay-family policy comparison remain residual/future
API work.

M3PV2-078 is complete: Snackbar layout/accessibility/enter-exit motion are now v2-covered for the
current toast-backed recipe surface. Compose Material3 was the primary source: SnackbarHost applies
12dp host padding, Snackbar caps container width at 600dp, uses 48dp/68dp single-line/two-line
heights, labels the live pane as `Alert`, labels the close affordance as `Dismiss`, and animates
visibility with fade plus scale from/to 0.8. The packet found a Material recipe gap plus a reusable
`fret-ui-kit` toast-surface gap, not a core mechanism gap: SnackbarHost still inherited generic
toast width/offset/close-label defaults, while the kit renderer had no design-system-agnostic
scale-from style slot and used style min-height tokens only for stack estimation. `ToastLayerStyle`
now has an optional `scale_from` with `None` preserving Sonner/shadcn defaults, toast surfaces apply
style min-height to real layout, and Material Snackbar wires 12dp margin/mobile offset, 600dp max
width, `Alert`, `Dismiss`, and zero-slide fade-scale motion. Queue, action, close, timer, and
removal behavior stayed green through existing kit and Material tests.

M3PV2-079 is complete: Tooltip layout/accessibility/current overlay motion are now v2-covered for
the current plain/rich recipe surface. Compose Material3 was the primary source: PlainTooltip uses
40x24dp minimum sizing and a 200dp max width; RichTooltip uses 40x24dp minimum sizing, a 320dp max
width, 16dp horizontal padding, title-aware vertical padding, and rich elevation/shape; Tooltip
visibility fades and scales from/to 0.8; BasicTooltip marks the popup as an assertive live region.
The packet found a Material recipe/token proof-density gap, not a core or kit mechanism gap:
Tooltip already used the shared overlay-motion foundation, and kit already owned delay groups,
safe hover, touch suppression, click-through overlays, and trigger `described_by` wiring. Material
Tooltip now uses separate plain/rich max-width tokens, applies the 40x24dp surface minimum, marks
the popup as an assertive live region, and has `tooltip_state` tests proving role, relation,
layout, first-open fade-scale, and first-close fade-scale. Rich tooltip action rows remain
residual because `OverlayLayerKind::Tooltip` is intentionally non-hit-testable; a portable
pane-title semantics field remains a future core a11y decision.

## Decisions

- This lane is about shadcn-level proof density, not shadcn visual styling.
- Material spec, Compose Material3, MUI Material UI, and Base UI are axis-specific references.
- Stable Fret-side shadcn components are exemplars for layering and gates only.
- Layout defaults must be classified as intrinsic recipe defaults or caller-owned before edits.
- Shared foundation refactors require multiple component proofs.

## Next Recommended Action

Continue with the next uncovered Material3 packet from the matrix. Higher-priority candidates now
move through remaining overlay surfaces: Menu/DropdownMenu item layout/focus packets are still
open. Broader overlay-family policy comparison remains in M3PV2-050.

## Useful Gates

```powershell
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test select_behavior
```
