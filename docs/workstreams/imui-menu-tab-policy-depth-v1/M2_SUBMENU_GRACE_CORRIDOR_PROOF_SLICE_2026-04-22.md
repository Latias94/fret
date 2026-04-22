# M2 Submenu Grace Corridor Proof Slice - 2026-04-22

Status: landed slice within the active lane

## Why this note exists

`M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md` already claimed that generic IMUI shipped submenu
hover-open, sibling hover-switch, and a basic grace corridor.

The new focused IMUI repro showed that this was not yet true end-to-end inside the immediate-mode
helper stack:

- the submenu primitive already owned delayed hover-open and pointer-grace intent in
  `primitives/menu/sub.rs`,
- but the generic IMUI hover-query runtime still overwrote component hover handlers, and
- `begin_submenu_with_options(...)` still had a helper-local hover path that selected sibling
  submenus directly from hover state.

That meant the primitive owner existed, but IMUI integration could still bypass it.

## Landed slice

Generic IMUI now enforces submenu grace behavior through the submenu primitive owner path instead of
through helper-local hover state:

- `install_hover_query_hooks_for_pressable(...)` now composes with existing hover handlers via
  `pressable_add_on_hover_change(...)` instead of overwriting them,
- `begin_submenu_with_options(...)` no longer selects sibling submenus directly from
  `hovered_signal`,
- hover-open and sibling hover-switch now flow through `sub_trigger::wire(...)` and the shared
  `sub.rs` delay / pointer-grace logic,
- and the focused IMUI proof now demonstrates that hitting a real sibling trigger point inside the
  grace corridor does not switch away from the open submenu.

Proof hardening on the same day also locked the adjacent safe-hover path that Dear ImGui comments
about in `BeginMenuEx(...)`:

- moving from an unsafe point back into the submenu-side void corridor now cancels the armed
  close-delay timer in the focused IMUI proof,
- so the current floor is no longer only "sibling switch is deferred inside the polygon",
- it also covers "submenu does not collapse while the pointer is moving toward the child through
  safe corridor space".

This keeps the owner split coherent:

- generic IMUI hover query remains generic infrastructure,
- submenu delay / grace semantics stay in the submenu primitive,
- and the helper layer only projects explicit click/open/close state into popup visibility.

## Evidence anchors

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/sub.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/pointer_grace_intent.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
- `repo-ref/imgui/imgui_widgets.cpp`

## Gates run

- `cargo nextest run -p fret-imui begin_submenu_helper_hover_opens_submenu_after_pointer_entry --no-fail-fast`
- `cargo nextest run -p fret-imui begin_submenu_helper_hover_switches_sibling_after_open_delay --no-fail-fast`
- `cargo nextest run -p fret-imui begin_submenu_helper_defers_sibling_switch_inside_grace_corridor --no-fail-fast`
- `cargo nextest run -p fret-imui begin_submenu_helper_safe_corridor_cancels_close_timer --no-fail-fast`
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`
- `git diff --check`

## Consequence for the lane

The remaining open question is now narrower than “does IMUI actually have submenu grace at all?”.

What remains open is only whether any richer submenu-intent tuning beyond the current enforced
corridor belongs in generic IMUI, or whether that pressure should close on a shell/product owner
verdict without widening the helper family again.
