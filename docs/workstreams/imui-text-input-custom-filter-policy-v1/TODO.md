# ImUi Text Input Custom Filter Policy v1 TODO

Status: Closed
Last updated: 2026-05-04

## M0 - Scope

- [x] Confirm Dear ImGui callback filter runs after named filters.
- [x] Keep Fret's custom filter insertion-only.
- [x] Avoid runtime callback widening.

## M1 - API

- [x] Add `InputTextCustomFilter`.
- [x] Add `InputTextOptions::custom_filter`.
- [x] Re-export the type from the IMUI kit surface.

## M2 - Wiring

- [x] Compose named filters before custom filters.
- [x] Reuse `TextInputProps::insert_filter`.
- [x] Keep `fret-imui` unchanged except tests.

## M3 - Gates And Closeout

- [x] Add a model-backed IMUI regression test proving filter order.
- [x] Update roadmap/workstream/audit docs.
- [x] Leave follow-on policy for richer completion/history recipes; undo/redo command routing is
  covered by `docs/workstreams/imui-text-input-undo-command-policy-v1/`.
