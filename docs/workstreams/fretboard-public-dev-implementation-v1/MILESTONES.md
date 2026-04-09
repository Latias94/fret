# Fretboard Public Dev Implementation v1 Milestones

Status: Closed
Last updated: 2026-04-09

## M0. Lane open

- Status: done
- Follow-on lane exists with explicit repro/gate/evidence.
- Prior taxonomy lane remains closed.

## M1. Public command exists

- Status: done
- `crates/fretboard` exposes `dev`.
- Typed parsing covers public native/web target inputs.
- Target selection is based on Cargo metadata rather than repo demo catalogs.

## M2. First shipped run loop

- Status: done
- `fretboard dev native` runs a selected project target.
- Watch/restart and supervisor semantics are available without repo-only flags.
- `fretboard dev web` runs from the selected package root and its `index.html`.

## M3. Docs align with shipped behavior

- Status: done for `dev`, follow-on remains for public `diag` / hotpatch / theme import posture.
- Public docs no longer describe shipped `dev native/web` as future-only.
- Remaining non-shipped follow-ons (`diag`, public hotpatch, theme import packaging) stay
  explicitly scoped.

## M4. Lane closeout

- Status: done
- Public `fretboard dev` native/web implementation is shipped in `crates/fretboard`.
- Native/help/tests gates passed, and the web path was closed with both a real `dev web` smoke and
  a terminating `wasm32-unknown-unknown` compile check.
