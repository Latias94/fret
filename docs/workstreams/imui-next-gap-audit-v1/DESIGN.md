# ImUi Next Gap Audit v1 - Design

Status: closed narrow P1 audit lane
Last updated: 2026-04-24

## Problem

The recent IMUI lanes closed collection helper readiness, the editor-notes inspector command proof,
and the editor-notes draft-status proof. The repo now needs a narrow evidence pass that chooses the
next non-multi-window IMUI gap without reopening broad umbrella lanes or widening public helper APIs
by default.

## Scope

Owned here:

1. Audit the remaining Dear ImGui-class gap after the recent closed follow-ons.
2. Rank the next locally testable, non-macOS-dependent IMUI follow-on candidates.
3. Keep closed lane verdicts authoritative unless fresh evidence names one exact new owner.
4. Name one recommended next lane and its minimum repro/gate surface.

Not owned here:

1. No `fret-ui-kit::imui`, `fret-imui`, `fret-authoring`, or `crates/fret-ui` API widening.
2. No macOS-only or multi-window runner implementation.
3. No new editor persistence, dirty-close prompt, or document-state runtime contract.
4. No component-library policy migration into `crates/fret-ui`.

## Target Outcome

Close this audit with a ranked next-gap decision that keeps the default next slice app-owned and
locally testable while parking multi-window/tear-off work until backend acceptance can be captured.
