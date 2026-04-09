# Fretboard Public Dev Implementation v1

Status: Historical reference (closed execution design)
Last updated: 2026-04-09

Status note (2026-04-09): this design remains the authoritative implementation rationale for the
first shipped public `fretboard dev` surface, but the lane itself is now closed. Follow-on work for
public `diag`, hotpatch, or theme-import packaging should start as narrower lanes instead of
reopening this one.

## Problem

The public CLI taxonomy is now frozen, but the shipped public `fretboard` package still lacks the
project-facing `dev` command that external app authors need after scaffolding a new app.

The repo already has a rich maintainer-only `fretboard-dev dev` implementation, but that surface is
not publishable as-is because it depends on:

- repo demo ids,
- interactive demo choosers,
- cookbook-specific hints,
- hotpatch-only orchestrators,
- and mono-repo shell packages.

## Scope

This lane implements the first shipped public `fretboard dev` surface in `crates/fretboard`.

In scope:

- typed clap surface for public `dev`,
- Cargo-project resolution by manifest path,
- package and target selection from `cargo metadata`,
- native run/watch/supervisor loop for selected Cargo `bin` / `example`,
- web runner integration only when it can stay package-root/index.html based,
- public docs/help/ADR wording updates.

Out of scope:

- public diagnostics,
- public hotpatch,
- repo demo registry aliases,
- chooser/list surfaces,
- theme import packaging.

## Implementation stance

### 1. Selection must be Cargo-shaped

Public `dev` should derive package and target resolution from `cargo metadata`, not from Fret-owned
registries.

Implications:

- `--manifest-path` is the root user input.
- `--package` disambiguates workspace members.
- `--bin` and `--example` select actual Cargo targets.
- ambiguous default selection fails with explicit hints instead of interactive prompts.

### 2. Native run loop may reuse repo-only mechanics, but not repo-only product semantics

The repo-only native implementation already contains reusable mechanics:

- polling workspace watcher,
- rebuild + restart loop,
- supervisor,
- dev-state env shaping.

Those mechanics can be copied/adapted into the public crate as long as the public contract does not
inherit demo ids, hotpatch hints, or repo package names.

### 3. Web stays package-root first

If public `dev web` lands in this lane, it must run against the selected package root and its
`index.html`, not `apps/fret-demo-web`.

That implies:

- choose the package via Cargo metadata,
- require an `index.html` at the selected package root,
- keep Trunk invocation rooted at that package directory,
- only synthesize a temporary HTML target when bin disambiguation must be made explicit.

## Done state

The lane is done when:

1. `fretboard --help` exposes public `dev`.
2. `fretboard dev native ...` works for an external-style Cargo project.
3. Docs no longer describe public `dev` as a future-only surface once the shipped native path
   exists.
4. Repo-only `fretboard-dev` remains free to keep richer demo/hotpatch helpers without widening the
   public package contract.
