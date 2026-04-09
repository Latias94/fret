# Fretboard Public Diag Implementation v1 Milestones

Status: Active
Last updated: 2026-04-09

## M0. Lane open

- Status: done
- Follow-on lane exists with explicit repro/gate/evidence.
- Prior taxonomy lane remains closed.

## M1. Diagnostics CLI seam exists

- Status: done
- `fret-diag` can render diagnostics help in more than one product mode.
- Repo-only branding no longer hardcodes the future public product surface.

## M2. Public-core command set is enforceable

- Status: done
- The shipped public verb subset is explicit in code.
- Repo-only verbs are rejected or absent from the public path.

## M3. Public `fretboard diag` exists

- Status: done
- `crates/fretboard` exposes `diag`.
- Help and examples teach explicit scripts/bundles/launch commands.
- Execution does not depend on suite/campaign/registry catalogs.

## M4. Publish/docs closure

- Status: in progress
- The `fret-diag` dependency posture is explicit and publishable.
- `fret-diag` dry-run succeeds and release closure puts it ahead of `fretboard`.
- Remaining work is the real publish sequence plus public docs/ADR wording.
