use super::super::super::super::*;
use crate::ui::doc_layout;
use fret::AppComponentCx;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

fn debug_readout_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    doc_layout::control_readout_text(cx, text)
}

fn delete_last_char(text: &mut String) -> bool {
    let Some((start, _)) = text.char_indices().last() else {
        return false;
    };
    text.truncate(start);
    true
}

#[cfg(target_arch = "wasm32")]
fn inject_debug_beforeinput(input_type: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(injector) = js_sys::Reflect::get(
        &window,
        &wasm_bindgen::JsValue::from_str("__FRET_IME_DEBUG_INJECT_BEFOREINPUT"),
    ) else {
        return false;
    };
    let Ok(injector) = injector.dyn_into::<js_sys::Function>() else {
        return false;
    };
    injector
        .call1(&window, &wasm_bindgen::JsValue::from_str(input_type))
        .is_ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn inject_debug_beforeinput(_input_type: &str) -> bool {
    false
}

pub(in crate::ui) fn preview_web_ime_harness(
    cx: &mut AppComponentCx<'_>,
    theme: &Theme,
    text_input: Model<String>,
    text_area: Model<String>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ImeHarnessState {
        committed: String,
        preedit: Option<String>,
        ime_enabled: bool,
        text_input_count: u64,
        ime_commit_count: u64,
        ime_preedit_count: u64,
        ime_delete_surrounding_count: u64,
        ime_enabled_count: u64,
        ime_disabled_count: u64,
        last: String,
    }

    let state = cx.slot_state(
        || std::rc::Rc::new(std::cell::RefCell::new(ImeHarnessState::default())),
        |st| st.clone(),
    );

    let header = ui::v_flex(|cx| {
        vec![
            doc_layout::paragraph_text(
                cx,
                "Goal: validate the wasm textarea IME bridge (ADR 0180).",
            ),
            doc_layout::paragraph_text(
                cx,
                "Try: CJK IME preedit → commit; ensure no double insert on compositionend + input.",
            ),
            doc_layout::paragraph_text(
                cx,
                "Click inside the region to focus it (IME should enable).",
            ),
            doc_layout::paragraph_text(
                cx,
                "Debug buttons below inject web-only beforeinput control intents without a physical keydown.",
            ),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N2)
    .into_element(cx);

    let inputs = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_token("background"))),
            LayoutRefinement::default().w_full(),
        ),
        |cx| {
            let body = ui::v_flex(|cx: &mut AppComponentCx<'_>| {
                vec![
                    doc_layout::control_readout_text(cx, "Editable widgets (sanity check):"),
                    shadcn::Input::new(text_input)
                        .a11y_label("Web IME input")
                        .placeholder("Type here (IME should work on web)")
                        .into_element(cx),
                    shadcn::Textarea::new(text_area)
                        .a11y_label("Web IME textarea")
                        .into_element(cx),
                ]
            })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2)
            .into_element(cx);
            vec![body]
        },
    );

    let mut region_props = fret_ui::element::TextInputRegionProps::default();
    region_props.layout.size.width = fret_ui::element::Length::Fill;
    region_props.layout.size.height = fret_ui::element::Length::Fill;
    region_props.a11y_label = Some(std::sync::Arc::from("Web IME harness"));

    let (a11y_value, a11y_text_selection, a11y_text_composition, desired_ime_cursor_area) = {
        let st = state.borrow();
        let committed = st.committed.as_str();
        let preedit = st.preedit.as_deref().unwrap_or("");

        let value = if preedit.is_empty() {
            committed.to_string()
        } else {
            format!("{committed}{preedit}")
        };

        // The harness caret is modeled at the end of the composed value.
        // Ranges are UTF-8 byte offsets within `a11y_value` (ADR 0071).
        let caret_utf8 = u32::try_from(value.len()).unwrap_or(u32::MAX);
        let selection = Some((caret_utf8, caret_utf8));
        let composition = (!preedit.is_empty()).then(|| {
            let start = u32::try_from(committed.len()).unwrap_or(u32::MAX);
            let end = caret_utf8;
            (start, end)
        });

        // Best-effort: place the IME cursor near the bottom of the region and advance X based on
        // the displayed composed text length. This is only for visual verification in the gallery.
        let bounds = cx.bounds;
        let cols = value.chars().count();
        let min_x = bounds.origin.x.0 + 16.0;
        let max_x = (bounds.origin.x.0 + bounds.size.width.0 - 16.0).max(min_x);
        let x = Px((bounds.origin.x.0 + 16.0 + cols as f32 * 8.0).clamp(min_x, max_x));

        let min_y = bounds.origin.y.0 + 16.0;
        let max_y = (bounds.origin.y.0 + bounds.size.height.0 - 16.0).max(min_y);
        let y = Px((bounds.origin.y.0 + bounds.size.height.0 - 28.0).clamp(min_y, max_y));
        let ime_cursor_area = Rect::new(fret_core::Point::new(x, y), Size::new(Px(1.0), Px(18.0)));

        (
            Some(std::sync::Arc::<str>::from(value)),
            selection,
            composition,
            Some(ime_cursor_area),
        )
    };

    region_props.a11y_value = a11y_value;
    region_props.a11y_text_selection = a11y_text_selection;
    region_props.a11y_text_composition = a11y_text_composition;
    region_props.ime_cursor_area = desired_ime_cursor_area;

    let region = cx
        .text_input_region(region_props, |cx| {
        let region_id = cx.root_id();
        let state_for_keydown = state.clone();
        cx.key_on_key_down_focused_for(
            region_id,
            std::sync::Arc::new(move |host, action_cx, key| {
                if key.modifiers.ctrl || key.modifiers.alt || key.modifiers.meta {
                    return false;
                }

                let mut st = state_for_keydown.borrow_mut();
                let mut handled = true;
                match key.key {
                    fret_core::KeyCode::Backspace => {
                        let _ = delete_last_char(&mut st.committed);
                    }
                    fret_core::KeyCode::Delete => {}
                    fret_core::KeyCode::Enter | fret_core::KeyCode::NumpadEnter => {
                        st.committed.push('\n');
                    }
                    _ => handled = false,
                }

                if !handled {
                    return false;
                }

                st.last = format!("KeyDown({:?})", key.key);
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
                true
            }),
        );

        let state_for_text_input = state.clone();
        cx.text_input_region_on_text_input(std::sync::Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  text: &str| {
                let mut st = state_for_text_input.borrow_mut();
                st.text_input_count = st.text_input_count.saturating_add(1);
                st.last = format!("TextInput({:?})", text);
                st.committed.push_str(text);
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
                true
            },
        ));

        let state_for_ime = state.clone();
        cx.text_input_region_on_ime(std::sync::Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  ime: &fret_core::ImeEvent| {
                let mut st = state_for_ime.borrow_mut();
                match ime {
                    fret_core::ImeEvent::Enabled => {
                        st.ime_enabled = true;
                        st.ime_enabled_count = st.ime_enabled_count.saturating_add(1);
                        st.last = "Ime(Enabled)".to_string();
                    }
                    fret_core::ImeEvent::Disabled => {
                        st.ime_enabled = false;
                        st.preedit = None;
                        st.ime_disabled_count = st.ime_disabled_count.saturating_add(1);
                        st.last = "Ime(Disabled)".to_string();
                    }
                    fret_core::ImeEvent::Commit(text) => {
                        st.ime_commit_count = st.ime_commit_count.saturating_add(1);
                        st.last = format!("Ime(Commit({:?}))", text);
                        st.committed.push_str(text);
                        st.preedit = None;
                    }
                    fret_core::ImeEvent::Preedit { text, .. } => {
                        st.ime_preedit_count = st.ime_preedit_count.saturating_add(1);
                        st.last = format!("Ime(Preedit({:?}))", text);
                        st.preedit = (!text.is_empty()).then(|| text.clone());
                    }
                    fret_core::ImeEvent::DeleteSurrounding {
                        before_bytes,
                        after_bytes,
                    } => {
                        st.ime_delete_surrounding_count =
                            st.ime_delete_surrounding_count.saturating_add(1);
                        st.last = format!(
                            "Ime(DeleteSurrounding(before_bytes={before_bytes}, after_bytes={after_bytes}))"
                        );
                    }
                }

                host.notify(action_cx);
                host.request_redraw(action_cx.window);
                true
            },
        ));

        let st = state.borrow();
        let committed_tail = {
            const MAX_CHARS: usize = 120;
            let total = st.committed.chars().count();
            if total <= MAX_CHARS {
                st.committed.clone()
            } else {
                let tail: String = st
                    .committed
                    .chars()
                    .skip(total.saturating_sub(MAX_CHARS))
                    .collect();
                format!("…{tail}")
            }
        };

        let preedit = st
            .preedit
            .as_deref()
            .unwrap_or("<none>");
        let harness_region_ime_enabled = st.ime_enabled as u8;

        let panel = cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .bg(ColorRef::Color(theme.color_token("background"))),
                LayoutRefinement::default()
                    .w_full()
                    .h_px(MetricRef::Px(Px(240.0))),
            ),
            |cx| {
                let body = ui::v_flex(|cx: &mut AppComponentCx<'_>| {
                        let inject_chip = |cx: &mut AppComponentCx<'_>,
                                           label: &'static str,
                                           input_type: &'static str,
                                           test_id: &'static str| {
                            let state = state.clone();
                            cx.pointer_region(
                                fret_ui::element::PointerRegionProps::default(),
                                move |cx| {
                                    cx.pointer_region_on_pointer_down(std::sync::Arc::new(
                                        move |host, action_cx, _down| {
                                            host.request_focus(region_id);
                                            let ok = inject_debug_beforeinput(input_type);
                                            let mut st = state.borrow_mut();
                                            st.last = format!(
                                                "DebugBeforeInput({input_type}, ok={})",
                                                ok as u8
                                            );
                                            host.notify(action_cx);
                                            host.request_redraw(action_cx.window);
                                            true
                                        },
                                    ));
                                    let body = cx.container(
                                        decl_style::container_props(
                                            theme,
                                            ChromeRefinement::default()
                                                .border_1()
                                                .rounded(Radius::Sm)
                                                .bg(ColorRef::Color(
                                                    theme.color_token("secondary"),
                                                )),
                                            LayoutRefinement::default(),
                                        ),
                                        move |cx| vec![doc_layout::button_label_text(cx, label)],
                                    );
                                    vec![body]
                                },
                            )
                            .test_id(test_id)
                        };
                        let inject_delete = inject_chip(
                            cx,
                            "Inject delete backward",
                            "deleteContentBackward",
                            "ui-gallery-web-ime-inject-delete-backward",
                        );
                        let inject_line_break = inject_chip(
                            cx,
                            "Inject line break",
                            "insertLineBreak",
                            "ui-gallery-web-ime-inject-line-break",
                        );
                        let inject_paragraph = inject_chip(
                            cx,
                            "Inject paragraph",
                            "insertParagraph",
                            "ui-gallery-web-ime-inject-paragraph",
                        );
                        let controls = ui::h_flex(move |_cx| {
                            vec![inject_delete, inject_line_break, inject_paragraph]
                        })
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N2)
                        .items_start()
                        .into_element(cx);

                        let mut lines = vec![
                            controls,
                            debug_readout_text(cx, format!(
                                "harness_region_ime_enabled={harness_region_ime_enabled}"
                            )),
                            debug_readout_text(cx, format!(
                                "desired_ime_cursor_area={:?}",
                                desired_ime_cursor_area
                            )),
                            debug_readout_text(cx, format!("preedit={preedit:?}")),
                            debug_readout_text(cx, format!("committed_tail={committed_tail:?}")),
                            debug_readout_text(cx, format!("last_event={:?}", st.last)),
                            debug_readout_text(cx, "Console logging: add ?ime_debug=1 or set window.__FRET_IME_DEBUG=true"),
                            debug_readout_text(cx, format!(
                                "counts: text_input={} ime_commit={} ime_preedit={} ime_delete_surrounding={} enabled={} disabled={}",
                                st.text_input_count,
                                st.ime_commit_count,
                                st.ime_preedit_count,
                                st.ime_delete_surrounding_count,
                                st.ime_enabled_count,
                                st.ime_disabled_count
                            )),
                        ];

                        if let Some(snapshot) = cx
                            .app
                            .global::<fret_runtime::WindowTextInputSnapshotService>()
                            .and_then(|svc: &fret_runtime::WindowTextInputSnapshotService| {
                                svc.snapshot(cx.window)
                            })
                            .cloned()
                        {
                            lines.push(debug_readout_text(cx, "window_text_input_snapshot:"));
                            lines.push(debug_readout_text(cx, format!(
                                "  focus_is_text_input={} is_composing={}",
                                snapshot.focus_is_text_input as u8, snapshot.is_composing as u8
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  text_len_utf16={} selection_utf16={:?} marked_utf16={:?}",
                                snapshot.text_len_utf16, snapshot.selection_utf16, snapshot.marked_utf16
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  ime_cursor_area={:?}",
                                snapshot.ime_cursor_area
                            )));
                        } else {
                            lines.push(debug_readout_text(cx, "window_text_input_snapshot: <unavailable>"));
                        }

                        if let Some(input_ctx) = cx
                            .app
                            .global::<fret_runtime::WindowInputContextService>()
                            .and_then(|svc: &fret_runtime::WindowInputContextService| {
                                svc.snapshot(cx.window)
                            })
                            .cloned()
                        {
                            lines.push(debug_readout_text(cx, "window_input_context_snapshot:"));
                            lines.push(debug_readout_text(cx, format!(
                                "  focus_is_text_input={} text_boundary_mode={:?}",
                                input_ctx.focus_is_text_input as u8, input_ctx.text_boundary_mode
                            )));
                        } else {
                            lines.push(debug_readout_text(cx, "window_input_context_snapshot: <unavailable>"));
                        }

                        if let Some(key) = cx.app.global::<fret_runtime::TextFontStackKey>() {
                            lines.push(debug_readout_text(cx, format!("text_font_stack_key={}", key.0)));
                        } else {
                            lines.push(debug_readout_text(cx, "text_font_stack_key: <unavailable>"));
                        }

                        if let Some(cfg) = cx.app.global::<fret_core::TextFontFamilyConfig>().cloned()
                        {
                            let fmt = |v: &[String]| -> String {
                                let head = v.iter().take(4).cloned().collect::<Vec<_>>().join(", ");
                                if v.len() > 4 {
                                    format!("[{head}, …] (len={})", v.len())
                                } else {
                                    format!("[{head}] (len={})", v.len())
                                }
                            };
                            lines.push(debug_readout_text(cx, "text_font_families:"));
                            lines.push(debug_readout_text(cx, format!("  ui_sans={}", fmt(&cfg.ui_sans))));
                            lines.push(debug_readout_text(cx, format!("  ui_serif={}", fmt(&cfg.ui_serif))));
                            lines.push(debug_readout_text(cx, format!("  ui_mono={}", fmt(&cfg.ui_mono))));
                            lines.push(debug_readout_text(cx, format!(
                                "  common_fallback={}",
                                fmt(&cfg.common_fallback)
                            )));
                        } else {
                            lines.push(debug_readout_text(cx, "text_font_families: <unavailable>"));
                        }

                        if let Some(catalog) = cx.app.global::<fret_runtime::FontCatalog>().cloned()
                        {
                            let head = catalog
                                .families
                                .iter()
                                .take(6)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ");
                            lines.push(debug_readout_text(cx, "font_catalog:"));
                            lines.push(debug_readout_text(cx, format!(
                                "  revision={} families_len={}",
                                catalog.revision,
                                catalog.families.len()
                            )));
                            if !catalog.families.is_empty() {
                                lines.push(debug_readout_text(cx, format!("  head=[{head}]")));
                            }
                        } else {
                            lines.push(debug_readout_text(cx, "font_catalog: <unavailable>"));
                        }

                        let snapshot = cx
                            .app
                            .global::<fret_core::input::WebImeBridgeDebugSnapshot>()
                            .cloned();
                        if let Some(snapshot) = snapshot {
                            lines.push(debug_readout_text(cx, "bridge_debug_snapshot (wasm textarea):"));
                            lines.push(debug_readout_text(cx, format!(
                                "  enabled={} composing={} suppress_next_input={}",
                                snapshot.enabled as u8,
                                snapshot.composing as u8,
                                snapshot.suppress_next_input as u8
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  last_preedit_text={:?} preedit_cursor_utf16={:?}",
                                snapshot.last_preedit_text.as_deref(),
                                snapshot.last_preedit_cursor_utf16
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  last_commit_text={:?}",
                                snapshot.last_commit_text.as_deref()
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  position_mode={:?} mount_kind={:?} dpr={:?}",
                                snapshot.position_mode.as_deref(),
                                snapshot.mount_kind.as_deref(),
                                snapshot.device_pixel_ratio,
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  textarea_has_focus={:?} active_element_tag={:?}",
                                snapshot.textarea_has_focus, snapshot.active_element_tag
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  last_input_type={:?}",
                                snapshot.last_input_type.as_deref()
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  last_beforeinput_data={:?}",
                                snapshot.last_beforeinput_data.as_deref()
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  last_input_data={:?}",
                                snapshot.last_input_data.as_deref()
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  last_key_code={:?} last_cursor_area={:?}",
                                snapshot.last_key_code, snapshot.last_cursor_area
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  last_cursor_anchor_px={:?}",
                                snapshot.last_cursor_anchor_px
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  counts: beforeinput={} input={} suppressed={} comp_start={} comp_update={} comp_end={} cursor_area_set={}",
                                snapshot.beforeinput_seen,
                                snapshot.input_seen,
                                snapshot.suppressed_input_seen,
                                snapshot.composition_start_seen,
                                snapshot.composition_update_seen,
                                snapshot.composition_end_seen,
                                snapshot.cursor_area_set_seen,
                            )));
                            lines.push(debug_readout_text(cx, format!(
                                "  textarea: chars={:?} sel_utf16={:?}..{:?} client={:?}x{:?} scroll={:?}x{:?}",
                                snapshot.textarea_value_chars,
                                snapshot.textarea_selection_start_utf16,
                                snapshot.textarea_selection_end_utf16,
                                snapshot.textarea_client_width_px,
                                snapshot.textarea_client_height_px,
                                snapshot.textarea_scroll_width_px,
                                snapshot.textarea_scroll_height_px,
                            )));

                            if !snapshot.recent_events.is_empty() {
                                lines.push(debug_readout_text(cx, "  recent_events:"));
                                for e in snapshot.recent_events.iter().rev().take(10) {
                                    lines.push(debug_readout_text(cx, format!("    {e}")));
                                }
                            }
                        } else {
                            lines.push(debug_readout_text(cx, "bridge_debug_snapshot: <unavailable>"));
                        }

                        lines
                    })
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N2).into_element(cx);
                vec![body]
            },
        );

        vec![panel]
    })
        .test_id("ui-gallery-web-ime-region");

    let page = doc_layout::wrap_preview_page(cx, None, "Web IME", vec![header, inputs, region]);

    vec![page.into_element(cx)]
}
