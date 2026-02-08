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
| `fret-core` | L1 | `docs/workstreams/crate-audits/fret-core.l1.md` |
| `fret-runtime` | L1 | `docs/workstreams/crate-audits/fret-runtime.l1.md` |
| `fret-app` | L1 | `docs/workstreams/crate-audits/fret-app.l1.md` |
| `fret-ui` | L1 | `docs/workstreams/crate-audits/fret-ui.l1.md` |
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

We keep this list selective: ecosystem is larger and more experimental, so the audits focus on
crates that are likely to produce long-lived contracts or that sit on major interaction seams.

| crate | status | note |
| --- | --- | --- |
| `fret-ui-headless` | Not started | |
| `fret-ui-kit` | L0 | `docs/workstreams/crate-audits/fret-ui-kit.l0.md` |
| `fret-ui-shadcn` | Not started | |
| `fret-docking` | Not started | |

## `apps/` (demo shells)

App audits focus on “golden path” quality and diagnostics, not on long-lived contracts.

- `fretboard`
- `fret-ui-gallery`
