# Bottom-Up Fearless Refactor v1 — Crate Audits

Status: Draft (tracking index; per-crate notes should stay evidence-backed)

This is the progress index for the “code-quality audit” pass described in:

- `docs/workstreams/bottom-up-fearless-refactor-v1.md`

Template for per-crate notes:

- `docs/workstreams/bottom-up-fearless-refactor-v1-crate-audit-template.md`

## Tracking legend

- `Not started`: no audit note yet
- `L0`: quick scan complete
- `L1`: targeted deep dive complete
- `L2`: closure audit complete

## `crates/` (kernel + backends + renderer)

| crate | status | note |
| --- | --- | --- |
| `fret-core` | Not started | |
| `fret-runtime` | Not started | |
| `fret-app` | Not started | |
| `fret-ui` | Not started | |
| `fret-render-core` | Not started | |
| `fret-render-wgpu` | Not started | |
| `fret-render` | Not started | |
| `fret-platform` | Not started | |
| `fret-platform-native` | Not started | |
| `fret-platform-web` | Not started | |
| `fret-runner-winit` | Not started | |
| `fret-runner-web` | Not started | |
| `fret-launch` | Not started | |
| `fret` | Not started | |

## `ecosystem/` (policy + components + tools)

We will keep this list coarse until we decide audit scope and priorities for ecosystem crates
(many are experimental and may be removed/merged).

Priority candidates (expected to produce long-lived contracts):

- `fret-ui-headless`
- `fret-ui-kit`
- `fret-ui-shadcn`
- `fret-docking`

## `apps/` (demo shells)

App audits focus on “golden path” quality and diagnostics, not on long-lived contracts.

- `fretboard`
- `fret-ui-gallery`

