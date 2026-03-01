# `fret-interaction`

Small, reusable interaction helpers for Fret (gesture thresholding, drag deltas, and related
primitives).

This crate is intentionally tiny and deterministic so it can be shared by:

- immediate-mode style authoring helpers (`fret-ui-kit` `imui` facade),
- docking and multi-window drag choreography,
- canvas / node-graph interactions.

## Upstream references (non-normative)

- Dear ImGui (drag thresholds and interaction vocabulary): https://github.com/ocornut/imgui
- dear-imgui-rs (Rust binding reference): https://github.com/Latias94/dear-imgui-rs
