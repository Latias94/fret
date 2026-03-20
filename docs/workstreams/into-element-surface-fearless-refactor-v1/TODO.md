# Into-Element Surface (Fearless Refactor v1) — TODO

Status: maintenance-only closeout tracker

This TODO list tracks the work described in `DESIGN.md`.

Because this is a pre-release reset, "done" means we actually delete superseded public conversion
names rather than preserve them for inertia.

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TARGET_INTERFACE_STATE.md`
- `MIGRATION_MATRIX.md`
- `CLOSEOUT_AUDIT_2026-03-20.md`

Closeout reading rule on 2026-03-16:

- this file is now a maintenance tracker, not the active owner of conversion-surface design
- read the settled public lane story from
  `../authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- remaining work here is helper-tail cleanup, explicit raw-seam inventory, source-policy gates,
  and stale-doc closeout
- historical execution notes below are archived evidence, not current sequencing instructions

Closeout note on 2026-03-18:

- there are now no unchecked execution items left on this lane,
- a sampled 2026-03-18 re-audit confirmed the representative closure gates still pass:
  - `reusable_component_helper_surface`
  - `copyable_ui_gallery_snippet_lane_has_no_top_level_raw_render_roots`
  - `direct_recipe_root_pages_mark_their_default_lane_without_inventing_compose`
  - `navigation_menu_and_pagination_pages_keep_their_dual_lane_story`
  - `gallery_doc_layout_retains_only_intentional_raw_boundaries`
  - `internal_preview_scaffold_retains_only_the_audited_vec_anyelement_seams`
- future edits in this folder should therefore be limited to drift control and audited seam
  inventory updates, not new conversion-surface design.

Closeout audit note on 2026-03-20:

- the lane now has an explicit closeout audit rather than only distributed sampled notes;
- treat the remaining checklist/history below as archived evidence, not as an open execution plan.

Historical execution note on 2026-03-13:

- this is now the first active interface-refactor lane,
- do M0/M1 here before expanding trait-budget follow-ups elsewhere,
- use the canonical compare set (`simple_todo_v2_target`, `todo_demo`, scaffold template) as the
  first downstream migration/evidence target after the unified trait lands.
- raw seam closure on the shadcn lane is now considered done; keep the next batches focused on
  remaining app/helper migration rather than reopening `kbd_icon(...)` or `text_edit_context_menu*`
  unless the underlying storage/builder model changes.

Historical execution note on 2026-03-14:

- the UI Gallery `data_table` family is now on the default app-facing surface:
  top-level snippets return `impl UiChild + use<>`, the page uses `DocSection::build(cx, ...)`,
  and the `guide_demo` state now lives inside the snippet instead of relaying through gallery
  window state.
- the obsolete gallery-wide relay fields `data_table_state` and
  `image_fit_demo_streaming_image` are now deleted from the UI gallery model/bootstrap/runtime
  path.
- the UI Gallery `motion_presets` family is now also on the default app-facing surface:
  top-level snippets return typed `UiChild` surfaces, the page uses `DocSection::build(cx, ...)`,
  `preset_selector` remains the explicit global motion-preset seam, and the remaining demos now
  keep their dialog/theme access inside the snippet.
- the cookbook shared page scaffold is now also on the default app-facing root lane:
  `apps/fret-cookbook/src/scaffold.rs` takes `UiCx` plus `UiChild` and returns `Ui`, and the
  canonical compare set (`hello_counter`, `simple_todo`, `simple_todo_v2_target`) no longer keeps
  a redundant `.into()` after that scaffold call.
- `apps/fret-examples/src/todo_demo.rs` and the generated todo/simple-todo helpers in
  `apps/fretboard/src/scaffold/templates.rs` now also keep their `todo_page(...)` helpers on that
  same typed-child lane (`impl UiChild`), so the canonical compare set no longer teaches
  `todo_page(...).into_element(cx).into()` as the default root wrapper pattern either.
- the onboarding cookbook `hello.rs` sample now also uses `hello_page(...) -> impl UiChild`, so
  the first-contact app example no longer teaches `let root = ...; root.into_element(cx).into()`
  in `render()`.
- the advanced `assets_demo` example now also follows the typed root-helper rule:
  `assets_page(cx, ...) -> Ui` owns the final landing, so the advanced/manual lane does not have
  to keep a root-local `let page = ...; page.into()` seam when the page wrapper itself is not raw.
- the advanced `embedded_viewport_demo` example now follows that same root-owned wrapper rule:
  `embedded_viewport_page(cx, ...) -> Ui` owns the final landing plus the optional diagnostics
  `test_id`, so the `render(...)` path no longer carries a separate root-local wrapper seam when
  the page itself is still ordinary typed composition.
- the advanced `genui_demo` example now also follows that root-owned wrapper rule:
  `genui_page(cx, ...) -> Ui` owns the final split-pane landing, so the view body no longer keeps
  a separate root-local `let page = ...` wrapper just to apply background/padding chrome.
- the advanced `query_demo` and `query_async_tokio_demo` examples now follow the same rule:
  `query_page(cx, ...) -> Ui` owns the centered card-shell landing, so the render path no longer
  teaches inline root wrapper chrome for that query authoring pair.
- the manual `App + UiTree` examples now also have an explicit typed root-helper rule instead of
  keeping inline page wrappers inside `render_root(...)` closures:
  `simple_todo_demo`, `cjk_conformance_demo`, and `emoji_conformance_demo` now keep their root
  shells on local `ElementContext<'_, App>` helpers returning `IntoUiElement<App>`.
- the first-contact `hello_counter_demo` on the default app lane now also keeps its root shell on
  `hello_counter_page(...) -> impl UiChild`, so that example no longer teaches inline root wrapper
  chrome inside `render(...)`.
- the advanced compare sample `hello_world_compare_demo` now also keeps its final root panel on a
  local `hello_world_compare_root(cx, ...) -> Ui` helper instead of leaving the whole final
  root-panel chain inline inside `render(...)`.
- the specialized `typography` teaching lane is now also aligned with that posture:
  UI Gallery typography snippets now expose `UiCx -> impl UiChild`, the page uses
  `DocSection::build(cx, ...)`, and the stale non-dev `dialog_open` relay is now gated back to
  `gallery-dev` only.
- the specialized `shadcn_extras` teaching lane is now aligned too:
  `apps/fret-ui-gallery/src/ui/snippets/shadcn_extras/*.rs` now expose
  `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/shadcn_extras.rs` now consumes those previews through
  `DocSection::build(cx, ...)`.
- the specialized `material3` lane has now advanced through its `controls` sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{badge,button,checkbox,icon_button,radio,segmented_button,slider,switch,touch_targets}.rs`
  now expose `UiCx -> impl UiChild`, and the new source gate locks those controls to the typed
  default-app teaching surface.
- the specialized `material3` lane has now also advanced through its `inputs` sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{autocomplete,date_picker,select,text_field,time_picker}.rs`
  now expose `UiCx -> impl UiChild`, and the field-family source gates now lock both copyable root
  ownership and the typed default-app authoring surface for those snippets.
- the specialized `material3` lane has now also advanced through its `navigation` sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{list,modal_navigation_drawer,navigation_bar,navigation_drawer,navigation_rail,tabs,top_app_bar}.rs`
  now expose `UiCx -> impl UiChild`, and the navigation/value-root source gates now lock both the
  typed top-level teaching surface and the removal of host-bound helper parameter spellings from
  the affected exemplar snippets.
- the specialized `material3` lane has now also advanced through its `overlays` sub-batch:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{bottom_sheet,dialog,menu,snackbar,tooltip}.rs`
  now expose `UiCx -> impl UiChild`, and the overlay source gates now lock both the typed
  top-level teaching surface and the local uncontrolled/copyable-root ownership story for the
  dialog/menu/bottom-sheet exemplars.
- the specialized `material3` lane is now fully aligned on the first-party default teaching
  surface:
  `apps/fret-ui-gallery/src/ui/snippets/material3/{gallery,state_matrix}.rs` now also expose the
  typed `UiCx -> impl UiChild` posture, their remaining helper signatures no longer spell
  `ElementContext<'_, H>`, and the composite source gates now close the last Material 3 teaching
  drift on this lane.
- the specialized `ai` lane has now advanced through its first curated snippet sweep:
  `apps/fret-ui-gallery/src/ui/snippets/ai/{agent_demo,artifact_demo,artifact_code_display,attachments_empty,attachments_grid,attachments_inline,attachments_list,attachments_usage,audio_player_demo,chain_of_thought_composable,chain_of_thought_demo,chat_demo,checkpoint_demo,code_block_demo,commit_custom_children,commit_demo,commit_large_demo,confirmation_accepted,confirmation_demo,confirmation_rejected,confirmation_request,context_default,context_demo,file_tree_basic,file_tree_demo,file_tree_expanded,file_tree_large,inline_citation_demo,message_demo,mic_selector_demo,model_selector_demo,open_in_chat_demo,package_info_demo,persona_basic,persona_custom_styling,persona_custom_visual,persona_demo,persona_state_management,persona_variants,plan_demo,prompt_input_action_menu_demo,prompt_input_docs_demo,prompt_input_provider_demo,prompt_input_referenced_sources_demo,reasoning_demo,schema_display_demo,shimmer_demo,shimmer_duration_demo,shimmer_elements_demo,snippet_demo,snippet_plain,sources_demo,stack_trace_collapsed,stack_trace_demo,stack_trace_large_demo,stack_trace_no_internal,task_demo,terminal_demo,test_results_basic,test_results_demo,test_results_errors,test_results_large_demo,test_results_suites,tool_demo,voice_selector_demo,web_preview_demo,workflow_canvas_demo,workflow_chrome_demo,workflow_connection_demo,workflow_controls_demo,workflow_edge_demo,workflow_node_demo,workflow_node_graph_demo,workflow_panel_demo,workflow_toolbar_demo}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and the new
  `ui_authoring_surface_default_app::ai_curated_snippets_prefer_ui_cx_on_the_default_app_surface`
  gate forbids those exemplars from drifting back to `ElementContext<'_, H> -> AnyElement`.
- the specialized `ai` lane has now also closed its remaining top-level tail:
  `apps/fret-ui-gallery/src/ui/snippets/ai/{canvas_world_layer_spike,conversation_demo,environment_variables_demo,image_demo,message_branch_demo,message_usage,queue_demo,sandbox_demo,speech_input_demo,suggestions_demo,transcript_torture,transcription_demo}.rs`
  now expose the same typed `UiCx -> impl UiChild` default-app posture, and the same AI source
  gate now covers those exemplars too.
  That reduces the remaining old-signature top-level `ai` snippet renders from 87 to 0.
- the corresponding AI docs pages are now also aligned on typed section registration:
  `apps/fret-ui-gallery/src/ui/pages/ai_*.rs` no longer use `DocSection::new(...)` for first-party
  demo sections, and the new
  `ui_authoring_surface_default_app::curated_ai_doc_pages_use_typed_doc_sections` gate prevents
  the AI docs surface from drifting back to eager `AnyElement` section registration.
- after these tracked landings, the current tracked default-app teaching-surface lane is
  effectively closed; remaining follow-up work now lives on the specialized `ai` lane plus any
  optional dead-field/runtime cleanup.
- this workstream now also treats first-party teaching cleanup as a family-taxonomy problem:
  the currently accepted lanes are
  `compose()` root default,
  dual-lane family,
  and direct recipe root/bridge.
- the overlay/menu sweep already closed the highest-value classified families:
  `DropdownMenu`, `ContextMenu`, `Dialog`, `Sheet`, `AlertDialog`, and `Drawer` now sit on the
  compose-root default lane;
  `Carousel` and `Menubar` are now explicitly dual-lane;
  `Popover`, `HoverCard`, and `Tooltip` stay on the direct recipe root/bridge lane.
- the remaining ambiguity on that queue is now closed too:
  `Select`, `Combobox`, and `Command` are now recorded as direct recipe root/bridge families,
  while `NavigationMenu` and `Pagination` are recorded as dual-lane families.
- the `Command` gallery lane is now also explicitly source-gated against split root authoring:
  first-party `apps/fret-ui-gallery/src/ui/snippets/command/*.rs` snippets keep the
  `command(...)` / `CommandPalette` root story and do not reintroduce `CommandInput::new(...)` or
  `CommandList::new(...)` on the default teaching surface.
- that direct-root lane has now also advanced from classification to exemplar cleanup:
  `Select` first-party snippets now prefer the compact
  `.trigger(...).value(...).content(...).entries(...)` root chain,
  `Combobox` now exposes root-lane `.trigger(...).input(...).clear(...).content(...)` builder
  steps, and the focused UI Gallery source gate now locks those compact default stories while
  leaving `into_element_parts(...)` as the explicit upstream-shaped adapter seam on the same lane.
- the `Select` gallery lane is now fully off closure-based parts adapters:
  `apps/fret-ui-gallery/src/ui/snippets/select/*.rs` no longer use `into_element_parts(...)` on
  the default teaching surface.
- the `Combobox` gallery lane is now fully off closure-based parts adapters too:
  the direct-root `Combobox::new(...)` snippets now use the compact chain by default, and
  `ComboboxChips` now also exposes matching root-lane builder steps so
  `combobox/multiple_selection.rs` no longer needs `into_element_parts(...)`.
- the form-input follow-up has now advanced too:
  `InputGroup` is now explicitly treated as a dual-lane family, with first-party default snippets
  preferring the compact `InputGroup::new(model)` slot shorthand while the addon/control parts
  remain the direct docs-parity lane, and the last `input_group/dropdown.rs` default snippet is now
  off `into_element_parts(...)`.
- `InputOtp` is now also classified and cleaned up as a direct recipe root/bridge family:
  first-party `apps/fret-ui-gallery/src/ui/snippets/input_otp/*.rs` now prefer the compact
  `InputOTP::new(model)` root builder with `length(...)` and optional `group_size(...)`, while
  `InputOTPGroup` / `InputOTPSlot` / `InputOTPSeparator` plus `into_element_parts(...)` remain the
  focused upstream-shaped bridge rather than the default copyable lane.
- the `Carousel` dual-lane story is now also explicitly reflected in the snippets themselves:
  `apps/fret-ui-gallery/src/ui/snippets/carousel/usage.rs` now mirrors the upstream-shaped docs
  lane, `compact_builder.rs` plus
  `apps/fret-ui-gallery/src/ui/snippets/carousel/{basic,sizes_thirds,sizes,spacing,spacing_responsive,orientation_vertical,options,loop_carousel}.rs`
  stay on the compact `Carousel::new(items)` builder lane, while `parts.rs` remains the explicit
  upstream-shaped adapter seam.
  A second tightening pass now moved the ordinary diagnostics snippets
  `api.rs`,
  `demo.rs`,
  `duration_embla.rs`,
  `expandable.rs`,
  `focus_watch.rs`,
  `loop_downgrade_cannot_loop.rs`,
  `plugin_autoplay.rs`,
  `plugin_autoplay_controlled.rs`,
  `plugin_autoplay_delays.rs`,
  `plugin_autoplay_stop_on_focus.rs`,
  `plugin_autoplay_stop_on_last_snap.rs`,
  and `plugin_wheel_gestures.rs`
  onto that same compact builder lane too.
  The focused gallery source gate now locks `Carousel` parts usage down to the explicit
  upstream-shaped lane plus the custom-control diagnostics snippets `events.rs` and `rtl.rs`.
