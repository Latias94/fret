# fret-bootstrap-shadcn

Thin `fret-bootstrap` + `fret-ui-shadcn` bridge crate.

This crate keeps recipe-heavy default UI wiring out of `fret-bootstrap` while preserving the
golden-path command palette story for shadcn-first apps.

Current scope:

- install the default shadcn command palette overlay on `fret-bootstrap::ui_app_driver::UiAppDriver`

Use this crate when:

- you want `fret-bootstrap`'s command palette capability,
- and you want the default shadcn `CommandDialog` presentation instead of supplying your own
  overlay renderer.
