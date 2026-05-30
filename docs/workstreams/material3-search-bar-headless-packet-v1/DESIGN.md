# Material 3 SearchBar Headless Automation Packet v1

Date: 2026-05-28
Status: Closed

## Problem

SearchBar already had stable selector and headless coverage, but the component matrix still left it
as a known follow-on. The correct closure path is to record the automation and headless evidence,
not to fabricate a gallery surface or move SearchBar policy into a shared kit layer.

## Target State

- SearchBar exposes stable dotted ids for the root, chrome, leading icon, and trailing icon.
- Recipe code owns pill field chrome, input surface, and icon slots.
- Material foundation owns token plumbing, state-layer/ripple indication, and interactive size
  policy.
- Diagnostics and test harness code own automation coverage and headless goldens.
- SearchView presentation and overlay policy remain separate.

## Source Truth

- Material 3 spec for field chrome and state behavior.
- Compose Material3 `SearchBar.kt` for the reference component shape.
- Fret `automation_surface.rs` and `radio_alignment.rs` for live selectors and headless state
  coverage.

## Layer Ownership

- `ecosystem/fret-ui-material3/src/search_bar.rs`: recipe composition and stable ids.
- `ecosystem/fret-ui-material3/src/tokens/*`: theme/token plumbing and imported Material web
  tokens.
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`: selector evidence.
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`: headless golden evidence.
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/*`: matrix context and
  field-family audit.
- `docs/workstreams/material3-search-view-state-packet-v1/*`: separate SearchView overlay and
  presentation lane.

## In Scope

- Close the matrix residual for SearchBar.
- Record the headless and automation evidence in a dedicated packet.
- Keep the boundary clear between SearchBar and SearchView.

## Out Of Scope

- New gallery scripts.
- SearchView overlay or presentation changes.
- Shared kit or mechanism changes.
- Additional field-family policy beyond what the existing SearchBar surface already proves.

## Closeout Condition

This lane is complete once the matrix row is updated, the dedicated workstream docs exist, and the
focused nextest gates pass.
