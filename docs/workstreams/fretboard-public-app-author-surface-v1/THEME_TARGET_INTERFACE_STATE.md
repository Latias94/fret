# Fretboard Public App-Author Surface v1 — Theme Import Target Interface State

Status: target posture for `theme import-vscode`
Last updated: 2026-04-09

This document freezes the intended **public posture** for VS Code theme import.

## Target reading rule

`theme import-vscode` is a format-conversion utility:

- input: a VS Code theme JSON plus optional mapping/base files,
- output: a Fret theme config JSON and optional report,
- scope: theme-authoring convenience, not the core app lifecycle.

That means its package boundary should be judged by utility shape and dependency focus, not by
whether it is merely possible to hang another subcommand off `fretboard`.

## M4 decision

The public posture is:

- do **not** add `theme import-vscode` to public `fretboard` v1,
- keep the current command on `fretboard-dev` for now,
- and treat the long-term public home, if we choose to publish one, as a dedicated package built on
  `fret-vscode-theme`, not as part of the main `fretboard` product.

## Reasoning

Why it should not live on public `fretboard`:

1. `fretboard` should stay focused on the app-author lifecycle:
   - create
   - configure
   - run
   - diagnose
2. `theme import-vscode` is project-agnostic, but it is not part of that lifecycle. It is a niche
   conversion utility.
3. The repo already has a domain-specific library boundary:
   - `ecosystem/fret-vscode-theme`
4. Pulling theme/syntax conversion dependencies into the main public CLI would widen
   `fretboard`'s dependency closure for a sidecar workflow rather than a core product capability.

Why it should not stay permanently repo-only:

- the command itself is not mono-repo-shaped,
- external users may reasonably want this utility,
- and the existing library boundary makes a dedicated package a natural future home.

## Public target posture

Public `fretboard` v1:

- no `theme` command

Repo-only retained for now:

- `fretboard-dev theme import-vscode <input> [--out ...] [--base ...] [--report ...]`

Future public direction, if demand justifies it:

- ship a dedicated thin package around `fret-vscode-theme`
- keep the user-facing contract file-in / file-out and project-agnostic
- avoid pulling theme-conversion concerns into the main app-lifecycle CLI

## Migration mapping

| Current surface | Target posture |
| --- | --- |
| `fretboard-dev theme import-vscode ...` | retained temporarily on repo-only CLI |
| `fretboard theme import-vscode ...` | not part of public v1 |
| future public theme converter | dedicated package, not main `fretboard` |

## Explicit non-targets

This document does **not** decide:

- the name of any future dedicated package,
- whether the future public utility is CLI-only or paired with example templates,
- or whether additional theme conversion/import tools should live beside VS Code import.

## Done-state summary

The done state is not "every generally useful utility moves onto `fretboard`".

It is:

- `fretboard` stays centered on the primary app-author lifecycle,
- `theme import-vscode` has an explicit non-`fretboard` public posture,
- and the future public path, if needed, is a focused package around `fret-vscode-theme`.
