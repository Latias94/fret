# Material3 Button Elevation State Motion Packet v2

Date: 2026-05-29
Task: M3PV2-047

## Truth

- Elevated buttons paint Material elevation by default, using separate key and ambient shadow
  layers from the existing Material surface/elevation foundation.
- Filled and tonal buttons remain level0 at rest, then animate to Material level1 elevation on
  hover; pressed/focused/default states follow the button elevation state matrix.
- Button pressed shape morphing uses the Compose Material3 `DefaultEffects` channel so the shape
  transition stays non-bouncy rather than using spatial-surface motion.
- Button semantics expose role, accessible label, disabled flag, and disabled invoke behavior.
- Existing layout invariants remain intact: Material min width, token-driven size/padding, and
  shrinkable label text between icon slots.

## Sources

- Compose Material3 `Button.kt`: `Surface` owns button color/elevation/shape/border, `Row`
  applies `defaultMinSize(minWidth = ButtonDefaults.MinWidth, minHeight = ButtonDefaults.MinHeight)`,
  and the animated-shape overload intentionally uses `MotionSchemeKeyTokens.DefaultEffects`.
- Compose Material3 `internal/Elevation.kt`: elevation animates into hover/focus/press states and
  snaps when entering disabled state.
- Compose Material3 button tokens: small button height, icon spacing, content padding, elevation
  levels, and disabled opacity.
- Material Web v30 token exports: button elevation, state-layer opacity, shadow-color, outline,
  size, and shape keys already exist in the Fret v30 theme.
- MUI Button is a web composition reference for button root semantics, min width, and stateful
  `box-shadow` transitions; it is not the Material 3 token truth.

## Artifacts

- `ecosystem/fret-ui-material3/src/button.rs`
- `ecosystem/fret-ui-material3/src/tokens/button.rs`
- `ecosystem/fret-ui-material3/tests/button_state.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Wiring

- Button recipe now resolves variant/state elevation through `tokens::button` and feeds the result
  into `foundation::surface::material_surface_style`, which applies the existing Material shadow
  lowering.
- Button elevation animates with Compose-aligned incoming/outgoing timings and snaps when disabled.
- The existing Material ink layer remains responsible for state-layer/ripple painting; the button
  now keeps requesting frames while either shape or elevation motion is active.
- `ButtonStyle` gained an override slot for container elevation, matching the existing background,
  label, outline, and state-layer override model.

## Proof

Red gate before the fix:

```powershell
cargo nextest run -p fret-ui-material3 --test button_state
```

Failed because elevated buttons painted no shadow layers and filled buttons never animated to a
hover elevation shadow.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test button_state
cargo nextest run -p fret-ui-material3 --lib button
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1
```

Result: the new scene tests prove default elevated shadows and filled-hover elevation animation;
the semantics test proves role/label/disabled outcomes; lib tests prove the elevation matrix,
DefaultEffects shape spring, Material min width, and label shrink behavior; existing automation
and headless controls goldens stayed green without a golden refresh.

## Layer Decision

This is a Material recipe/token gap, not a `crates/fret-ui` or `fret-ui-kit` mechanism gap:

- `ContainerProps.shadow`, Material surface style, elevation-to-shadow lowering, pressable
  semantics, and indication primitives already existed.
- The missing wiring was inside the button recipe and button token resolver.
- No design-system-agnostic behavior was added to `fret-ui-kit`, and no renderer/core contract
  changed.

## Residual Risk

- Button loading states, full-width web API parity, and button-group composition remain outside the
  current Fret Material3 Button surface.
- This packet closes the standalone Button row. Related components such as FAB, icon button,
  segmented button, chips, and cards still need their own state/elevation/motion packets.
