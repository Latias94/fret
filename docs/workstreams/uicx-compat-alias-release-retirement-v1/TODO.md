# UiCx Compat Alias Release Retirement v1 — TODO

Status: Active

## Baseline evidence

- [x] UICX-RR-001 Record why the previous broad lane stays closed and why this narrower lane exists.
- [x] UICX-RR-002 Freeze the current `UiCx*` inventory across:
  - root compatibility aliases,
  - explicit app/advanced exports,
  - and deprecated hidden carrier aliases.
- [x] UICX-RR-003 Freeze the current canonical teaching posture on
  `AppComponentCx<'a>` / `AppRenderCx<'a>` / `AppRenderContext<'a>`.

## Release policy

- [ ] UICX-RR-010 Decide the release-window posture for the explicit compatibility aliases:
  - `UiCx<'a>`
  - `UiCxActionsExt`
  - `UiCxDataExt`
- [ ] UICX-RR-011 Decide whether the hidden deprecated carriers retire:
  - in the same release-facing slice as the explicit aliases, or
  - in a separate earlier slice with proof that they are not part of the supported semver surface.
- [ ] UICX-RR-012 Write the explicit removal criteria:
  - required downstream evidence,
  - required release note / migration wording,
  - and required gate set before alias deletion is acceptable.

## Implementation / migration

- [ ] UICX-RR-020 If the verdict is retain, keep deprecation wording and release docs consistent.
- [ ] UICX-RR-021 If the verdict is remove, delete the chosen alias set in one explicit reviewed
  slice or an explicitly justified split slice.
- [ ] UICX-RR-022 Refresh tests, source-policy gates, and release preflight after the verdict
  lands.

## Closeout

- [ ] UICX-RR-030 Close the lane with one explicit verdict:
  - retained for a defined release window, or
  - removed with release-facing evidence and migration notes.
