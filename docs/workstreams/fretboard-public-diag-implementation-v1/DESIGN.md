# Fretboard Public Diag Implementation v1

Status: Active execution design
Last updated: 2026-04-09

## Problem

The public CLI taxonomy is frozen, but the shipped public `fretboard` package still lacks the
project-facing diagnostics core that external app authors need after they begin capturing scripted
diagnostics bundles.

The repo already has a rich `fretboard-dev diag` implementation, but that surface is not
publishable as-is because it currently mixes:

- repo-only help/branding,
- suite/campaign/registry inventory workflows,
- script-id/catalog defaults tied to `tools/diag-scripts`,
- and a non-published `fret-diag` implementation crate.

## Scope

This lane implements the first shipped public `fretboard diag` surface.

In scope:

- mode-aware CLI/help/usage branding for diagnostics,
- explicit public-core verb allowlisting,
- `crates/fretboard` integration for the shipped public subset,
- publish-boundary audit for `fret-diag` and its dependency closure,
- docs/help/ADR alignment for the public diagnostics story.

Out of scope:

- public suite/campaign/registry workflows,
- repo dashboard/index/list inventory helpers,
- public devtools inspector posture,
- public hotpatch,
- theme import packaging.

## Implementation stance

### 1. Public diagnostics stay explicit

The public story must stay centered on explicit script paths, explicit bundle paths, and explicit
`--launch -- <cmd...>` commands.

Implications:

- no repo script registry is required for the baseline story,
- no suite/campaign taxonomy leaks into public docs,
- and bundle inspection verbs teach artifact analysis, not repository maintenance.

### 2. `fret-diag` needs a product-mode seam before public wiring

The current diagnostics implementation is deeply reusable, but its CLI contract layer still
assumes the repo-only product.

Implications:

- branding/help/bin-name hardcoding must be removed first,
- then the public allowlist can be enforced without duplicating the command tree blindly,
- and only after that should `crates/fretboard` grow a real `diag` entrypoint.

### 3. Publish boundary is an implementation constraint, not a second product

If `fret-diag` must be published to make public `fretboard diag` possible, that is acceptable.
It does not create a second end-user CLI.

Implications:

- any package split must still preserve `fretboard diag` as the taught user surface,
- `fret-diag` publication work should be judged by dependency correctness and portability,
- and repo-only diagnostics helpers may remain on `fretboard-dev` even if the library is split.

## Done state

The lane is done when:

1. `fretboard --help` exposes public `diag`.
2. `fretboard diag --help` teaches only the public-core diagnostics story.
3. The shipped public verbs execute without depending on repo suite/registry/campaign inventory.
4. `fretboard-dev diag` can retain richer maintainer workflows without widening the public
   contract.
