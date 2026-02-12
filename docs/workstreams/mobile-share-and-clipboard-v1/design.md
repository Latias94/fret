---
title: Mobile Share + Clipboard (v1)
status: draft
date: 2026-02-12
scope: contract-first share/export + open-in/import + clipboard portability (Android-first)
---

# Mobile Share + Clipboard (v1) — Workstream

This workstream focuses on two mobile-facing platform bridges that are “hard to change” once app
ecosystems depend on them:

- share/export via the platform share sheet,
- open-in/import via OS intents,
- clipboard reads/writes under mobile privacy constraints.

The goal is to lock portable **contracts first** (ADRs), then implement minimal runner/platform
glue with diagnostics gates so future Android/iOS support does not require cross-repo rewrites.

## Principles

- `crates/fret-ui` stays mechanism-only (ADR 0066).
- Platform APIs are accessed via effects and token-based resources (ADR 0003).
- Reads are bounded and explicit (limits + explicit release) to keep mobile predictable.
- Treat clipboard reads as best-effort “commands”, not a reactive data feed.

## Contract anchors (v1)

- Share sheet + open-in intent bridge: `docs/adr/0265-mobile-share-sheet-and-open-in-intents-v1.md`
- Mobile clipboard portability: `docs/adr/0266-mobile-clipboard-portability-v1.md`
- External payload portability and read limits: `docs/adr/0053-external-drag-payload-portability.md`
- File picker + sandbox handles (import alignment): `docs/adr/0264-mobile-file-picker-and-sandbox-handles-v1.md`
- Clipboard/DnD baseline (desktop-first): `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

## Tracking

- TODO list: `docs/workstreams/mobile-share-and-clipboard-v1/todo.md`
- Milestones: `docs/workstreams/mobile-share-and-clipboard-v1/milestones.md`

