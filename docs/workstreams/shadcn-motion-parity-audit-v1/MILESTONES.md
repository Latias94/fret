# Shadcn Motion Parity Audit v1 — Milestones

Last updated: 2026-03-04.

## M0 — Baseline agreement (audit-ready)

- Define motion taxonomy (transition vs continuous) and ownership rules.
- Populate `PARITY_MATRIX.md` with initial audit rows and evidence anchors.

## M1 — Discrete transitions parity

- NavigationMenu trigger chevron rotates with a tokenized transition (duration/easing).
- Reduced motion: chevron snaps to target and does not request continuous frames.
- Gate: deterministic diag or unit test.

## M2 — Continuous animations parity (duration-driven)

- Spinner: stable continuous rotation with reduced-motion settling.
- Skeleton: stable pulse with reduced-motion settling.
- InputOtp: caret blink aligned to upstream `duration-1000` semantics.
- Gates: at least one per component.

## M3 — Public API cleanup (fearless refactor)

- Remove public “per-frame” speed semantics where possible.
- Ensure no component-level math reintroduces fixed 60Hz coupling.
- Update docs/evidence anchors for any moved symbols.
