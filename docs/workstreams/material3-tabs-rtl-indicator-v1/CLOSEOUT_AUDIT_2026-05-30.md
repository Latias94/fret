# Material3 Tabs RTL Indicator v1 - Closeout Audit

Status: Closed
Date: 2026-05-30

## Result

Closed as a focused component-policy parity slice.

Tabs now resolves Material layout direction, flips horizontal ArrowLeft/ArrowRight behavior under
RTL, and mirrors active-indicator fallback positioning when measured geometry is not available. The
public Material3 context facade also exposes layout direction overrides, so consumers no longer need
to rely on private foundation APIs or raw theme tokens.

## Verification

- Unit coverage locks keyboard direction mapping and fallback indicator coordinate mapping.
- Diagnostics integration coverage verifies RTL ArrowLeft moves forward from the first tab when
  wrapping is disabled.
- Full `tabs_state`, check, clippy, and workstream catalog gates are part of the lane gates.

## Residual Risk

This does not claim full RTL row mirroring. The Fret layout engine still needs a separate physical
layout direction contract before Tabs can match Compose `placeRelative` behavior end-to-end for all
measured geometry and scroll positioning.