- the selected first-party shadcn family lanes now also have a crate-root/facade export guard:
  `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs::authoring_critical_family_exports_live_on_curated_facade_only`
  now locks `Select`, `Combobox`, `ComboboxChips`, `Command`, `NavigationMenu`, and `Pagination`
  so newly added builder steps or upstream-shaped parts do not become root-only exports and drift
  away from the curated `shadcn::...` teaching surface.
- there is no remaining ambiguous family on the current shadcn focus lane; any new family or major
  extension should declare its lane before API expansion resumes.
- the default-app UI Gallery page/snippet surface is now effectively closed on the target posture:
  `apps/fret-ui-gallery/src/ui/{pages,snippets}` no longer contains `DocSection::new(...)`,
  there is no remaining default snippet `render(cx: &mut UiCx<'_>) -> AnyElement`,
  and the former scroll-area diagnostics exceptions now live on the dedicated
  `apps/fret-ui-gallery/src/ui/diagnostics/scroll_area/{drag_baseline,expand_at_bottom}.rs` lane.
- the source-policy closure is now lane-level rather than family-level:
  `ui_authoring_surface_default_app::{copyable_ui_gallery_snippet_lane_has_no_top_level_raw_render_roots,ui_gallery_diagnostics_raw_render_roots_are_explicitly_documented}`
  keeps the entire copyable snippet tree off raw landed roots and requires every diagnostics raw
  root to carry an explicit rationale comment.
- the internal preview page surface is now aligned on the same typed section posture too:
  `apps/fret-ui-gallery/src/ui/previews/pages/**` no longer contains `DocSection::new(...)`, and
  the focused `ui_authoring_surface_internal_previews` gate now locks the selected harness/torture
  pages on `DocSection::build(cx, ...)`.
- the remaining internal preview raw-looking signatures are now classified too:
  `apps/fret-ui-gallery/src/ui/previews/**::preview_*` intentionally remains
  `UiCx -> Vec<AnyElement>` as the preview-registry seam, while page-local helpers and preview-page
  wrappers stay on typed `UiChild` surfaces.
- the remaining UI Gallery closeout is now inventory work rather than migration work:
  keep the audited intentional seams explicit in tests/docs and do not reopen broad surface edits
  unless one of these boundaries changes:
  `apps/fret-ui-gallery/src/ui/doc_layout.rs` scaffold vectors,
  `apps/fret-ui-gallery/src/ui/previews/**::preview_*` registry vectors,
  `apps/fret-ui-gallery/src/ui/previews/gallery/overlays/overlay.rs` /
  `overlay/flags.rs` diagnostics result vectors,
  and the two `apps/fret-ui-gallery/src/ui/diagnostics/scroll_area/*` diagnostics raw roots.

Closeout note on 2026-03-15:

- this file is no longer the tracker for a broad migration sweep,
- the remaining work here is maintenance only: explicit raw-seam inventory, source-policy gates,
  and target-state/docs closeout,
- the current small first-party cleanup batch is "single-child wrapper forwarders":
  if a render root or wrapper closure only forwards one already-typed child, prefer
  `ui::single(cx, child)` over `ui::children![cx; child]`,
- that follow-up is now landed on the shared cookbook scaffold, the generated `todo` template
  shell, and the advanced `utility_window_materials_windows` example,
- the low-level interop/app-element-context surfaces
  (`external_texture_imports*`, `external_video_imports*`, `chart_declarative_demo`,
  `node_graph_demo`) are now explicitly classified as a direct-root leaf lane; future work there
  should only add wrapper helpers when surrounding chrome grows, not normalize the leaf contract
  itself onto the wrong lane,
- if a future change reopens conversion-surface design, document the new boundary first instead of
  treating this TODO as a standing invitation to widen APIs again.
- clarified default-app helper posture:
  named app helpers no longer imply `UiCx -> Ui` by default;
  pure page-shell helpers should stay on `fn helper(...) -> impl UiChild` and let `render(...)`
  own the final late landing, while helpers that actually touch runtime/context state may still
  take `&mut UiCx<'_>`.

## M0 — Lock the target vocabulary

- [x] Finalize `TARGET_INTERFACE_STATE.md` as the single source of truth for conversion vocabulary.
- [x] Decide the final public name for the unified component conversion trait.
- [x] Explicitly classify each current conversion name as:
  - [x] kept publicly,
  - [x] kept internally only,
  - [x] moved to advanced/raw only,
  - [x] deleted entirely.
- [x] Confirm that `Ui = Elements` and app-facing `UiChild` are retained.

## M1 — Introduce one public conversion contract

- [x] Add one unified public conversion trait in `fret-ui-kit`.
- [x] Ensure the trait is generic over `H: UiHost` at the trait level rather than splitting host
  agnostic and host-bound concepts publicly.
- [x] Provide temporary internal adapters so current implementations can migrate incrementally.
- [x] Keep `.into_element(cx)` method syntax working on both ordinary values and host-bound
  builder values.

Implementation note after the first landing:

- `IntoUiElement<H>` is now the curated public conversion name on `fret-ui-kit` / `fret`
  component-facing surfaces.
- already-landed raw values (`AnyElement`) now feed that public surface directly via
  `IntoUiElement<H>`; there is no remaining `UiIntoElement` bridge in production code.
- `UiBuilderHostBoundIntoElementExt` has now been deleted from the codebase; `UiBuilder<T>`
  lands through `IntoUiElement<H>` directly.
- the legacy `UiIntoElement` name is now deleted from code; `fret_ui_kit::ui_builder` keeps only
  the public `IntoUiElement<H>` contract plus a direct raw `AnyElement` implementation.

Validation note on 2026-03-12:

- verified the landing with
  `cargo test -p fret-ui-kit --lib --no-run`,
  `cargo test -p fret --lib --no-run`,
  `cargo test -p fret-examples --lib --no-run`,
  and `cargo check -p fretboard`.

## M2 — Rewire builders and child pipelines

- [x] Migrate `UiBuilder<T>` landing paths to the unified public conversion contract without
  relying on the hidden bridge import.
- [x] Migrate `ui::children!` to consume the unified contract.
- [x] Migrate heterogeneous child builders (`FlexBox`, `ContainerBox`, `StackBox`, and related
  host-bound builders) to the unified contract.
- [x] Keep any extra bridging traits private or advanced-only if Rust still needs them
  internally.

Implementation note on 2026-03-12:

- `fret-ui-kit::imui::UiWriterUiKitExt::add_ui(...)` now lands directly through
  `IntoUiElement<H>`; there is no separate immediate-mode child conversion bridge anymore.
- `ui::children!`, `UiElementSinkExt`, and the `fret-ui-kit::ui` child-collection helpers now use
  `IntoUiElement<H>` directly, so the old child-pipeline bridge trait is gone from code.
- host-bound builders in `fret-ui-kit::ui` now implement `IntoUiElement<H>` directly, and
  `UiBuilder<T>::into_element(cx)` resolves through the unified contract.
- `UiHostBoundIntoElement<H>` has now also been deleted from `fret-ui-kit`; there is no remaining
  public host-bound compatibility alias in code.
- `fret-ui-kit` no longer re-exports `UiIntoElement` or `UiChildIntoElement` from the crate root.
- `fret::UiChild` now lands directly through `IntoUiElement<App>` rather than the child-pipeline
  bridge trait.
- first-party `fret-ui-shadcn` overlay/single-child builders and `fret-router-ui` route outlet
  helpers now also accept `IntoUiElement<H>` directly instead of spelling `UiChildIntoElement<H>`.

Validation note on 2026-03-12:

- verified with
  `cargo test -p fret-ui-shadcn --lib --no-run --message-format=short`,
  `cargo test -p fret-ui-kit --lib --no-run --message-format=short`,
  `cargo test -p fret --lib --no-run --message-format=short`,
  `cargo test -p fret-examples --lib --no-run --message-format=short`,
  `cargo check -p fretboard --message-format=short`,
  `cargo test -p fret --lib --message-format=short`,
  `cargo test -p fretboard --message-format=short`,
  `cargo test -p fret-ui-shadcn --lib dropdown_menu_trigger_build_push_ui_accepts_late_landed_child --message-format=short`,
  and `cargo test -p fret-ui-shadcn --lib popover_build_opens_on_trigger_activate_with_late_landed_parts --message-format=short`.

## M3 — Migrate first-party surfaces

- [x] Migrate `ecosystem/fret` curated app/component re-exports to the new vocabulary.
- [x] Migrate `ecosystem/fret-ui-kit` curated docs and examples.
- [x] Migrate `ecosystem/fret-ui-shadcn` reusable helper surfaces where raw `AnyElement` is not
  conceptually required.
- [x] Keep the canonical authoring compare set aligned on the target vocabulary:
  - [x] `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
  - [x] `apps/fret-examples/src/todo_demo.rs`
  - [x] `apps/fretboard/src/scaffold/templates.rs`
- [x] Migrate official cookbook examples toward `Ui` / `UiChild`.
- [x] Migrate selected `apps/fret-examples` helper surfaces that are still on raw child returns.
- [x] Migrate UI Gallery in two lanes:
  - [x] app-facing teaching snippets toward `UiChild`,
  - [x] generic reusable snippets toward the unified component conversion trait,
  - [x] leave justified diagnostics/harness/raw helpers on `AnyElement`.
- [x] Record the currently accepted first-party family taxonomy in the workstream docs:
  - [x] compose-root default lane,
  - [x] dual-lane family,
  - [x] direct recipe root/bridge.
- [x] Classify the remaining ambiguous first-party families before widening APIs again:
  - [x] `Select`
  - [x] `Combobox`
  - [x] `NavigationMenu`
  - [x] `Command`
  - [x] `Pagination`
- [x] Confirm that the current shadcn focus lane does not need a fourth family lane.

Validation snapshot on 2026-03-13:

- `CARGO_TARGET_DIR=target/codex-assets-reload cargo test -p fret-cookbook --lib`
- `CARGO_TARGET_DIR=target/codex-fret-examples cargo test -p fret-examples --lib`
- `CARGO_TARGET_DIR=target/codex-fretboard cargo test -p fretboard scaffold::templates::tests::todo_template_uses_default_authoring_dialect -- --exact`
- `CARGO_TARGET_DIR=target/codex-fretboard cargo test -p fretboard scaffold::templates::tests::simple_todo_template_has_low_adapter_noise_and_no_query_selector -- --exact`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app accordion_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app tabs_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app toggle_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app radio_group_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app slider_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app native_select_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app resizable_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app navigation_menu_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app scroll_area_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app progress_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app chart_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app combobox_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app carousel_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app item_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app table_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app remaining_app_facing_tail -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app badge_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app aspect_ratio_ -- --nocapture`

Validation addendum on 2026-03-14:

- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app context_menu_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app dropdown_menu_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app menubar_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app popover_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app hover_card_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app tooltip_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app button_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app button_group_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app input_group_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app toggle_group_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app switch_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app checkbox_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app separator_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app input_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app field_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app textarea_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app input_otp_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app select_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app calendar_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app alert_dialog_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app dialog_snippets_prefer_ui_cx_on_the_default_app_surface -- --exact --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app dialog_page_uses_typed_doc_sections_for_app_facing_snippets -- --exact --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app drawer_ -- --nocapture`
- `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app sheet_ -- --nocapture`

Implementation note on 2026-03-13:

- the canonical compare set now shares the same posture:
  app-facing imports via `fret::app::prelude::*`,
  `App` / `WindowId`,
  extracted helpers returning `impl UiChild`,
  with page-shell helpers dropping local `cx` when the body can stay fully late-landed and the
  remaining explicit `.into_element(cx)` step living only at the final render-root `Ui`
  conversion boundary,
  `ui::for_each_keyed(...)` as the default keyed-list helper,
  and `shadcn::card(...)` / `card_header(...)` / `card_content(...)` as the default card
  teaching family instead of `Card::build(...)`.
- `apps/fret-examples/src/todo_demo.rs::todo_page(...)` now stays on `impl UiChild`, drops the
  helper-local `cx`, and lets the page shell accept the card value without an intermediate
  `.into_element(cx)` landing seam.
- `apps/fret-examples/src/simple_todo_demo.rs::todo_row(...)` now stays on
  `impl IntoUiElement<App> + use<>` instead of `AnyElement`, while intentionally keeping
  `ui::for_each_keyed_with_cx(...)` because the keyed row still watches a per-item model inside
  the keyed child scope.
- `apps/fretboard/src/scaffold/templates.rs::{todo_page(...),simple_todo::todo_page(...)}`
  now follows that same rule: page helpers stay on `impl UiChild`, drop helper-local `cx`, and
  let the generated `todo` / `simple-todo` templates land those helpers through
  `ui::single(cx, todo_page(...))` instead of helper-local `.into_element(cx)`.
- `apps/fret-cookbook/src/scaffold.rs::{centered_page,centered_page_background,centered_page_muted}`
  now follows the same input-side rule: the shared cookbook page shell accepts
  `IntoUiElement<H>` directly, keeps `AnyElement` only as the named final page-root landing seam,
  and no longer exposes parallel `*_ui(...)` overloads for `UiBuilder`.
- the remaining advanced IMUI compare lane now also hides its non-raw helper returns behind typed
  signatures:
  `apps/fret-examples/src/imui_editor_proof_demo.rs::{render_editor_name_assist_surface,render_authoring_parity_surface,render_authoring_parity_shared_state,render_authoring_parity_declarative_group,render_authoring_parity_imui_group,render_authoring_parity_imui_host}`
  now expose `IntoUiElement<...>`-based signatures while keeping the internal
  `PropertyGroup::into_element(...)` / `imui_build(...)` landing seams explicit.
- after that cleanup, the current real non-`lib.rs` source scan now leaves only one intentional
  `-> AnyElement` helper on the examples/cookbook lane:
  `apps/fret-cookbook/examples/chart_interactions_basics.rs::chart_canvas(...)`.
- the UI Gallery app-facing snippet lane now also starts landing on typed top-level snippet
  surfaces:
  `apps/fret-ui-gallery/src/ui/snippets/accordion/{basic,borders,card,demo,disabled,extras,multiple,rtl,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/accordion.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the tabs family:
  `apps/fret-ui-gallery/src/ui/snippets/tabs/{demo,disabled,extras,icons,line,list,rtl,usage,vertical,vertical_line}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/tabs.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the toggle family:
  `apps/fret-ui-gallery/src/ui/snippets/toggle/{demo,disabled,label,outline,rtl,size,usage,with_text}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/toggle.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the radio-group family:
  `apps/fret-ui-gallery/src/ui/snippets/radio_group/{choice_card,demo,description,disabled,extras,fieldset,invalid,label,plans,rtl,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/radio_group.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the slider family:
  `apps/fret-ui-gallery/src/ui/snippets/slider/{demo,extras,label,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their local model
  state inside the snippet instead of routing it through the page shell, and
  `apps/fret-ui-gallery/src/ui/pages/slider.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the native-select family:
  `apps/fret-ui-gallery/src/ui/snippets/native_select/{demo,disabled,invalid,label,rtl,usage,with_groups}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep local value/open
  model state inside each snippet instead of routing it through `pages/native_select.rs`, and
  `apps/fret-ui-gallery/src/ui/pages/native_select.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the resizable family:
  `apps/fret-ui-gallery/src/ui/snippets/resizable/{demo,handle,notes,rtl,usage,vertical}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep local fractions
  model state inside each snippet instead of routing it through page/content/runtime-driver relay
  state, and `apps/fret-ui-gallery/src/ui/pages/resizable.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the navigation-menu family:
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/{demo,docs_demo,link_component,rtl,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep the local menu
  value state inside each snippet, and `apps/fret-ui-gallery/src/ui/pages/navigation_menu.rs`
  now routes those previews through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the scroll-area family:
  `apps/fret-ui-gallery/src/ui/snippets/scroll_area/{demo,usage,horizontal,nested_scroll_routing,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, the app-facing examples
  now consistently teach `shadcn::scroll_area(cx, |_cx| [...])`, and
  `apps/fret-ui-gallery/src/ui/pages/scroll_area.rs` routes those previews through
  `DocSection::build(cx, ...)`, while the dedicated diagnostics harnesses now live in
  `apps/fret-ui-gallery/src/ui/diagnostics/scroll_area/{drag_baseline,expand_at_bottom}.rs`,
  carry explicit raw-boundary comments, and are registered through
  `DocSection::build_diagnostics(cx, ...)`; the pair of source gates
  `ui_authoring_surface_default_app::{scroll_area_app_facing_snippet_lane_has_no_raw_boundaries,scroll_area_diagnostics_lane_keeps_intentional_raw_boundaries}`
  now lock the split between copyable snippets and raw diagnostics roots.
- the same UI Gallery top-level snippet cleanup now also covers the progress family:
  `apps/fret-ui-gallery/src/ui/snippets/progress/{demo,usage,label,controlled,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/progress.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the chart family:
  `apps/fret-ui-gallery/src/ui/snippets/chart/{demo,usage,contracts,tooltip,legend,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/chart.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the combobox family:
  `apps/fret-ui-gallery/src/ui/snippets/combobox/{conformance_demo,basic,usage,label,auto_highlight,clear_button,groups,groups_with_separator,trigger_button,multiple_selection,custom_items,long_list,invalid,disabled,input_group,rtl}.rs`
  now expose typed app-facing `render(...)` signatures, and
  `apps/fret-ui-gallery/src/ui/pages/combobox.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the carousel family:
  `apps/fret-ui-gallery/src/ui/snippets/carousel/{demo,usage,parts,basic,sizes_thirds,sizes,spacing,spacing_responsive,orientation_vertical,options,api,events,plugin_autoplay,plugin_autoplay_controlled,plugin_autoplay_stop_on_focus,plugin_autoplay_stop_on_last_snap,plugin_autoplay_delays,plugin_wheel_gestures,rtl,loop_carousel,loop_downgrade_cannot_loop,focus_watch,duration_embla,expandable}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/carousel.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the item family:
  `apps/fret-ui-gallery/src/ui/snippets/item/{demo,usage,variants,size,icon,avatar,image,group,header,link,dropdown,extras_rtl,gallery,link_render}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/item.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same UI Gallery top-level snippet cleanup now also covers the table family:
  `apps/fret-ui-gallery/src/ui/snippets/table/{demo,usage,footer,actions,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/table.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the remaining curated default-app tail snippets are now also closed on the same typed lane:
  `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/form/notes.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/sidebar/rtl.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`; the corresponding page
  sections on `pages/breadcrumb.rs`, `pages/date_picker.rs`, `pages/form.rs`, and
  `pages/sidebar.rs` now keep those snippet-owned app-facing sections on `DocSection::build(cx, ...)`.
- the first full legacy page-family sweep is now open on the `badge` lane:
  `apps/fret-ui-gallery/src/ui/snippets/badge/{demo,usage,spinner,rtl,counts,colors,link,icon,variants}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/badge.rs` now routes those previews through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `aspect_ratio`:
  `apps/fret-ui-gallery/src/ui/snippets/aspect_ratio/{demo,usage,portrait,square,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`;
  `pages/aspect_ratio.rs` already stays on `DocSection::build(cx, ...)`, while the
  asset-backed gallery preview path remains intentionally on `render_preview(...)` and the
  copyable top-level `render(...)` functions are explicitly retained as the code-surface seam.
- the same first full legacy page-family sweep now also covers `context_menu`:
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/{demo,basic,usage,submenu,shortcuts,groups,icons,checkboxes,radio,destructive,sides,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, and
  `apps/fret-ui-gallery/src/ui/pages/context_menu.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `dropdown_menu`:
  `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/{avatar,basic,checkboxes,checkboxes_icons,complex,demo,destructive,icons,parts,radio_group,radio_icons,rtl,shortcuts,submenu,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their menu-local
  checkbox/radio/demo state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/dropdown_menu.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `menubar`:
  `apps/fret-ui-gallery/src/ui/snippets/menubar/{checkbox,demo,parts,radio,rtl,submenu,usage,with_icons}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their menubar-local
  checkbox/radio/demo state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/menubar.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `popover`:
  `apps/fret-ui-gallery/src/ui/snippets/popover/{align,basic,demo,rtl,usage,with_form}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their popover-local
  form state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/popover.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `hover_card`:
  `apps/fret-ui-gallery/src/ui/snippets/hover_card/{basic,demo,positioning,rtl,sides,trigger_delays,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their demo assets
  and timing/placement state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/hover_card.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `tooltip`:
  `apps/fret-ui-gallery/src/ui/snippets/tooltip/{demo,disabled_button,keyboard_focus,keyboard_shortcut,long_content,rtl,sides,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their tooltip-local
  provider/content composition beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/tooltip.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `button`:
  `apps/fret-ui-gallery/src/ui/snippets/button/{demo,usage,size,default,outline,secondary,ghost,destructive,link,icon,with_icon,rounded,loading,button_group,link_render,rtl,variants}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/button.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `button_group`:
  `apps/fret-ui-gallery/src/ui/snippets/button_group/{accessibility,button_group_select,demo,dropdown_menu,flex_1_items,input,input_group,nested,orientation,popover,rtl,separator,size,split,text,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their local menu /
  select / input / tooltip state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/button_group.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `input_group`:
  `apps/fret-ui-gallery/src/ui/snippets/input_group/{align_block_end,align_block_start,align_inline_end,align_inline_start,button,button_group,custom_input,demo,dropdown,icon,kbd,label,rtl,spinner,text,textarea,tooltip}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their local input /
  dropdown / tooltip state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/input_group.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `toggle_group`:
  `apps/fret-ui-gallery/src/ui/snippets/toggle_group/{custom,demo,disabled,flex_1_items,full_width_items,label,large,outline,rtl,single,size,small,spacing,usage,vertical}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep reusable item
  helpers generic while removing eager top-level landing, and
  `apps/fret-ui-gallery/src/ui/pages/toggle_group.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `switch`:
  `apps/fret-ui-gallery/src/ui/snippets/switch/{airplane_mode,bluetooth,choice_card,description,disabled,invalid,label,rtl,sizes,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep their local checked
  state beside the snippet itself, and
  `apps/fret-ui-gallery/src/ui/pages/switch.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `checkbox`:
  `apps/fret-ui-gallery/src/ui/snippets/checkbox/{basic,checked_state,demo,description,disabled,group,invalid_state,label,rtl,table,usage,with_title}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep table/group helper
  seams typed, and
  `apps/fret-ui-gallery/src/ui/pages/checkbox.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `separator`:
  `apps/fret-ui-gallery/src/ui/snippets/separator/{demo,list,menu,rtl,usage,vertical}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, keep local row/section
  helpers on `IntoUiElement<H>`, and
  `apps/fret-ui-gallery/src/ui/pages/separator.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `input`:
  `apps/fret-ui-gallery/src/ui/snippets/input/{badge,basic,button_group,disabled,field,field_group,file,form,grid,inline,input_group,invalid,label,required,rtl}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/input.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `field`:
  `apps/fret-ui-gallery/src/ui/snippets/field/{checkbox,choice_card,field_group,fieldset,input,radio,responsive,rtl,select,slider,switch,textarea,validation_and_errors}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/field.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `textarea`:
  `apps/fret-ui-gallery/src/ui/snippets/textarea/{button,demo,disabled,field,invalid,label,rtl,usage,with_text}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/textarea.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `input_otp`:
  `apps/fret-ui-gallery/src/ui/snippets/input_otp/{alphanumeric,controlled,demo,disabled,form,four_digits,invalid,pattern,rtl,separator,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/input_otp.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `select`:
  `apps/fret-ui-gallery/src/ui/snippets/select/{align_item_with_trigger,demo,diag_surface,disabled,groups,invalid,label,rtl,scrollable}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/select.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `calendar`:
  `apps/fret-ui-gallery/src/ui/snippets/calendar/{basic,booked_dates,custom_cell_size,date_and_time_picker,date_of_birth_picker,demo,hijri,locale,month_year_selector,natural_language_picker,presets,range,responsive_mixed_semantics,rtl,usage,week_numbers}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/calendar.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `alert_dialog`:
  `apps/fret-ui-gallery/src/ui/snippets/alert_dialog/{basic,demo,destructive,detached_trigger,media,parts,rich_content,rtl,small,small_with_media,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/alert_dialog.rs` now routes those snippet-backed sections
  through `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `dialog`:
  `apps/fret-ui-gallery/src/ui/snippets/dialog/{custom_close_button,demo,no_close_button,parts,rtl,scrollable_content,sticky_footer,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/dialog.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `drawer`:
  `apps/fret-ui-gallery/src/ui/snippets/drawer/{demo,responsive_dialog,rtl,scrollable_content,sides,snap_points,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/drawer.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `sheet`:
  `apps/fret-ui-gallery/src/ui/snippets/sheet/{demo,no_close_button,parts,rtl,side,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/sheet.rs` now routes those snippet-backed sections through
  `DocSection::build(cx, ...)` instead of `DocSection::new(...)`.
- the same first full legacy page-family sweep now also covers `spinner`, `form`, `empty`,
  `breadcrumb`, and `collapsible`:
  `apps/fret-ui-gallery/src/ui/snippets/spinner/{badges,buttons,customization,demo,empty,extras,input_group,rtl,sizes,usage}.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/form/{controls,demo,fieldset,input,rtl,textarea,upstream_demo,usage}.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/empty/{avatar,avatar_group,background,demo,input_group,outline,rtl,usage}.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/{basic,collapsed,custom_separator,demo,dropdown,link_component,rtl,usage}.rs`,
  and
  `apps/fret-ui-gallery/src/ui/snippets/collapsible/{basic,controlled_state,demo,file_tree,rtl,settings_panel,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while the corresponding
  pages now route those snippet-backed sections through `DocSection::build(cx, ...)` instead of
  `DocSection::new(...)`.
- the focused fourth-batch verification ran with:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app spinner_ -- --nocapture`,
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app form_ -- --nocapture`,
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app empty_ -- --nocapture`,
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app breadcrumb_ -- --nocapture`,
  and
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app collapsible_ -- --nocapture`.
- the fifth high-yield default-app family batch is now landed for `skeleton`, `pagination`,
  `alert`, `sidebar`, and `label`:
  `apps/fret-ui-gallery/src/ui/snippets/skeleton/{avatar,card,demo,form,rtl,table,text,usage}.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/pagination/{demo,extras,icons_only,rtl,simple,usage}.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/alert/{action,basic,custom_colors,demo,destructive,interactive_links,rich_title,rtl}.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/sidebar/{controlled,demo,mobile,usage,use_sidebar}.rs`,
  and
  `apps/fret-ui-gallery/src/ui/snippets/label/{demo,label_in_field,rtl,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while the corresponding
  pages now route those snippet-backed sections through `DocSection::build(cx, ...)` instead of
  `DocSection::new(...)`.
- the focused fifth-batch verification ran with:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app skeleton_ -- --nocapture`,
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app pagination_ -- --nocapture`,
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app alert_ -- --nocapture`,
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app sidebar_ -- --nocapture`,
  and
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app label_ -- --nocapture`.
- the sixth default-app family batch is now landed for `kbd`, `icons`, and `sonner`:
  `apps/fret-ui-gallery/src/ui/snippets/kbd/{button,demo,group,input_group,rtl,tooltip}.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/icons/{grid,spinner}.rs`,
  and
  `apps/fret-ui-gallery/src/ui/snippets/sonner/{demo,extras,notes,position,setup,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while the corresponding
  pages now route those snippet-backed sections through `DocSection::build(cx, ...)` instead of
  `DocSection::new(...)`; `pages/sonner.rs` also now mounts a snippet-local toaster lane and keeps
  the last-action / toaster-position state inside the snippet module instead of relaying page
  models through `ui/content.rs`.
- the focused sixth-batch verification ran with:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app kbd_ -- --nocapture`,
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app icons_ -- --nocapture`,
  and
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app sonner_ -- --nocapture`.
- the next state-heavy default-app family is now also landed for `date_picker`:
  `apps/fret-ui-gallery/src/ui/snippets/date_picker/{basic,demo,dob,dropdowns,input,label,natural_language,notes,presets,range,rtl,time_picker,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/date_picker.rs` now only assembles doc sections and no longer
  relays per-demo `open/month/selected/value` models from the page shell.
- the focused `date_picker` verification ran with:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app date_picker_ -- --nocapture`.
- the self-contained image/default-app family is now also landed for `avatar`:
  `apps/fret-ui-gallery/src/ui/snippets/avatar/{badge_icon,basic,demo,dropdown,fallback_only,group,group_count,group_count_icon,rtl,sizes,usage,with_badge}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/avatar.rs` now only assembles doc sections and no longer
  relays a page-owned `Model<Option<ImageId>>`; the gallery avatar demos now generate a
  self-contained `ImageSource::rgba8(...) -> ImageId` inside the snippet module instead.
- the focused `avatar` verification ran with:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app avatar_ -- --nocapture`.
- the next state-heavy default-app family is now also landed for `command`:
  `apps/fret-ui-gallery/src/ui/snippets/command/{action_first_view,basic,docs_demo,groups,loading,rtl,scrollable,shortcuts,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/command.rs` now only assembles doc sections and no longer
  relays a page-owned `last_action` model through the docs shell; the shared command local-state
  and action helpers now live in `snippets/command/mod.rs`.
- the focused `command` verification ran with:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app command_ -- --nocapture`.
- the next self-contained docs-page family is now also landed for `card`:
  `apps/fret-ui-gallery/src/ui/snippets/card/{card_content,compositions,demo,image,meeting_notes,rtl,size,usage}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/card.rs` now routes those sections through
  `DocSection::build(cx, ...)` and no longer relays a page-owned `event_cover_image` model; the
  image example now resolves its demo `ImageSource` entirely inside the snippet.
- the focused `card` verification ran with:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app card_ -- --nocapture`.
- the small media-docs cleanup is now also landed for `image_object_fit`:
  `apps/fret-ui-gallery/src/ui/snippets/image_object_fit/{mapping,sampling}.rs`
  now expose `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`, while
  `apps/fret-ui-gallery/src/ui/pages/image_object_fit.rs` now routes those sections through
  `DocSection::build(cx, ...)` and no longer relays gallery-owned `ImageId` models; the snippet
  module now generates its own fit/sampling demo `ImageSource`s.
- the focused `image_object_fit` verification ran with:
  `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app image_object_fit_ -- --nocapture`.
- after `accordion` / `tabs` / `toggle` / `radio_group` / `slider` / `native_select` /
  `resizable` / `navigation_menu` / `scroll_area` / `progress` / `chart` / `combobox` /
  `carousel` / `item` / `table` plus the remaining curated tail snippets and the recent
  `button` / `button_group` / `input_group` / `toggle_group` / `switch` / `checkbox` /
  `separator` / `input` / `field` / `textarea` / `input_otp` / `select` / `calendar` /
  `alert_dialog` / `dialog` / `drawer` / `sheet` / `spinner` / `form` / `empty` /
  `breadcrumb` / `collapsible` / `skeleton` / `pagination` / `alert` / `sidebar` / `label` /
  `kbd` / `icons` / `sonner` / `date_picker` / `avatar` / `command` / `card` /
  `image_object_fit` full-family sweeps, the tracked default-app workstream-local backlog now falls
  from 66 to 9 top-level snippet renders still teaching
  `ElementContext<'_, H> -> AnyElement` on that lane (down from 95 before the recent
  high-yield batches, 136 before the broader family sweeps, and 184 before the default-app
  migration run started).
- for the default-app lane specifically, the next queue should now continue on the remaining
  long-tail stateful families after `command` / `card` / `image_object_fit`, with `data_table`
  and `motion_presets` now carrying most of the remaining tracked backlog; `ai` continues on its
  remaining specialized alignment lane now that `material3`, `typography`, and
  `shadcn_extras` have been aligned.
- `apps/fret-cookbook/examples/customv1_basics.rs` now keeps both advanced reusable helpers
  `panel_shell(...)` and `preview_content(...)` on `IntoUiElement<KernelApp>`-based signatures
  instead of returning raw `AnyElement` for non-raw composition.
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs::{render_image_panel,render_svg_panel}`
  now also keep their advanced helper surfaces on `IntoUiElement<KernelApp>`-based signatures
  instead of returning raw `AnyElement` for ordinary card/panel composition.
- `fret-ui-shadcn` `ui_ext/support.rs` and `ui_ext/data.rs` now implement
  `IntoUiElement<H>` directly, so shadcn reusable glue no longer spells
  `UiIntoElement` on those adapters.
- `fret-ui-shadcn` `ui_builder_ext/*` reusable helper closures now accept values that land through
  `IntoUiElement<H>` instead of requiring `AnyElement`-typed closure returns up front.
- `fret-ui-shadcn` internal menu-slot wrappers in
  `context_menu.rs`, `dropdown_menu.rs`, and `menubar.rs`
  now accept `IntoUiElement<H>` inputs for `menu_icon_slot(...)` instead of forcing pre-landed
  `AnyElement`, while keeping the wrapper's own output as an explicit landed menu row slot.
- `fret-ui-shadcn` now has a thin public constructor/wrapper trial:
  `badge.rs::badge<H, T>(...)`, `kbd.rs::kbd<H, T>(...)`,
  `separator.rs::separator<H>()`, `input_group.rs::input_group<H>(...)`,
  `input_otp.rs::input_otp<H>(...)`, `command.rs::command<H, I, F, T>(...)`,
  `checkbox.rs::{checkbox<H>(...), checkbox_opt<H>(...)}`,
  `input.rs::input(...)`,
  `textarea.rs::textarea(...)`,
  `slider.rs::slider(...)`,
  `progress.rs::progress<H>(...)`,
  `switch.rs::{switch<H>(...), switch_opt<H>(...)}`, and
  `toggle.rs::{toggle(...), toggle_uncontrolled(...)}`,
  now keep the helper itself on a typed surface as well, with toggle preserving the concrete
  `Toggle` builder while also accepting typed child inputs before the internal child-list seam.
  `tabs.rs::{tabs(...), tabs_uncontrolled(...)}` now also preserves the concrete `Tabs` builder
  instead of eagerly landing the root helper. Meanwhile,
  `accordion.rs::{accordion_single(...), accordion_single_uncontrolled(...), accordion_multiple(...), accordion_multiple_uncontrolled(...)}`
  now likewise preserves the concrete `Accordion` builder so root-level
  `.collapsible(...)` / `.orientation(...)` / `.loop_navigation(...)` steps stay available before
  the explicit landing seam. Meanwhile,
  `toggle_group.rs::{toggle_group_single(...), toggle_group_single_uncontrolled(...), toggle_group_multiple(...), toggle_group_multiple_uncontrolled(...)}`
  now likewise preserves the concrete `ToggleGroup` builder so root-level
  `.variant(...)` / `.size(...)` / `.orientation(...)` / `.roving_focus(...)` steps stay available
  before the explicit landing seam. Meanwhile,
  `resizable.rs::resizable_panel_group(...)` now likewise preserves the concrete
  `ResizablePanelGroup` builder so root-level `.axis(...)` / `.style(...)` /
  `.test_id_prefix(...)` steps stay available before the explicit landing seam. Meanwhile,
  `navigation_menu.rs::{navigation_menu(...), navigation_menu_uncontrolled(...)}` now likewise
  preserves the concrete `NavigationMenu` builder so root-level `.viewport(...)` /
  `.indicator(...)` / `.md_breakpoint_query(...)` / `.delay_ms(...)` steps stay available before
  the explicit landing seam. Meanwhile,
  `avatar.rs::avatar_sized(...)`, `item.rs::{item_sized(...), item_group(...)}`,
  `scroll_area.rs::scroll_area(...)`, and `native_select.rs::native_select(...)` now likewise
  preserve the concrete `Avatar`, `Item`, `ItemGroup`, `ScrollArea`, and `NativeSelect` builders
  so common size/group/scroll/select configuration can stay fluent before the explicit landing
  seam. Meanwhile, the first-party UI Gallery teaching lane now follows those same
  helpers: `avatar/sizes.rs`, `item/{size,group}.rs`, `scroll_area/{usage,horizontal,nested_scroll_routing}.rs`,
  `native_select/{demo,usage,disabled,invalid,label,with_groups,rtl}.rs`, and the corresponding
  page copy now teach the builder-preserving helper family instead of the older eager `::new(...)`
  or `new_controllable(...)` defaults. The same teaching-lane cleanup now also covers
  `slider/{usage,label,demo}.rs`, `field/slider.rs`, `progress/controlled.rs`,
  `radio_group/{usage,label}.rs`, `form/upstream_demo.rs`, and the `slider` / `radio_group`
  pages so default controlled/uncontrolled examples now teach `slider(model)`,
  `radio_group(model, items)`, and `radio_group_uncontrolled(default, items)` while keeping
  `new_controllable(...)` only on the default-value bridge examples. The same first-party
  teaching sweep now also closes the default root-helper drift on `navigation_menu` /
  `resizable`: `navigation_menu/{usage,demo,docs_demo,link_component,rtl}.rs`,
  `resizable/{usage,demo,vertical,handle,rtl}.rs`, and the corresponding pages now teach
  `navigation_menu(cx, model, |cx| ..)` and `resizable_panel_group(cx, model, |cx| ..)` on the
  default lane instead of leading with raw `::new(...)` root constructors. The same first-party
  teaching sweep now also closes the remaining default-root drift on `tabs` / `toggle` /
  `accordion`: `tabs/{usage,demo,disabled,extras,icons,line,list,rtl,vertical,vertical_line}.rs`,
  `toggle/{usage,demo,outline,with_text,disabled,size,rtl,label}.rs`, and
  `accordion/{demo,basic,multiple,disabled,borders,card,extras,rtl}.rs` now teach
  `tabs_uncontrolled(cx, default, |cx| ..)`,
  `toggle_uncontrolled(cx, default, |cx| ..)` / `toggle(cx, model, |cx| ..)`, and
  `accordion_single_uncontrolled(cx, default, |cx| ..)` /
  `accordion_multiple_uncontrolled(cx, default, |cx| ..)` as the default first-party helper
  family, while `accordion/usage.rs` intentionally keeps the composable `AccordionRoot` surface
  as the explicit advanced seam and the corresponding `tabs` / `toggle` / `accordion` page copy
  now records that boundary directly. Meanwhile,
  `radio_group.rs::{radio_group(...), radio_group_uncontrolled(...)}` now expose typed
  constructor/wrapper outputs, with the radio-group helpers returning the concrete `RadioGroup`
  value so fluent `.a11y_label(...)` / `.disabled(...)` / `.style(...)` steps remain available
  before the explicit landing seam, while
  `card.rs::{card<H, I, F, T>(...), card_sized<H, I, F, T>(...), card_header<H, I, F, T>(...), card_action<H, I, F, T>(...), card_title<H, T>(...), card_description<H, T>(...), card_description_children<H, I, F, T>(...), card_content<H, I, F, T>(...), card_footer<H, I, F, T>(...)}`
  now expose typed constructor/wrapper outputs, while
  `kbd.rs::kbd_icon<H>(...)` remains the explicit raw helper seam because
  `Kbd::from_children(...)` still owns concrete landed-child storage. Combobox anchor overrides now
  reuse `PopoverAnchor::build(...).into_anchor(cx)` instead of a combobox-specific raw alias.
  The card lane now also has builder support where the wrapper still needs late child collection:
  `CardAction::build(...)` and `CardDescription::build(...)`.
- `fret-ui-shadcn/src/typography.rs` now completes the dedicated typography lane:
  `h1` / `h2` / `h3` / `h4` / `p` / `lead` / `large` / `small` / `muted` /
  `inline_code` / `blockquote` / `list` now expose
  `(text_or_items) -> impl IntoUiElement<H> + use<...>` rather than
  `(cx, ...) -> AnyElement`, while intentionally keeping the `shadcn::raw::typography::*`
  namespace stable because typography is still a docs/helper surface rather than a promoted
  registry component contract.
- the typography sweep is now migrated across first-party call sites:
  `ecosystem/fret-genui-shadcn` resolvers, dedicated typography snippets/pages, prose-heavy
  Gallery snippets (`separator` / `sidebar` / `resizable` / `accordion` / `combobox` /
  `calendar` / `date_picker` / `scroll_area` / `sonner` / `slider` / `item` / `ai`), the
  `previews/magic.rs` page, and the affected `apps/fret-examples/*` surfaces now all land
  typography helpers explicitly via `.into_element(cx)` where a concrete `AnyElement` seam is
  still required.
- first-party direct-crate shadcn teaching is now starting to normalize the eager child-list path
  as well: where a component still intentionally owns `new(children: Vec<AnyElement>)`, Gallery
  snippets should prefer `ui::children![cx; ...]` over ad-hoc `vec![...into_element(cx)]`
  assembly so typed helpers can still read like component values at the call site.
- that eager child-list guidance now also covers first-party card wrappers:
  `apps/fret-ui-gallery/src/ui/snippets/card/{usage,demo,rtl}.rs`
  now teach `shadcn::card(|cx| ui::children![cx; ...])` and
  `shadcn::card_header(|cx| ui::children![cx; ...])`
  instead of the old `card(cx, ...)` / `card_header(cx, ...)` eager-landing pattern.
- that eager child-list guidance now also covers the first-party modal/form exemplars:
  selected `dialog`, `sheet`, and `drawer` snippets now build `Field`, `FieldSet`,
  `DialogContent`, `DialogHeader`, `DialogFooter`, `SheetContent`, `SheetHeader`,
  `SheetFooter`, `DrawerContent`, `DrawerHeader`, and `DrawerFooter` children through
  `ui::children![cx; ...]` instead of manually pre-landing every child.
- the same child-list normalization now also covers selected `popover` form snippets:
  `PopoverContent`, `PopoverHeader`, `FieldGroup`, and `Field` now teach
  `ui::children![cx; ...]` rather than manual `AnyElement` assembly on first-party examples.
- helper ergonomics are also tightening around render-local `cx` ownership:
  where a helper can remain fully late-landed, Gallery examples should drop the `cx` parameter
  instead of carrying it only to attach `.into_element(cx)` / `test_id(...)` at helper scope.
- the keyed-list lane now also has a dedicated default helper on `fret-ui-kit`:
  `ui::for_each_keyed(cx, items, |item| key, |item| child)` intentionally serves the
  conditional-list case where a layout closure wants to stay on the ordinary
  `ui::v_flex(|cx| ...)` / `ui::h_flex(|cx| ...)` lane instead of dropping to
  `*_build(|cx, out| ...)` only to spell `for item in ... { out.push_ui(cx, ui::keyed(...)) }`.
  The canonical compare set (`todo_demo`, `simple_todo_v2_target`, scaffold `simple-todo` /
  `todo`) now uses that helper for row lists, and the scaffold README/tests record
  `ui::for_each_keyed(...)` as the default first-party keyed-list teaching surface.
- the keyed-list lane now also has the explicit keyed-scope variant:
  `ui::for_each_keyed_with_cx(cx, items, |item| key, |cx, item| child)` exists for the smaller
  but real class of row builders that need the inner keyed child scope itself (for example
  row-local `cx.text(...)`, local keyed state, or other child-scope work that should happen
  inside the keyed boundary). `apps/fret-examples/src/simple_todo_demo.rs` now uses that helper
  instead of `ui::v_flex_build(...)` + `cx.keyed(...)`, which keeps the keyed-scope escape hatch
  on the ordinary `ui::v_flex(|cx| ..)` lane without introducing a wider `keyed_column(...)`
  abstraction yet.
- the default product docs now teach the same keyed-list lane:
  `docs/first-hour.md`, `docs/authoring-golden-path-v2.md`, and
  `docs/examples/todo-app-golden-path.md` now prefer `ui::for_each_keyed(...)` as the default
  list-identity story, reserve `ui::for_each_keyed_with_cx(...)` for row builders that truly need
  the inner keyed child scope, and frame `*_build(...)` sinks as explicit advanced/manual
  collection seams rather than the first-contact default.
- the query-demo compare lane now also follows the promoted card/default composition posture:
  `apps/fret-examples/src/query_demo.rs` and
  `apps/fret-examples/src/query_async_tokio_demo.rs` now use `shadcn::card(...)` /
  `card_header(...)` / `card_content(...)` plus ordinary `ui::h_row(...)` / `ui::v_flex(...)`
  composition for fixed child lists, while keeping one narrow `ui::v_flex_build(...)` seam only
  where `query_demo` still conditionally appends optional diagnostic text rows.
- the cookbook high-signal app-authoring lane now also follows the promoted card teaching family:
  `apps/fret-cookbook/examples/query_basics.rs`,
  `apps/fret-cookbook/examples/form_basics.rs`,
  `apps/fret-cookbook/examples/async_inbox_basics.rs`, and
  `apps/fret-cookbook/examples/router_basics.rs` now all teach
  `shadcn::card(...)` / `card_header(...)` / `card_content(...)` instead of
  `Card::build(...)` / `CardHeader::build(...)` / `CardContent::build(...)`, while keeping only
  the justified narrow manual seams that are still driven by conditional child emission
  (`query_basics::lines(...)`) or typed router/outlet ownership boundaries (`router_basics`).
  That router lane has now closed its temporary dual surface too:
  `ecosystem/fret-router-ui` no longer exposes `RouterOutletIntoElement` or outlet `*_ui(...)`
  overloads, and `router_basics` now teaches `.into_element_by_leaf(...)` directly while the
  outlet helpers accept `IntoUiElement<App>` outputs on the same named landing seam.
  The same crate now also closes the low-level link-child raw leak:
  `router_link*` helpers no longer publish `IntoIterator<Item = AnyElement>` and instead accept
  iterable `IntoUiElement<App>` children directly while keeping `AnyElement` only as the final
  pressable landing seam.
- the same helper-inventory sweep now also covers `fret-ui-magic`:
  `magic_card`, `border_beam`, `lens`, `marquee`, `dot_pattern`, `grid_pattern`,
  `stripe_pattern`, `dock`, and `sparkles_text` keep `AnyElement` only as their final
  effect/material wrapper seam while accepting iterable `IntoUiElement<H>` children directly.
  The crate-local `collect_children(...)` helper plus source gate now lock that posture so the
  visual helper lane does not regress back to `IntoIterator<Item = AnyElement>` public signatures.
- the same visual-helper inventory now also covers `fret-ui-kit` declarative effect panels:
  `ecosystem/fret-ui-kit/src/declarative/{bloom,pixelate}.rs` now keep `AnyElement` only as the
  final effect-layer wrapper seam, while `bloom_panel(...)` and `pixelate_panel(...)` accept
  iterable `IntoUiElement<H>` children directly. The shared crate-local `collect_children(...)`
  helper plus a focused source-policy gate now lock that posture.
- the same wrapper-helper inventory now also covers the policy wrappers in `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/declarative/{dismissible,visually_hidden}.rs` and
  `ecosystem/fret-ui-kit/src/primitives/focus_scope.rs::{focus_trap,focus_trap_with_id}` now keep
  `AnyElement`/`NodeId` only as the final dismissible-root or focus/a11y wrapper seam, while their
  child closures accept iterable `IntoUiElement<H>` values directly. The same shared
  `collect_children(...)` helper plus the expanded source-policy gate now lock that posture.
- the same wrapper-helper inventory now also covers the scroll / roving policy batch in
  `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/declarative/scroll.rs`,
  `ecosystem/fret-ui-kit/src/primitives/roving_focus_group.rs`,
  `ecosystem/fret-ui-kit/src/primitives/toolbar.rs`, and
  `ecosystem/fret-ui-kit/src/primitives/dismissable_layer.rs` now keep `AnyElement`/`NodeId` only
  as the final scroll, roving-container, toolbar, or dismissable-root seam, while their child
  closures accept iterable `IntoUiElement<H>` values directly. Direct wrappers land through the
  shared `collect_children(...)` helper, and delegate wrappers now forward only to already-typed
  helper seams; the expanded source-policy gate plus `cargo check -p fret-ui-kit --lib` lock that
  posture.
- the same wrapper-helper inventory now also covers the layout/effect query batch in
  `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/declarative/chrome.rs`,
  `ecosystem/fret-ui-kit/src/declarative/glass.rs`, and
  `ecosystem/fret-ui-kit/src/declarative/container_queries.rs` now keep `AnyElement` only as the
  final control-chrome, effect-layer, or layout-query wrapper seam, while their child closures
  accept iterable `IntoUiElement<H>` values directly. The same shared `collect_children(...)`
  helper plus the expanded source-policy gate and `cargo check -p fret-ui-kit --lib` lock that
  posture.
- the same wrapper-helper inventory now also covers the menu/popup skeleton batch in
  `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/primitives/menu/{content,content_panel,sub_content}.rs` and
  `ecosystem/fret-ui-kit/src/primitives/popper_content.rs` now keep `AnyElement` only as the final
  menu semantics/panel or popper wrapper seam, while their child closures accept iterable
  `IntoUiElement<H>` values directly. Direct wrappers land through `collect_children(...)`, and
  delegate wrappers now forward only to already-typed helper seams; the expanded source-policy gate
  plus `cargo check -p fret-ui-kit --lib` lock that posture.
- the same wrapper-helper inventory now also covers the cache/list batch in `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/declarative/cached_subtree.rs` and
  `ecosystem/fret-ui-kit/src/declarative/list.rs` now keep `AnyElement` only as the final
  cache-root or list-row wrapper seam, while their child closures accept iterable
  `IntoUiElement<...>` values directly. `cached_subtree` lands through `collect_children(...)`
  behind the `ViewCache` wrapper, the retained list path lands cached row payloads through the same
  helper, and the expanded source-policy gate plus `cargo check -p fret-ui-kit --lib` lock that
  posture.
- the same wrapper-helper inventory now also covers the tab/toggle/accordion primitives in
  `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/primitives/{tabs,toggle,accordion}.rs` now keep `AnyElement` only as
  the final semantics/pressable/content wrapper seam, while their public child closures accept
  iterable `IntoUiElement<H>` values directly. The tabs/accordion roving-list wrappers, toggle and
  accordion pressable wrappers, and tab/accordion content wrappers now land those typed values
  behind `collect_children(...)`; the expanded source-policy gate plus
  `cargo check -p fret-ui-kit --lib` lock that posture.
- the same wrapper-helper inventory now also covers the dialog/popover/alert-dialog/select/tooltip overlay helpers in
  `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/primitives/{alert_dialog,dialog,popover,select,tooltip}.rs` now accept iterable
  `IntoUiElement<H>` values on wrapper helpers that still have an `ElementContext` available
  (`DialogRoot`/`PopoverRoot` request adapters, barrier helpers, semantics wrappers, and modal
  layer assemblers). Those wrappers land typed values through `collect_children(...)`, while the
  final overlay-request constructors intentionally remain `IntoIterator<Item = AnyElement>` because
  they are the no-`cx` landing seam. The dedicated source-policy test plus
  `cargo check -p fret-ui-kit --lib` lock that posture.
- the same wrapper-helper inventory now also covers the sortable DnD recipe helper in
  `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs` now accepts iterable `IntoUiElement<H>`
  values for row-content closures and lands them through `collect_children(...)` inside the row
  container wrapper. The expanded source-policy gate plus `cargo check -p fret-ui-kit --lib` lock
  that posture.
- the same wrapper-helper inventory now also covers the virtualized table helpers in
  `fret-ui-kit`:
  `ecosystem/fret-ui-kit/src/declarative/table.rs` now accepts iterable `IntoUiElement<H>` values
  for header and cell render closures in `table_virtualized(...)` and
  `table_virtualized_copyable(...)`. The table-owned header container now lands typed header
  values through `collect_children(...)`, and the expanded source-policy gate plus
  `cargo check -p fret-ui-kit --lib` lock that posture.
- the next cookbook app-authoring batch now follows the same surface:
  `apps/fret-cookbook/examples/toggle_basics.rs`,
  `apps/fret-cookbook/examples/payload_actions_basics.rs`,
  `apps/fret-cookbook/examples/hello_counter.rs`,
  `apps/fret-cookbook/examples/text_input_basics.rs`, and
  `apps/fret-cookbook/examples/commands_keymap_basics.rs` now also teach the card wrapper family
  (`card(...)`, `card_header(...)`, `card_content(...)`, and `card_footer(...)` where needed)
  instead of leading with `Card::build(...)` / section builders on the default teaching lane.
- the next cookbook interop/state batch now also follows the same outer-card teaching surface:
  `apps/fret-cookbook/examples/undo_basics.rs`,
  `apps/fret-cookbook/examples/drag_basics.rs`,
  `apps/fret-cookbook/examples/external_texture_import_basics.rs`,
  `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`, and
  `apps/fret-cookbook/examples/date_picker_basics.rs` now all teach
  `shadcn::card(...)` plus the slot helper family for their outer shells instead of
  `Card::build(...)`, while preserving the justified advanced seams that still belong to
  interop/assets internals (`Alert::build(...)`, conditional asset error callouts, and retained
  viewport ownership).
- the next cookbook visual/app-support batch now also follows the same outer-card teaching
  surface:
  `apps/fret-cookbook/examples/customv1_basics.rs`,
  `apps/fret-cookbook/examples/drop_shadow_basics.rs`,
  `apps/fret-cookbook/examples/effects_layer_basics.rs`,
  `apps/fret-cookbook/examples/toast_basics.rs`, and
  `apps/fret-cookbook/examples/markdown_and_code_basics.rs` now teach `shadcn::card(...)` plus
  the slot helper family instead of `Card::build(...)`, with
  `customv1_basics.rs::panel_shell(...)` also moving onto `card(...)` / `card_header(...)` /
  `card_content(...)` so advanced cookbook helper code no longer re-teaches the old outer-shell
  pattern.
- the advanced cookbook bootstrap lane now also matches the real `UiAppBuilder` surface:
  `apps/fret-cookbook/examples/{external_texture_import_basics,embedded_viewport_basics,gizmo_basics,docking_basics,chart_interactions_basics}.rs`
  now explicitly install Lucide icons through `.setup(fret_icons_lucide::app::install)` instead
  of teaching a nonexistent `.with_lucide_icons()` facade, and
  `apps/fret-cookbook/Cargo.toml` now includes `dep:fret-icons-lucide` behind the
  `cookbook-bootstrap` feature so the documented bootstrap path matches the code that actually
  compiles.
- the advanced retained/interop cookbook lane now also converges on the promoted outer-card
  family where the shell is ordinary authoring surface:
  `apps/fret-cookbook/examples/icons_and_assets_basics.rs`,
  `apps/fret-cookbook/examples/embedded_viewport_basics.rs`,
  `apps/fret-cookbook/examples/gizmo_basics.rs`,
  `apps/fret-cookbook/examples/docking_basics.rs`, and
  `apps/fret-cookbook/examples/chart_interactions_basics.rs` now teach `shadcn::card(...)` plus
  the slot helper family for their top-level shells instead of `Card::build(...)`, while
  intentionally keeping raw retained/canvas/viewport seams such as
  `chart_interactions_basics.rs::chart_canvas(...)`, dock-host retained roots, and embedded
  viewport ownership explicit.
- the remaining cookbook card-shell cleanup is now closed on the examples tree:
  `apps/fret-cookbook/examples/theme_switching_basics.rs`,
  `apps/fret-cookbook/examples/data_table_basics.rs`,
  `apps/fret-cookbook/examples/virtual_list_basics.rs`,
  `apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs`,
  `apps/fret-cookbook/examples/overlay_basics.rs`, and
  `apps/fret-cookbook/examples/utility_window_materials_windows.rs` now also teach
  `shadcn::card(...)` plus the slot helper family for their ordinary shells, so
  `apps/fret-cookbook/examples/**` no longer contains first-party
  `Card::build(...)` / `CardHeader::build(...)` / `CardContent::build(...)` teaching on the
  default authoring lane.
- the cookbook source-policy lane now also locks that convergence directly in code:
  `apps/fret-cookbook/src/lib.rs::cookbook_examples_keep_card_wrapper_family_as_the_only_card_teaching_surface`
  now fails if any example reintroduces `shadcn::Card::build(...)`,
  `shadcn::CardHeader::build(...)`, or `shadcn::CardContent::build(...)`, while
  `retained_canvas_helpers_keep_raw_landing_seams` continues to record
  `chart_interactions_basics.rs::chart_canvas(...) -> AnyElement` as the one explicit retained
  bridge seam on that lane.
- the examples helper-return lane is now also tightened on the default-app side:
  `apps/fret-examples/src/{custom_effect_v2_identity_web_demo,custom_effect_v2_web_demo,custom_effect_v2_lut_web_demo,custom_effect_v2_glass_chrome_web_demo}.rs`
  now keep `stage_tile(...)` on `impl IntoUiElement<App> + use<>` and move the explicit
  `.into_element(cx)` seam back to the stage `Vec<AnyElement>` assembly site, while
  `apps/fret-examples/src/async_playground_demo.rs::catalog_item(...)` now likewise returns
  `impl IntoUiElement<KernelApp> + use<>` and lands explicitly at the `out.push(...)` sink.
  `apps/fret-examples/src/lib.rs` now records those helpers in source-policy tests so they do not
  drift back to `AnyElement`-typed authoring helpers.
- `fret-ui-shadcn::prelude::*` now re-exports `IntoUiElement`, so direct-crate first-party
  shadcn examples do not need ad-hoc trait imports just to land typed helpers such as
  `shadcn::raw::typography::*`.
- Verification update on 2026-03-13:
  the remaining thin helper family (`checkbox`, `progress`, `switch`) now also stays on typed
  `IntoUiElement<H>` outputs all the way through the current first-party teaching lane, with
  `fret-ui-shadcn` crate docs calling out the rule explicitly, `fret_ui_shadcn::prelude::*`
  also re-exporting `UiElementTestIdExt` / `UiElementA11yExt` / `UiElementKeyContextExt`, and the
  affected UI Gallery snippets (`tooltip`, `input_group`, `kbd`, `label`, `typography`, selected
  AI attachment hover-card examples) no longer re-teach eager helper landing just to attach
  decoration or diagnostics hooks. Validation for the production surface: `cargo check -p
  fret-ui-shadcn --lib`.
- Verification update on 2026-03-13:
  `radio_group.rs::{radio_group(...), radio_group_uncontrolled(...)}` now also leaves the default
  authoring surface on a typed lane by returning `RadioGroup` instead of eagerly landing
  `AnyElement`, with the crate-level source gate updated to forbid the old `(cx, ...) ->
  AnyElement` helper shape for this wrapper pair as well.
- Verification update on 2026-03-13:
  `input.rs::input(...)` and `textarea.rs::textarea(...)` now also expose the obvious builder
  values (`Input` / `Textarea`) instead of public `(cx, ...) -> AnyElement` render functions, so
  simple text-field authoring can stay on fluent component values before the explicit landing
  seam. The same crate-level thin-helper source gate now forbids the old raw helper signatures.
- Verification update on 2026-03-13:
  `slider.rs::slider(...)` now also returns the concrete `Slider` builder instead of exposing the
  old parameter-heavy `(cx, ...) -> AnyElement` render function on the public surface, keeping
  ordinary slider authoring aligned with the same builder-preserving rule as `input`,
  `textarea`, and `radio_group`.
- Verification update on 2026-03-13:
  `toggle.rs::{toggle(...), toggle_uncontrolled(...)}` now also returns the concrete `Toggle`
  builder instead of eagerly landing the helper output, and the closure inputs now accept typed
  child values (`IntoUiElement<H>`) instead of forcing `AnyElement` before the helper-owned
  internal child-list seam.
- Verification update on 2026-03-13:
  `tabs.rs::{tabs(...), tabs_uncontrolled(...)}` now also returns the concrete `Tabs` builder
  instead of eagerly landing the root helper, aligning the helper posture with the same
  builder-preserving story already used by `slider`, `toggle`, and `radio_group`.
- Verification update on 2026-03-13:
  `accordion.rs::{accordion_single(...), accordion_single_uncontrolled(...), accordion_multiple(...), accordion_multiple_uncontrolled(...)}`
  now also returns the concrete `Accordion` builder instead of eagerly landing the root helper,
  keeping common accordion configuration on the same builder-preserving lane as `tabs` while the
  explicit `.into_element(cx)` seam remains the only place where the root lands.
- Verification update on 2026-03-13:
  `toggle_group.rs::{toggle_group_single(...), toggle_group_single_uncontrolled(...), toggle_group_multiple(...), toggle_group_multiple_uncontrolled(...)}`
  now also returns the concrete `ToggleGroup` builder instead of eagerly landing the root helper,
  keeping grouped toggle configuration on the same builder-preserving lane as `accordion` and
  `tabs` while the explicit `.into_element(cx)` seam remains the only root landing point.
- Verification update on 2026-03-13:
  `resizable.rs::resizable_panel_group(...)` now also returns the concrete
  `ResizablePanelGroup` builder instead of eagerly landing the root helper, so common resizable
  root configuration can stay fluent until the explicit `.into_element(cx)` seam.
- Verification update on 2026-03-13:
  `navigation_menu.rs::{navigation_menu(...), navigation_menu_uncontrolled(...)}` now also
  returns the concrete `NavigationMenu` builder instead of eagerly landing the root helper, so
  common navigation-menu root configuration can stay fluent until the explicit
  `.into_element(cx)` seam.
- Verification update on 2026-03-13:
  `avatar.rs::avatar_sized(...)`, `item.rs::{item_sized(...), item_group(...)}`,
  `scroll_area.rs::scroll_area(...)`, and `native_select.rs::native_select(...)` now also stay on
  the builder-preserving lane instead of eagerly landing the obvious helper surface, so ordinary
  avatar/item/scroll/select authoring can keep fluent root configuration open until the explicit
  `.into_element(cx)` seam.
- first-party docs/tests no longer need to mention the old scaffold name for ordinary authoring:
  `docs/first-hour.md` now teaches `IntoUiElement<H>`, and `fret-ui-ai` builder smoke tests
  (`elements/message.rs`, `elements/workflow/panel.rs`) now assert against the public
  `IntoUiElement<fret_app::App>` contract instead of `UiIntoElement`.
- exported `fret-ui-kit` adapter macros now attach `IntoUiElement<H>` directly, and the legacy
  `UiIntoElement` production bridge is now deleted from `ui_builder.rs`; the old name survives
  only in source-policy assertions and historical workstream docs.
- the RenderOnce helper macro is now also renamed onto the public vocabulary:
  component-authoring docs should teach `fret_ui_kit::ui_component_render_once!(Ty)` rather than
  the old `ui_into_element_render_once!` name.
- declarative semantics decorators now also sit on the public landing trait:
  `UiElementTestIdExt`, `UiElementA11yExt`, and `UiElementKeyContextExt` wrappers land through
  `IntoUiElement<H>` directly, so `declarative/semantics.rs` no longer depends on
  `UiIntoElement` outside tests.
- built-in text primitives now also bypass the hidden bridge:
  `ui::TextBox` and `ui::RawTextBox` implement `IntoUiElement<H>` directly, so the remaining
  `UiIntoElement` production usage is now concentrated in `ui_builder.rs`.
- typography remains decoupled from the model-heavy constructor lane:
  this sweep does not mix with `checkbox`, `progress`, or `switch` refactors.
- selected advanced/manual-assembly examples now also keep reusable helpers off raw landed return
  types by default:
  `apps/fret-examples/src/assets_demo.rs` (`render_image_panel`, `render_svg_panel`),
  `apps/fret-examples/src/async_playground_demo.rs`
  (`header_bar`, `body`, `catalog_panel`, `main_panel`, `inspector_panel`,
  `policy_editor`, `query_panel_for_mode`, `query_inputs_row`, `query_result_view`,
  `status_badge`),
  `apps/fret-examples/src/custom_effect_v1_demo.rs`
  (`stage`, `lens_row`, `lens_shell`, `plain_lens`, `custom_effect_lens`, `inspector`),
  `apps/fret-examples/src/custom_effect_v2_demo.rs`
  (`stage`, `lens_row`, `lens_shell`, `plain_lens`, `custom_effect_lens`, `inspector`),
  `apps/fret-examples/src/custom_effect_v3_demo.rs`
  (`stage`, `stage_controls`, `animated_backdrop`, `lens_row`, `lens_shell`),
  `apps/fret-examples/src/postprocess_theme_demo.rs`
  (`inspector`, `stage`, `stage_body`, `stage_cards`),
  `apps/fret-examples/src/drop_shadow_demo.rs` (`card<H>(...)`),
  `apps/fret-examples/src/markdown_demo.rs` (`render_image_placeholder<H>(...)`),
  `apps/fret-examples/src/liquid_glass_demo.rs` (`lens_panel<H>(...)`),
  `apps/fret-cookbook/examples/customv1_basics.rs`
  (`panel_shell(...)`, `preview_content(...)`),
  `apps/fret-cookbook/examples/drop_shadow_basics.rs` (`shadow_card(...)`),
  and `apps/fret-cookbook/examples/icons_and_assets_basics.rs`
  (`render_image_preview(...)`)
  now return `impl IntoUiElement<...>`.
- the manual/web `custom_effect_v2_*_web_demo` family now also follows the same typed helper-return
  rule for reusable helpers (`lens`, `inspector`, `label_row`, `controls_panel`), but those files
  remain advanced/manual harness evidence rather than default app-lane teaching surfaces.
- `apps/fret-examples/src/hello_world_compare_demo.rs` now keeps its local `swatch(...)` closure
  off raw landed returns by default; it lands explicitly only where the surrounding child array
  wants raw elements.

Update on 2026-03-13:

- the first-party wrapper return-shape rule is now explicit:
  when a helper should preserve fluent builder affordances before the explicit landing seam,
  prefer returning the concrete builder/component type rather than an opaque
  `impl IntoUiElement<H>`.
- this rule is now landed across the `card`, `alert`, `table`, and `field` families:
  `card(...)` / `card_header(...)` / `card_action(...)` / `card_content(...)` / `card_footer(...)`
  return concrete builder or component types, `alert(...)` now returns `AlertBuild<H, ...>`
  while `AlertAction::build(...)` stays on the builder-first typed lane, and the new
  `table(...)` / `table_header(...)` / `table_body(...)` / `table_footer(...)` /
  `table_row(...)` / `table_head(...)` / `table_cell(...)` / `table_caption(...)` helpers now
  cover the first-party table authoring lane without forcing `Table::build(...)` /
  `TableRow::build(...)` boilerplate at the call site, while `field_set(...)` /
  `field_group(...)` now remove the remaining raw/eager wrapper seam around grouped form authoring.
- the same typed-wrapper rule now also covers the `empty` family:
  `empty(...)` / `empty_header(...)` / `empty_media(...)` / `empty_title(...)` /
  `empty_description(...)` / `empty_content(...)` now keep the default empty-state slot surface on
  the typed lane, while `EmptyMedia::variant(...)` remains available on the builder/component
  surface when call sites still need to tweak media chrome before the explicit landing seam.
- page-level/default-teaching cleanup is now an explicit sub-lane of M3:
  once a family is promoted onto a typed wrapper path, first-party Gallery page prose/code blocks
  should stop teaching the old constructor/builder surface as the default authoring path.

Update on 2026-03-13 (page/docs teaching drift cleanup):

- selected AI Gallery doc pages now standardize their small reference tables on
  `doc_layout::text_table(...)` instead of open-coding `Table::build(...)` /
  `TableHeader::build(...)` / `TableRow::build(...)` / `TableCell::build(...)` boilerplate.
- the `field` page usage/code block now teaches `shadcn::field_set(|cx| { ... })` and
  `shadcn::field_group(|cx| { ... })` as the default grouped authoring path while keeping
  `Field::new([...])` as the low-level single-field root.
- lower-traffic first-party snippets are now being swept under the same rule:
  `src/ui/snippets/pagination/extras.rs` teaches the pagination wrapper family instead of
  `Pagination::new(...)` / `PaginationContent::new(...)`, while
  `src/ui/snippets/checkbox/table.rs` and `src/ui/snippets/typography/table.rs` now teach the
  table wrapper family instead of raw `Table::build(...)`.
- the first-party alert teaching lane is now also tightened for already-promoted wrappers:
  `src/ui/snippets/alert/{demo,interactive_links,custom_colors,rich_title}.rs` and
  `src/ui/snippets/motion_presets/fluid_tabs_demo.rs` now teach
  `shadcn::alert(|cx| ui::children![cx; ...])` instead of `Alert::new(...)`, and
  `selected_alert_snippets_prefer_alert_wrapper_family` now locks that source policy.
- the same wrapper-family cleanup now also covers grouped-field control snippets beyond the
  dedicated field/form pages:
  `combobox/label.rs`, `input/{form,field_group}.rs`, `toggle_group/label.rs`,
  `date_picker/{label,time_picker}.rs`, `native_select/label.rs`, `select/label.rs`,
  `select/align_item_with_trigger.rs`, `checkbox/with_title.rs`, `slider/label.rs`,
  `switch/{label,choice_card}.rs`, and `radio_group/{label,fieldset,invalid,extras}.rs` now teach
  `field_group(...)` / `field_set(...)` instead of `FieldGroup::new(...)` / `FieldSet::new(...)`.
- the `card` page notes now also present `card(...)` as the default first-party teaching path,
  with `Card::build(...)` explicitly framed as a lower-level option rather than a default-equal
  recommendation.
- selected first-party card snippets now also follow the wrapper-family lane end-to-end:
  `card/{size,card_content,image,compositions}.rs` now teach `card(...)` plus the slot helper
  family instead of `Card::new(...)` / `CardHeader::new(...)` / `CardContent::new(...)` /
  `CardFooter::new(...)`, while `card/{demo,rtl}.rs` now also teach `card_footer(...)` instead of
  `CardFooter::build(...)`; `compositions.rs::cell(...)` now accepts any `IntoUiElement<App>`
  card input so the example no longer needs a concrete `shadcn::Card` parameter to stay typed.
- the same card-family cleanup now also reaches a few non-card composition snippets that were still
  teaching `Card::*` as the outer container:
  `tabs/demo.rs`, `input_otp/form.rs`, `collapsible/basic.rs`, and
  `motion_presets/fluid_tabs_demo.rs` now teach `card(...)` plus the slot helper family instead of
  `Card::new(...)` / `CardHeader::new(...)` / `CardContent::new(...)` / `CardFooter::new(...)`.
- another low-risk composition sweep now lands on the same lane:
  `calendar/presets.rs`, `motion_presets/{overlay_demo,stagger_demo,stack_shift_list_demo}.rs`,
  and `accordion/card.rs` now also teach `card(...)` plus the slot helper family instead of
  `Card::new(...)` / `CardHeader::new(...)` / `CardContent::new(...)` / `CardFooter::new(...)`
  for their outer showcase shell.
- the card-shell cleanup now also reaches two ordinary non-card demo surfaces:
  `toast/deprecated.rs` now teaches its deprecation card through `card(...)` plus the slot helper
  family, and `chart/demo.rs` now teaches `chart_card(...)` / `trending_footer(...)` on typed
  `IntoUiElement<App>` helpers while using `card(...)` plus the slot helper family for the chart
  shell instead of eager `Card::*` constructors.
- `form/upstream_demo.rs` now joins the same card-shell lane for its lightweight inset rows:
  the `mobile_field` card plus the marketing/security email-notification cards now teach
  `card(...)` + `card_content(...)` instead of raw `Card::new(...)` / `CardContent::new(...)`
  while leaving the broader upstream form/state logic untouched.
- another low-risk sweep now lands on the same card-family teaching rule:
  `motion_presets/{preset_selector,token_snapshot}.rs`, `skeleton/card.rs`,
  `accordion/extras.rs`, `collapsible/settings_panel.rs`, and `card/meeting_notes.rs`
  now teach `card(...)` plus the slot helper family instead of eager `Card::*` constructors,
  with `meeting_notes.rs` also switching its header action to `card_action(...)`.
- the last remaining low-risk non-material/non-carousel card shell in this sweep is now gone too:
  `ai/speech_input_demo.rs` now teaches its transcript surface through
  `card(...)` + `card_content(...)` instead of `Card::new(...)` / `CardContent::new(...)`.
- the remaining ordinary `carousel` teaching drift is now closed too:
  `carousel/{basic,api,demo,duration_embla,events,expandable,focus_watch,loop_carousel,loop_downgrade_cannot_loop,options,orientation_vertical,parts,plugin_autoplay,plugin_autoplay_controlled,plugin_autoplay_delays,plugin_autoplay_stop_on_focus,plugin_autoplay_stop_on_last_snap,plugin_wheel_gestures,rtl,sizes,sizes_thirds,spacing,spacing_responsive,usage}.rs`
  now teach `card(...)` + `card_content(...)` instead of eager `Card::new(...)` /
  `CardContent::new(...)`, and the selected card-wrapper source gate now covers those carousel
  call sites too.
- the follow-up policy decision is now made explicitly:
  `sidebar/*` and `material3/*` should also teach the wrapper family for ordinary showcase shells,
  because those cards are still first-party teaching surfaces rather than low-level recipe
  implementation examples.
- with that rule applied, the remaining `sidebar/{usage,demo,rtl,use_sidebar,mobile,controlled}.rs`
  and `material3/{state_matrix,menu,text_field,autocomplete,touch_targets,snackbar,tooltip,list}.rs`
  card shells now also teach `card(...)` + `card_header(...)` + `card_content(...)` instead of
  eager `Card::*` constructors, and the source-policy gate now locks the stronger invariant that
  `src/ui/snippets/**` no longer reintroduces any `shadcn::Card::*` constructor family at all.
- a follow-up page/docs sweep now confirms the same policy at the page layer too:
  `src/ui/pages/**` no longer reintroduces `shadcn::Card::*` constructors either, while
  `pages/card.rs` intentionally keeps one note that frames `Card::build(...)` as a lower-level
  late-child-collection escape hatch rather than a default teaching path.
- focused source-policy gates now also cover this page-level teaching lane, so already-promoted
  families cannot silently regress back to legacy `FieldSet::new(...)` / `Table::build(...)`
  teaching on first-party doc pages.
- the same typed-wrapper rule now also covers the `pagination` family where the old surface still
  forced eager child landing:
  `pagination(...)` / `pagination_content(...)` / `pagination_item(...)` /
  `pagination_link(...)` now keep pagination root/content/item/link authoring on the typed lane,
  while `PaginationPrevious::new()`, `PaginationNext::new()`, and `PaginationEllipsis::new()`
  remain ordinary typed leaf constructors because they never required a raw child-list seam.
- the crate root/facade now also expose `alert(...)` and the table helper family, and the UI
  Gallery alert/table snippets now teach `shadcn::alert(|cx| ui::children![cx; ...])` and
  `shadcn::table(|cx| ui::children![cx; ...])` as the default first-party authoring pattern, and
  the selected field/form snippets now also teach `shadcn::field_set(|cx| ...)` /
  `shadcn::field_group(|cx| ...)` instead of `FieldSet::build(...)` / `FieldGroup::build(...)`.
- the first-party Empty docs/snippets now follow that same teaching lane:
  `apps/fret-ui-gallery/src/ui/snippets/empty/*` and `src/ui/snippets/spinner/empty.rs`
  now teach `shadcn::empty(|cx| ui::children![cx; ...])` plus the slot helper family rather than
  `Empty::new([...])` / `EmptyHeader::new([...])` / `EmptyContent::new([...])`,
  and the gallery source gate now locks that posture with
  `selected_empty_snippets_prefer_empty_wrapper_family`.
- the first-party Pagination docs/snippets now follow the same teaching lane:
  `apps/fret-ui-gallery/src/ui/snippets/pagination/{demo,icons_only,rtl,simple,usage}.rs`
  now teach `shadcn::pagination(|cx| ui::children![cx; ...])` plus the content/item/link helper
  family rather than `Pagination::new([ PaginationContent::new([ ... ]) ])`, and the gallery
  source gate now locks that posture with
  `selected_pagination_snippets_prefer_pagination_wrapper_family`.
- remaining follow-up after the current pagination landing:
  finish auditing page-level docs/code blocks that still teach old type-first examples for
  already-promoted families beyond the now-updated `field` page and AI page-local table helpers,
  before widening the thin-wrapper trial much further.
- next priority after the `card` + `alert` + `table` + `field` sweep:
  review the remaining lower-frequency wrappers and decide which ones still justify promotion,
  versus leaving them as explicit type-first APIs.
- `apps/fret-cookbook/examples/chart_interactions_basics.rs::chart_canvas(...)` is now treated as
  an intentional raw retained seam rather than migration debt: it owns
  `RetainedSubtreeProps::new::<KernelApp>(...)` and `cached_subtree_with(...)` landing.
- selected WebGPU manual/web harness examples now also keep reusable helpers off raw landed returns
  by default:
  `custom_effect_v2_identity_web_demo`, `custom_effect_v2_web_demo`,
  `custom_effect_v2_lut_web_demo`, and `custom_effect_v2_glass_chrome_web_demo`
  now use `impl IntoUiElement<fret_app::App> + use<>` for non-raw helper composition, with
  explicit `.into_element(cx)` reserved for stage child arrays, overlay child collections, and
  other concrete raw landing seams. This is advanced/manual harness closure, not default
  app-lane proof.
- selected UI Gallery AI doc pages now keep page-local helpers on the default app-facing child
  surface:
  `ai_persona_demo.rs`, `ai_commit_demo.rs`, `ai_context_demo.rs`,
  `ai_model_selector_demo.rs`, `ai_voice_selector_demo.rs`, `ai_mic_selector_demo.rs`,
  `ai_checkpoint_demo.rs`, `ai_shimmer_demo.rs`, `ai_test_results_demo.rs`,
  `ai_artifact_demo.rs`, and `ai_chain_of_thought_demo.rs`
  now return `impl UiChild + use<>` for page-local notes/table helpers, with explicit
  `.into_element(cx)` seams only where `DocSection::new(...)` still intentionally consumes
  `AnyElement`.
- selected UI Gallery badge snippets now also keep their local layout helper off raw landed
  returns by default:
  `src/ui/snippets/badge/{demo,spinner,rtl,counts,colors,icon,variants}.rs`
  now use `fn row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>`,
  while the snippet `render(...) -> AnyElement` entrypoint remains unchanged.
- selected UI Gallery context-menu snippets now also keep local trigger helpers off raw landed
  returns by default:
  `src/ui/snippets/context_menu/{basic,radio,checkboxes,groups,icons,shortcuts,destructive,demo,rtl,submenu}.rs`
  now use `trigger_surface(...) -> impl IntoUiElement<H> + use<...>`,
  with explicit `.into_element(cx)` only at the menu trigger seam.
- selected UI Gallery combobox snippets now also keep local state display helpers off raw landed
  returns by default:
  `src/ui/snippets/combobox/{long_list,input_group,trigger_button,groups_with_separator,groups,disabled,custom_items,clear_button,invalid}.rs`
  now use `state_row(...)` and `state_rows(...) -> impl IntoUiElement<fret_app::App> + use<>`,
  with explicit `.into_element(cx)` only at sibling child-collection and render-boundary seams.
- selected UI Gallery pagination snippets now also keep local page label helpers off raw landed
  returns by default:
  `src/ui/snippets/pagination/{simple,usage}.rs`
  now use `page_number(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at the `PaginationLink::new([..])` seam.
- selected UI Gallery carousel snippets now also keep local slide helpers off raw landed returns
  by default:
  `src/ui/snippets/carousel/{basic,sizes,plugin_wheel_gestures,spacing_responsive,loop_carousel,options,loop_downgrade_cannot_loop,spacing,usage,sizes_thirds,parts,api,duration_embla,rtl,plugin_autoplay,plugin_autoplay_delays,plugin_autoplay_controlled,plugin_autoplay_stop_on_focus,plugin_autoplay_stop_on_last_snap,events}.rs`
  now keep `slide_card(...) -> impl IntoUiElement<fret_app::App> + use<>` wherever a card helper
  exists and `slide(...) -> impl IntoUiElement<fret_app::App> + use<>` wherever a slide wrapper
  exists, with explicit `.into_element(cx)` only at `ui::container(...)`,
  `CarouselItem::new(...)`, and final child-collection seams.
- selected UI Gallery skeleton snippets now also keep local shape/row helpers off raw landed
  returns by default:
  `src/ui/snippets/skeleton/{avatar,rtl,form,table}.rs`
  now use `round(...)` / `row(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at the stack/row collection seam.
- selected UI Gallery popover snippets now also keep layout-wrapper helpers off raw landed returns
  by default:
  `src/ui/snippets/popover/{basic,demo,with_form}.rs`
  now uses `centered<H, B>(body: B) -> impl IntoUiElement<H> + use<H, B>`
  where `B: IntoUiElement<H>`,
  so the wrapper no longer forces callers to pre-land `AnyElement`.
- selected UI Gallery dropdown-menu snippets now also keep preview wrappers on the unified
  conversion contract:
  `src/ui/snippets/dropdown_menu/mod.rs`
  now uses `preview_frame<H, B>(body: B) -> impl IntoUiElement<H> + use<H, B>` and
  `preview_frame_with<H, F, B>(...) -> impl IntoUiElement<H> + use<H, F, B>`,
  and child snippets land them explicitly with `.into_element(cx)` at the render boundary.
- selected UI Gallery AI snippet wrappers and doc-preview helpers now also keep local layout and
  section helpers off raw landed returns by default:
  `src/ui/snippets/ai/{context_default,context_demo}.rs`
  now use `centered<H, B>(body: B) -> impl IntoUiElement<H> + use<H, B>` where
  `B: IntoUiElement<H>`;
  `src/ui/snippets/ai/{file_tree_basic,file_tree_expanded}.rs`
  now use `preview(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/file_tree_large.rs`
  now uses `preview(...)` and `invisible_marker(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/test_results_demo.rs`
  now uses `progress_section(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/attachments_usage.rs`
  now uses `render_grid_attachment(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/{attachments_grid,attachments_list}.rs`
  now use `render_grid_attachment(...)` / `render_list_attachment(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/file_tree_demo.rs`
  now uses `invisible_marker(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/ai/speech_input_demo.rs`
  now uses `body_text(...)` and `clear_action(...) -> impl UiChild + use<>`;
  `src/ui/snippets/breadcrumb/dropdown.rs`
  now uses `dot_separator(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at doc-section, child-array, and breadcrumb-list seams.
- internal UI Gallery wrapper shells now also keep raw landing local instead of forcing pre-landed
  `AnyElement` inputs:
  `src/ui/doc_layout.rs`
  now uses `demo_shell<B>(...) -> impl IntoUiElement<fret_app::App> + use<B>` and lands the body
  at the shell boundary;
  `src/ui/previews/pages/editors/code_editor/mvp/gates.rs`
  now uses `gate_panel<B>(...) -> impl IntoUiElement<fret_app::App> + use<B>` and lands the
  editor child only at the preview-panel boundary.
- the internal docs prose/layout lane is now also on the default app-facing helper surface:
  `src/ui/doc_layout.rs::{wrap_row,wrap_controls_row,text_table,muted_full_width,muted_inline}` and the local
  `notes_block(... )::muted_flex_1_min_w_0(...)` helper now use
  `UiCx -> impl UiChild` instead of `ElementContext<'_, H> -> AnyElement`, and the corresponding
  source gate now records those gallery-only text helpers as non-generic app helpers.
- the remaining raw seams in `src/ui/doc_layout.rs` are now explicitly audited instead of being
  "accidental leftovers":
  landed `DocSection.preview` storage still exists because the scaffold decorates preview roots
  after section assembly, and `gap_card(...)` stays a tuple-return raw seam because placeholder
  sections still register landed preview values. `render_doc_page(...)`, `wrap_preview_page(...)`,
  `icon(...)`, `render_section(...)`, `preview_code_tabs(...)`, `code_block_shell(...)`, and
  `section_title(...)` are now all back on typed helper signatures, with explicit landing either
  inside the scaffold or at the page/preview call sites. The
  `gallery_doc_layout_retains_only_intentional_raw_boundaries`,
  `render_doc_page_callers_land_the_typed_doc_page_explicitly`, and
  `wrap_preview_page_callers_land_the_typed_preview_shell_explicitly` source gates now lock that
  split until the page-collection lane migrates as a batch.
- the internal overlay preview lane is now also explicitly audited:
  `src/ui/previews/gallery/overlays/overlay.rs`,
  `src/ui/previews/gallery/overlays/overlay/layout.rs`,
  `src/ui/previews/gallery/overlays/overlay/widgets.rs`, and
  `src/ui/previews/gallery/overlays/overlay/flags.rs` now split much more cleanly:
  `layout.rs::{row,row_end,compose_body}` and `flags.rs::last_action_status(...)` are back on
  typed helper signatures, `widgets.rs` no longer stores an `OverlayWidgets` landed-root
  inventory and now keeps all local widget helpers on `UiCx -> impl UiChild + use<>`, while
  `overlay.rs::preview_overlay(...)` and `flags.rs::status_flags(...)` remain the only intentional
  raw seams because the diagnostics surface still returns a concrete result vector; the
  `ui_authoring_surface_internal_previews::gallery_overlay_preview_retains_intentional_raw_boundaries`
  gate now locks that narrower inventory.
- ecosystem follow-up has now started on 2026-03-14:
  `ecosystem/fret-ui-shadcn/src/{dropdown_menu.rs,context_menu.rs}` now expose typed
  `compose()` root builders that implement `IntoUiElement<H>` directly, and the overlay gallery
  helpers plus canonical `dropdown_menu/usage.rs` and `context_menu/usage.rs` snippets now teach
  that typed root path instead of eagerly landing through `build_parts(...)`; remaining overlay
  root cleanup now concentrates on migrating other first-party call sites.
- overlay-family extension on 2026-03-14:
  existing `DialogComposition`, `AlertDialogComposition`, `SheetComposition`, and
  `DrawerComposition` now also implement `IntoUiElement<H>` directly, so first-party overlay
  helpers can return typed composition builders across the whole modal/menu family instead of
  landing roots eagerly inside helper bodies.
- snippet migration addendum on 2026-03-14:
  first-party `dropdown_menu` / `context_menu` usage-bearing snippets in `apps/fret-ui-gallery`
  now overwhelmingly teach `compose().trigger(...).content(...).entries(...)`; the only remaining
  `DropdownMenu::build_parts(...)` snippet on that lane is `dropdown_menu/parts.rs`, which is now
  an intentional low-level adapter example rather than the default docs path.
- the low-traffic notification teaching lane is now also aligned:
  `src/ui/snippets/toast/deprecated.rs` now exposes
  `pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>`,
  `src/ui/pages/toast.rs` now uses `DocSection::build(cx, ...)`, and
  `src/ui/snippets/sonner/mod.rs::local_toaster(...)` now stays on the app-owned `UiChild`
  surface with the explicit landing seam moved back to `pages/sonner.rs`; the
  `toast_*` and `sonner_local_toaster_prefers_ui_child_over_anyelement` gates now lock that
  posture.
- the internal code-editor MVP preview lane now also sheds a small raw-helper tail:
  `src/ui/previews/pages/editors/code_editor/mvp/{header,word_boundary,gates}.rs`
  now keep `build_header(...)`, `word_boundary_controls(...)`,
  `word_boundary_debug_view(...)`, `gate_panel(...)`, and the
  `*_gate(...)` helpers on `UiCx -> impl UiChild`, while
  `src/ui/previews/pages/editors/code_editor/mvp.rs` performs the explicit landing only at the
  final typed `wrap_preview_page(...)` call site; the
  `ui_authoring_surface_internal_previews::code_editor_mvp_internal_helpers_prefer_ui_child_over_anyelement`
  gate now locks that posture.
- a few lower-traffic internal preview helpers are now also aligned on the same rule:
  `src/ui/previews/pages/editors/text/outline_stroke.rs::toggle_button(...)` and
  `src/ui/previews/pages/editors/text/mixed_script_fallback.rs::sample_row(...)` plus
  `src/ui/previews/pages/editors/text/feature_toggles.rs::{toggle_button,sample_text}(...)`
  now use `UiCx -> impl UiChild`, and
  `src/ui/previews/pages/harness/intro.rs::{card(...),preview_intro(...)}` now keep the local
  card helper typed while registering the overview block through `DocSection::build(cx, ...)`; the
  `ui_authoring_surface_internal_previews::selected_internal_preview_helpers_prefer_typed_outputs`
  gate now locks those smaller internal-preview teaching surfaces too.
- selected UI Gallery avatar snippets now also keep row wrappers, avatar builders, and group/icon
  helpers off raw landed returns by default:
  `src/ui/snippets/avatar/{demo,group,with_badge,fallback_only,sizes,group_count,group_count_icon,badge_icon,dropdown}.rs`
  now use `wrap_row(...)`, `avatar_with_image(...)`, `avatar_with_badge(...)`,
  `avatar_fallback_only(...)`, `group(...)`, `group_with_count(...)`, and `icon(...)`
  as `impl IntoUiElement<H> + use<...>` helpers,
  with explicit `.into_element(cx)` only at `AvatarGroup::new(...)`, `children([..])`, and final
  render-boundary seams.
- selected UI Gallery button snippets now also keep row wrappers and local size-composition helpers
  off raw landed returns by default:
  `src/ui/snippets/button/{demo,size,with_icon,link_render,rtl,loading,variants,button_group,rounded}.rs`
  now use `wrap_row(...) -> impl IntoUiElement<H> + use<H, F>`,
  and `src/ui/snippets/button/size.rs`
  now also keeps `row(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at vector collection and final render-boundary seams.
- selected UI Gallery tabs snippets now also keep local form-field helpers off raw landed returns
  by default:
  `src/ui/snippets/tabs/demo.rs`
  now uses `field(...) -> impl IntoUiElement<fret_app::App> + use<>`,
  with explicit `.into_element(cx)` only at `CardContent::new(...)` child vectors and
  `TabsItem::new(...)` seams.
- selected UI Gallery collapsible snippets now also keep icon/field/tree helpers off raw landed
  returns by default:
  `src/ui/snippets/collapsible/{basic,settings_panel,rtl,file_tree}.rs`
  now use `rotated_lucide(...)`, `radius_input(...)`, `details_collapsible(...)`,
  `file_leaf(...)`, and `folder(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at child arrays, `with_direction_provider(...)`, and
  `Collapsible::into_element_with_open_model(...)` seams.
- selected UI Gallery hover-card and tooltip snippets now also keep local overlay/content helpers
  off raw landed returns by default:
  `src/ui/snippets/hover_card/{sides,trigger_delays}.rs`
  now use `card(...)` and `demo_content(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/tooltip/{rtl,sides}.rs`
  now use `make_tooltip(...) -> impl IntoUiElement<H> + use<H>`, and
  `src/ui/snippets/tooltip/rtl.rs`
  now also keeps `make_tooltip_with_test_ids(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at row child-collection seams and the final
  render/provider boundary.
- selected UI Gallery resizable snippets now also keep panel/container wrapper helpers off raw
  landed returns by default:
  `src/ui/snippets/resizable/usage.rs`
  now uses `panel(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/resizable/{vertical,handle}.rs`
  now use `box_group<H, B>(..., body: B) -> impl IntoUiElement<H> + use<H, B>` where
  `B: IntoUiElement<H>`,
  and `panel(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at `ResizablePanel::new([..])` entry seams and the final
  render boundary.
- selected UI Gallery data-table and table-action snippets now also keep alignment/footer/row
  helpers off raw landed returns by default:
  `src/ui/snippets/data_table/{basic_demo,default_demo,guide_demo}.rs`
  now use `align_end<B>(child: B) -> impl IntoUiElement<fret_app::App> + use<B>` where
  `B: IntoUiElement<fret_app::App>`;
  `src/ui/snippets/data_table/default_demo.rs`
  now also keeps `footer(...) -> impl IntoUiElement<fret_app::App> + use<>`;
  `src/ui/snippets/data_table/{basic_demo,rtl_demo}.rs`
  now keep `bottom_controls(...) -> impl IntoUiElement<fret_app::App> + use<>`;
  `src/ui/snippets/data_table/rtl_demo.rs`
  now uses `align_inline_start<B>(cx, child) -> impl IntoUiElement<fret_app::App> + use<B>`;
  `src/ui/snippets/table/actions.rs`
  now uses `align_end<B>(child: B) -> impl IntoUiElement<fret_app::App> + use<B>` and
  `action_row(...) -> impl IntoUiElement<fret_app::App> + use<>`,
  with explicit `.into_element(cx)` only at data-table cell/table-row seams and final render
  boundaries.
- selected UI Gallery button-group, toggle-group, and drawer snippets now also expose reusable
  helpers as `IntoUiElement`-based signatures:
  `src/ui/snippets/button_group/api_reference.rs`
  now exports `basic_button_group(...)`, `button_group_with_separator(...)`, and
  `button_group_with_text(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/toggle_group/size.rs`
  now uses `group(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/drawer/demo.rs`
  now uses `goal_adjust_button(...)` and `goal_chart(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/drawer/{responsive_dialog,sides,scrollable_content}.rs`
  now use `profile_field(...)`, `profile_form(...)`, `side_button(...)`, and
  `paragraph_block(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at `DialogContent::new(...)`,
  `DrawerContent::new(...)`, child vectors, and scroll-area/content seams.
- selected UI Gallery sheet and dialog snippets now also keep shared form helpers off raw landed
  returns by default:
  `src/ui/snippets/sheet/{demo,rtl}.rs` and `src/ui/snippets/dialog/{demo,rtl}.rs`
  now use `profile_fields(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at `SheetContent::new(...)`,
  `DialogContent::new(...)`, and intermediate container seams.
- selected UI Gallery dialog scroll-content snippets now also keep paragraph/content helpers off
  raw landed returns by default:
  `src/ui/snippets/dialog/{scrollable_content,sticky_footer}.rs`
  now use `lorem_block(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at `ScrollArea::new([..])`,
  `DialogContent::new(...)`, and final dialog-content seams.
- selected UI Gallery separator snippets now also keep local section/row helpers off raw landed
  returns by default:
  `src/ui/snippets/separator/{menu,list}.rs`
  now use `section(...)` and `row(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at sibling child-collection seams.
- selected UI Gallery table snippets now also keep shared table wrappers off raw landed returns by
  default:
  `src/ui/snippets/table/{demo,footer,rtl}.rs`
  now use `make_invoice_table(...) -> impl IntoUiElement<fret_app::App> + use<>`,
  with the explicit raw landing kept inside the helper at
  `Table::build(...).into_element(cx).test_id(test_id)` because the table builder still needs an
  internal semantics-decoration seam.
- selected UI Gallery sidebar snippets now also keep menu-entry helpers off raw landed returns by
  default:
  `src/ui/snippets/sidebar/{demo,controlled,mobile}.rs`
  now use `menu_button(...) -> impl IntoUiElement<H> + use<H>`;
  `src/ui/snippets/sidebar/rtl.rs`
  now uses `menu_button(...) -> impl IntoUiElement<fret_app::App> + use<>`,
  with explicit `.into_element(cx)` only at `SidebarMenuItem::new(...)` seams.
- selected UI Gallery aspect-ratio snippets now also keep image/frame helpers off raw landed
  returns by default:
  `src/ui/snippets/aspect_ratio/{portrait,square,rtl}.rs`
  now use `portrait_image(...)`, `square_image(...)`, `rtl_image(...)`, and
  `ratio_example(...) -> impl IntoUiElement<H> + use<H>`,
  with explicit `.into_element(cx)` only at `AspectRatio::with_child(...)` and render-boundary
  seams.
- selected UI Gallery item, toast, and motion-presets snippets now also keep local helpers off raw
  landed returns by default:
  `src/ui/snippets/item/{avatar,icon,link,link_render,dropdown,extras_rtl,gallery}.rs`
  now keep `icon_button(...)`, `item_team(...)`, `item_icon(...)`, `icon(...)`,
  `outline_button(...)`, `outline_button_sm(...)`, `item_basic(...)`, and `item_avatar(...)`
  helpers on `impl IntoUiElement<fret_app::App> + use<>`, with explicit `.into_element(cx)` only
  at `ItemMedia::new(...)`, `ItemActions::new(...)`, dropdown child arrays, vector collection,
  and the final render-boundary seams;
  `src/ui/snippets/toast/deprecated.rs`
  now uses `centered<B>(body: B) -> impl IntoUiElement<fret_app::App> + use<B>`;
  `src/ui/snippets/motion_presets/fluid_tabs_demo.rs`
  now uses `panel(...) -> impl IntoUiElement<fret_app::App> + use<>`.
- explicit raw seams remain where the helper is genuinely low-level composition glue, for example
  the internal body-landing step inside `custom_effect_v1_demo.rs::lens_shell(...)` and
  `custom_effect_v2_demo.rs::lens_shell(...)`, plus stage child arrays and retained-subtree
  bridges that intentionally still own raw `AnyElement` assembly.
- heterogenous sibling arrays remain valid explicit landing seams even after helper migration; for
  example `custom_effect_v3_demo.rs::{stage, stage_controls}` and
  `postprocess_theme_demo.rs::render(...)` now keep helpers on `IntoUiElement<KernelApp>` but
  still call `.into_element(cx)` where the surrounding child collection intentionally wants
  `AnyElement`.
- `async_playground_demo.rs` now follows the same rule:
  local helpers stay on `IntoUiElement<KernelApp>`, while
  `render(...)`, `body(...)`, `main_panel(...)`, `inspector_panel(...)`, and
  `query_panel_for_mode(...)` still land explicitly at heterogenous child arrays,
  `TabsItem::new([..])`, and `ScrollArea::new([..])` seams.

Validation note on 2026-03-12:

- verified the expanded UI Gallery helper gate with
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app selected_`;
  the focused source gate now covers 32 `selected_*` checks and passed after the AI wrapper,
  breadcrumb, avatar, button, tabs, collapsible, drawer, sheet, dialog (including scrollable and
  sticky-footer content helpers), separator, table, sidebar, aspect-ratio, popover, hover-card,
  tooltip, resizable, data-table, table-action, combobox state-row/state-rows, item
  (`extras_rtl`, `dropdown`, `gallery`), and carousel (`basic`/`usage` plus `api`,
  `duration_embla`, `rtl`, `plugin_autoplay*`, and `events`) helper migrations landed.
- verified the advanced example helper gate with
  `cargo nextest run -p fret-examples --lib`;
  the source gate now also records `custom_effect_v3_demo.rs::{stage, stage_controls,
  animated_backdrop, lens_row, lens_shell}` and `postprocess_theme_demo.rs::{inspector, stage,
  stage_body, stage_cards}` plus
  `async_playground_demo.rs::{header_bar, body, catalog_panel, main_panel, inspector_panel,
  policy_editor, query_panel_for_mode, query_inputs_row, query_result_view, status_badge}`
  on `IntoUiElement<KernelApp>`-based signatures, with explicit `.into_element(cx)` kept only at
  heterogenous sibling child-collection, tabs-item, and scroll-area seams.

## M4 — Delete the old public surface

- [x] Remove `UiIntoElement` from curated public surfaces.
- [x] Remove `UiHostBoundIntoElement` from curated public surfaces.
- [x] Remove `UiChildIntoElement` from curated public surfaces.
- [x] Remove `UiBuilderHostBoundIntoElementExt` from curated public surfaces.
- [x] Rewrite or delete stale docs that still teach the old names.

## M5 — Add guardrails

- [x] Add a gate that the app prelude does not publicly re-export old conversion traits.
- [x] Add a gate that the component prelude exports exactly one public conversion trait.
- [x] Add a source/doc gate that the canonical authoring compare set (`simple_todo_v2_target`,
  `todo_demo`, and the simple-todo scaffold template) stays on the target conversion vocabulary.
- [x] Add a source/doc gate that app-facing examples prefer `Ui` / `UiChild`.
- [x] Add a source/doc gate that generic reusable first-party helpers prefer the unified
  conversion trait over raw `AnyElement` when a raw landed element is not required.
- [x] Add a source gate that old names (`UiChildIntoElement`, `UiHostBoundIntoElement`,
  `UiBuilderHostBoundIntoElementExt`) do not return in curated surfaces.

Implementation note on 2026-03-12:

- the canonical compare set now has direct stale-name guards in:
  `apps/fret-cookbook/src/lib.rs`,
  `apps/fret-examples/src/lib.rs`,
  and `apps/fretboard/src/scaffold/templates.rs`.
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs` now guards that
  `ui_ext/support.rs` and `ui_ext/data.rs` stay on `IntoUiElement<H>` rather than
  reintroducing direct `UiIntoElement` glue.
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs` now also guards that
  `ui_builder_ext/*` reusable helper closures keep accepting `IntoUiElement<H>`.
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs` now also guards that
  `ui_builder_ext/*::into_element(...)` remains an explicit `AnyElement` landing seam while its
  closure inputs do not regress to `AnyElement`-typed signatures.
- `ecosystem/fret/tests/reusable_component_helper_surface.rs` now guards the facade-level
  source/doc story: shadcn reusable helpers stay on `IntoUiElement<H>` rather than requiring
  pre-landed `AnyElement` inputs.
- `docs/component-author-guide.md` and `docs/component-authoring-contracts.md` now teach only
  `IntoUiElement<H>` on the curated component-authoring lane.
- `ecosystem/fret-ui-kit/tests/curated_conversion_surface_docs.rs` now guards curated docs against
  legacy conversion trait names.
- semantic decorator helper names are now neutralized to `UiElement*Ext`, and
  `ecosystem/fret-ui-kit::source_policy_tests` guards against reintroducing the old
  `UiIntoElement*Ext` export names on declarative surfaces.
- future ergonomics follow-up: if explicit `.into_element(cx).attach_semantics(...)` starts
  showing up frequently in app/ecosystem call sites, add one unified semantics decorator helper on
  top of the public `IntoUiElement<H>` contract rather than teaching per-component builder-specific
  `.a11y(...)` APIs.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now guards that selected
  `badge/*` snippet helpers stay on `IntoUiElement<H>` rather than reverting to raw `AnyElement`
  row helpers.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `context_menu/*` snippet trigger helpers stay on `IntoUiElement<H>` rather than reverting to raw
  `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `combobox/*` snippet `state_row(...)` and `state_rows(...)` helpers stay on
  `IntoUiElement<fret_app::App>` rather than reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `pagination/*` snippet page-number helpers stay on `IntoUiElement<H>` rather than reverting to
  raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `carousel/*` snippet slide helpers stay on `IntoUiElement<fret_app::App>` rather than
  reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `skeleton/*` snippet shape/row helpers stay on `IntoUiElement<H>` rather than reverting to raw
  `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `popover/*` wrapper helpers accept/return `IntoUiElement<H>` rather than forcing pre-landed
  `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that
  `dropdown_menu/mod.rs` preview wrappers stay on `IntoUiElement<H>` rather than forcing
  pre-landed `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `button_group/*`, `toggle_group/*`, and `drawer/*` reusable helpers stay on `IntoUiElement`
  signatures rather than reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `hover_card/*` content helpers and `tooltip/*` helper builders stay on `IntoUiElement<H>`
  signatures rather than reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `resizable/*` panel/container wrapper helpers stay on `IntoUiElement<H>`-based signatures
  rather than reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `data_table/*` alignment/footer helpers and `table/actions.rs` row helpers stay on
  `IntoUiElement<fret_app::App>` rather than reverting to raw `AnyElement`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` now also guards that selected
  `item/*`, `toast/*`, and `motion_presets/*` helpers stay on `IntoUiElement` signatures rather
  than reverting to raw `AnyElement`.

## M6 — Keep advanced/raw seams explicit and justified

- [x] Document the legitimate raw `AnyElement` cases:
  - [x] overlay/controller internals.
    Evidence: `ecosystem/fret-ui-shadcn/src/ui_builder_ext/*.rs` still keeps
    `into_element(...) -> AnyElement` as the explicit landing seam while closure inputs are already
    typed via `IntoUiElement<H>`.
  - [x] no-`cx` request constructors.
    Evidence: `ecosystem/fret-ui-kit/src/overlay_controller.rs::OverlayRequest` and the
    request-constructor families in
    `ecosystem/fret-ui-kit/src/primitives/{dialog,popover,alert_dialog,select,tooltip}.rs`
    intentionally remain raw because typed children have already been landed before those request
    payloads are assembled.
  - [x] diagnostics/harness helpers.
    Evidence: retained/manual-assembly seams such as
    `apps/fret-cookbook/examples/chart_interactions_basics.rs::chart_canvas(...) -> AnyElement`
    remain explicitly called out in this workstream rather than reintroduced into default teaching
    surfaces.
  - [x] low-level heterogeneous landing APIs.
    Evidence: `kbd.rs::kbd_icon(...)` remains inventoried by
    `surface_policy_tests::{explicit_raw_or_bridge_public_anyelement_helpers_stay_small_and_reviewable,kbd_icon_stays_an_explicit_raw_helper_for_kbd_child_lists}`,
    while combobox anchor overrides now route through
    `surface_policy_tests::combobox_surface_uses_generic_popover_anchor_builder_not_combobox_specific_raw_alias`.
  - [x] manual assembly / advanced runtime seams.
    Evidence: `text_edit_context_menu.rs::{text_edit_context_menu,text_selection_context_menu,text_edit_context_menu_controllable,text_selection_context_menu_controllable}`
    now keep the final wrapper landing seam explicit while accepting typed trigger values through
    `IntoUiElement<H>`; `state.rs::{use_selector_badge,query_status_badge}` have been promoted
    back to typed `Badge` outputs, `query_error_alert(...)` now returns `Option<Alert>`, and this
    lane is covered by
    `surface_policy_tests::state_helpers_prefer_typed_badge_outputs_when_no_runtime_landing_seam_is_required`.
  - [x] slot-scoped typed helper surfaces.
    Evidence: `tooltip.rs::TooltipContent::{build,text}(...)` now stay on the typed lane and are
    covered by
    `surface_policy_tests::tooltip_content_helpers_prefer_typed_build_and_text_outputs_when_slot_scope_is_required`.
- [x] Ensure raw surfaces remain explicit rather than leaking back into the app-facing story.
  - Current status: legacy module-local root helpers are cleared (`drawer(...)`, `menubar(...)`,
    `combobox(...)` deleted).
  - These helpers are now inventoried by
    `surface_policy_tests::legacy_public_anyelement_helper_inventory_is_explicit_until_promoted_or_deleted`
    and the inventory is expected to stay empty unless a new helper is added by explicit review.
  - Current shadcn deliberate-raw helper contracts are now fixed to
    `kbd.rs::kbd_icon(...)` and
    `text_edit_context_menu.rs::{text_edit_context_menu,text_selection_context_menu,text_edit_context_menu_controllable,text_selection_context_menu_controllable}`,
    with `surface_policy_tests::text_edit_context_menu_helpers_keep_landing_seam_explicit_but_accept_typed_triggers`
    documenting the final wrapper-seam rationale directly in source.
