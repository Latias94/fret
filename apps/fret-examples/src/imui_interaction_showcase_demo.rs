//! Advanced/reference demo: IMUI interaction showcase surface.
//!
//! Why advanced: product shell polish, immediate-mode interaction affordances, and shadcn shell
//! chrome belong on a reference/product-validation lane rather than the first-contact teaching
//! path.
//! Not a first-contact teaching surface: keep onboarding on `imui_action_basics` and
//! `imui_editor_proof_demo`, then use this file when you need a presentable shell that still keeps
//! the direct IMUI control-flow story.
//!
//! Showcase surface for immediate-mode interaction affordances.
//! Current proof/contract surface stays in `imui_response_signals_demo`.
//! This file intentionally mixes `fret_imui` control flow with shadcn shell chrome so the
//! immediate lane has one surface that looks like a product rather than a contract log.

use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*};
use fret_core::Px;
use fret_ui::Theme;
use fret_ui::element::AnyElement;
use fret_ui_kit::{ColorRef, Space, UiExt as _, ui};
use fret_ui_shadcn::facade as shadcn;

const TEST_ID_ROOT: &str = "imui-interaction-showcase.root";
const TEST_ID_HERO: &str = "imui-interaction-showcase.hero";
const TEST_ID_LAB: &str = "imui-interaction-showcase.lab";
const TEST_ID_SHELL: &str = "imui-interaction-showcase.shell";
const TEST_ID_TIMELINE: &str = "imui-interaction-showcase.timeline";

#[derive(Clone)]
struct ShowcaseEvent {
    id: u64,
    label: Arc<str>,
}

struct ImUiInteractionShowcaseView;

pub fn run() -> anyhow::Result<()> {
    FretApp::new("imui-interaction-showcase-demo")
        .window("imui_interaction_showcase_demo", (1180.0, 760.0))
        .view::<ImUiInteractionShowcaseView>()?
        .run()?;
    Ok(())
}

impl View for ImUiInteractionShowcaseView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        shadcn::themes::apply_shadcn_new_york(
            app,
            shadcn::themes::ShadcnBaseColor::Slate,
            shadcn::themes::ShadcnColorScheme::Light,
        );
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let pulse_count = cx.state().local_init(|| 0u32);
        let secondary_pulse_count = cx.state().local_init(|| 0u32);
        let long_press_count = cx.state().local_init(|| 0u32);
        let drag_count = cx.state().local_init(|| 0u32);
        let drag_distance = cx.state().local_init(|| 0.0f32);
        let autosave_enabled = cx.state().local_init(|| true);
        let exposure_value = cx.state().local_init(|| 38.0f32);
        let review_mode = cx.state().local_init(|| Some(Arc::<str>::from("Studio")));
        let draft_note = cx.state().local_init(|| {
            String::from("Use IMUI for fast control flow, not for raw visual dumps.")
        });

        let menu_open_count = cx.state().local_init(|| 0u32);
        let submenu_toggle_count = cx.state().local_init(|| 0u32);
        let tab_switch_count = cx.state().local_init(|| 0u32);
        let context_action_count = cx.state().local_init(|| 0u32);
        let context_toggle = cx.state().local_init(|| false);
        let selected_tab = cx.state().local_init(|| Some(Arc::<str>::from("overview")));

        let timeline_next_id = cx.state().local_init(|| 1u64);
        let timeline = cx.state().local_init(|| {
            vec![ShowcaseEvent {
                id: 0,
                label: Arc::from(
                    "Showcase ready. Open menus, switch tabs, and right-click the canvas.",
                ),
            }]
        });

        let pulse_count_value = pulse_count.layout_value(cx);
        let secondary_pulse_count_value = secondary_pulse_count.layout_value(cx);
        let long_press_count_value = long_press_count.layout_value(cx);
        let drag_count_value = drag_count.layout_value(cx);
        let drag_distance_value = drag_distance.layout_value(cx);
        let autosave_enabled_value = autosave_enabled.layout_value(cx);
        let exposure_value_value = exposure_value.layout_value(cx);
        let review_mode_value = review_mode.layout_value(cx);
        let draft_note_value = draft_note.layout_value(cx);
        let menu_open_count_value = menu_open_count.layout_value(cx);
        let submenu_toggle_count_value = submenu_toggle_count.layout_value(cx);
        let tab_switch_count_value = tab_switch_count.layout_value(cx);
        let context_action_count_value = context_action_count.layout_value(cx);
        let context_toggle_value = context_toggle.layout_value(cx);
        let selected_tab_value = selected_tab.layout_value(cx);
        let timeline_value = timeline.layout_value(cx);

        let hero = render_showcase_hero(
            cx,
            pulse_count_value,
            drag_count_value,
            menu_open_count_value,
            tab_switch_count_value,
            selected_tab_value.as_deref().unwrap_or("overview"),
            timeline_value.first().map(|event| event.label.clone()),
        );

        let lab = render_interaction_lab_card(
            cx,
            pulse_count.clone(),
            secondary_pulse_count.clone(),
            long_press_count.clone(),
            drag_count.clone(),
            drag_distance.clone(),
            autosave_enabled.clone(),
            exposure_value.clone(),
            review_mode.clone(),
            draft_note.clone(),
            timeline_next_id.clone(),
            timeline.clone(),
            pulse_count_value,
            secondary_pulse_count_value,
            long_press_count_value,
            drag_count_value,
            drag_distance_value,
            autosave_enabled_value,
            exposure_value_value,
            review_mode_value.clone(),
            draft_note_value.clone(),
        );

        let shell = render_shell_showcase_card(
            cx,
            menu_open_count.clone(),
            submenu_toggle_count.clone(),
            tab_switch_count.clone(),
            context_action_count.clone(),
            context_toggle.clone(),
            selected_tab.clone(),
            timeline_next_id.clone(),
            timeline.clone(),
            menu_open_count_value,
            submenu_toggle_count_value,
            tab_switch_count_value,
            context_action_count_value,
            context_toggle_value,
            selected_tab_value.clone(),
        );

        let timeline_card = render_timeline_card(cx, &timeline_value);

        let body = ui::h_flex(move |cx| {
            vec![
                ui::container(move |_cx| [lab])
                    .w_px(Px(360.0))
                    .flex_shrink_0()
                    .into_element(cx),
                ui::v_flex(move |cx| vec![shell, timeline_card])
                    .gap(Space::N4)
                    .flex_1()
                    .min_w_0()
                    .into_element(cx),
            ]
        })
        .gap(Space::N4)
        .items_stretch()
        .w_full()
        .flex_1()
        .min_w_0()
        .min_h_0();

        ui::container(move |cx| {
            vec![
                ui::v_flex(move |cx| vec![hero, body.into_element(cx)])
                    .gap(Space::N4)
                    .w_full()
                    .h_full()
                    .into_element(cx),
            ]
        })
        .p(Space::N4)
        .size_full()
        .bg(ColorRef::Color(
            Theme::global(&*cx.app).color_token("background"),
        ))
        .into_element(cx)
        .test_id(TEST_ID_ROOT)
        .into()
    }
}

#[allow(clippy::too_many_arguments)]
fn render_interaction_lab_card(
    cx: &mut ElementContext<'_, KernelApp>,
    pulse_count: LocalState<u32>,
    secondary_pulse_count: LocalState<u32>,
    long_press_count: LocalState<u32>,
    drag_count: LocalState<u32>,
    drag_distance: LocalState<f32>,
    autosave_enabled: LocalState<bool>,
    exposure_value: LocalState<f32>,
    review_mode: LocalState<Option<Arc<str>>>,
    draft_note: LocalState<String>,
    timeline_next_id: LocalState<u64>,
    timeline: LocalState<Vec<ShowcaseEvent>>,
    pulse_count_value: u32,
    secondary_pulse_count_value: u32,
    long_press_count_value: u32,
    drag_count_value: u32,
    drag_distance_value: f32,
    autosave_enabled_value: bool,
    exposure_value_value: f32,
    review_mode_value: Option<Arc<str>>,
    draft_note_value: String,
) -> AnyElement {
    let status_row = ui::h_flex(|cx| {
        vec![
            badge(
                cx,
                format!("primary {}", pulse_count_value),
                shadcn::BadgeVariant::Secondary,
            ),
            badge(
                cx,
                format!("secondary {}", secondary_pulse_count_value),
                shadcn::BadgeVariant::Outline,
            ),
            badge(
                cx,
                format!("long {}", long_press_count_value),
                shadcn::BadgeVariant::Outline,
            ),
        ]
    })
    .gap(Space::N1p5)
    .items_center()
    .wrap()
    .w_full()
    .into_element(cx);

    let summary = ui::text(format!(
        "Autosave {}. Exposure {:.0}. Review mode {}. Drag probes {} ({:.0}px).",
        if autosave_enabled_value {
            "armed"
        } else {
            "paused"
        },
        exposure_value_value,
        review_mode_value.as_deref().unwrap_or("none"),
        drag_count_value,
        drag_distance_value,
    ))
    .text_sm()
    .text_color(ColorRef::Color(
        Theme::global(&*cx.app).color_token("muted-foreground"),
    ))
    .wrap(fret_core::TextWrap::Word)
    .into_element(cx);

    let draft_preview = ui::container(move |cx| {
        vec![
            ui::text("Current draft")
                .text_xs()
                .font_semibold()
                .text_color(ColorRef::Color(
                    Theme::global(&*cx.app).color_token("muted-foreground"),
                ))
                .into_element(cx),
            ui::text(draft_note_value)
                .text_sm()
                .wrap(fret_core::TextWrap::Word)
                .into_element(cx),
        ]
    })
    .p_3()
    .rounded_md()
    .border_1()
    .border_color(ColorRef::Color(
        Theme::global(&*cx.app).color_token("border"),
    ))
    .bg(ColorRef::Color(
        Theme::global(&*cx.app).color_token("muted"),
    ))
    .w_full()
    .into_element(cx);

    let body = ui::v_flex(move |cx| {
        vec![
            status_row,
            summary,
            ui::container(move |cx: &mut ElementContext<'_, KernelApp>| {
                fret_imui::imui(cx, move |ui| {
                    use fret_ui_kit::imui::{
                        ComboModelOptions, InputTextOptions, SliderOptions, UiWriterImUiFacadeExt as _,
                        UiWriterUiKitExt as _,
                    };

                    let pulse_count = pulse_count.clone();
                    let secondary_pulse_count = secondary_pulse_count.clone();
                    let long_press_count = long_press_count.clone();
                    let drag_count = drag_count.clone();
                    let drag_distance = drag_distance.clone();
                    let autosave_enabled = autosave_enabled.clone();
                    let exposure_value = exposure_value.clone();
                    let review_mode = review_mode.clone();
                    let draft_note = draft_note.clone();
                    let timeline_next_id = timeline_next_id.clone();
                    let timeline = timeline.clone();

                    let mode_items = [
                        Arc::<str>::from("Studio"),
                        Arc::<str>::from("Review"),
                        Arc::<str>::from("Performance"),
                    ];

                    let headline = ui::text("Interaction lab")
                        .text_sm()
                        .font_semibold()
                        .into_element(ui.cx_mut());
                    ui.add_ui(headline);

                    let hint = ui::text(
                        "Immediate controls stay simple here; the shell around them is what turns the lane into a presentable product surface.",
                    )
                    .text_xs()
                    .wrap(fret_core::TextWrap::Word)
                    .into_element(ui.cx_mut());
                    ui.add_ui(hint);

                    ui.separator_text("Pulse");
                    let pulse = ui.button("Pulse interaction surface");
                    if pulse.clicked() {
                        let _ =
                            pulse_count.update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                        push_showcase_event(
                            ui.cx_mut().app,
                            &timeline_next_id,
                            &timeline,
                            "Primary pulse registered.",
                        );
                    }
                    if pulse.secondary_clicked() {
                        let _ = secondary_pulse_count
                            .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                        push_showcase_event(
                            ui.cx_mut().app,
                            &timeline_next_id,
                            &timeline,
                            "Secondary pulse opened the alternate path.",
                        );
                    }
                    if pulse.long_pressed() {
                        let _ = long_press_count
                            .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                        push_showcase_event(
                            ui.cx_mut().app,
                            &timeline_next_id,
                            &timeline,
                            "Long press crossed the hold threshold.",
                        );
                    }

                    let drag = ui.button("Drag to scrub the stage");
                    if drag.drag_started() {
                        let _ =
                            drag_count.update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                        push_showcase_event(
                            ui.cx_mut().app,
                            &timeline_next_id,
                            &timeline,
                            "Drag probe started.",
                        );
                    }
                    if drag.dragging() {
                        let delta = drag.drag_delta();
                        let _ = drag_distance.update_in(ui.cx_mut().app.models_mut(), |value| {
                            *value += delta.x.0.abs() + delta.y.0.abs();
                        });
                    }

                    ui.separator_text("Controls");
                    let toggle = ui.switch_model("Autosave snapshots", autosave_enabled.model());
                    if toggle.changed() {
                        let label = if autosave_enabled
                            .value_in(ui.cx_mut().app.models())
                            .unwrap_or_default()
                        {
                            "Autosave re-armed."
                        } else {
                            "Autosave paused for experimentation."
                        };
                        push_showcase_event(ui.cx_mut().app, &timeline_next_id, &timeline, label);
                    }

                    let exposure = ui.slider_f32_model_with_options(
                        "Exposure bias",
                        exposure_value.model(),
                        SliderOptions {
                            min: 0.0,
                            max: 100.0,
                            step: 1.0,
                            ..Default::default()
                        },
                    );
                    if exposure.deactivated_after_edit() {
                        let value = exposure_value
                            .value_in(ui.cx_mut().app.models())
                            .unwrap_or_default();
                        push_showcase_event(
                            ui.cx_mut().app,
                            &timeline_next_id,
                            &timeline,
                            Arc::<str>::from(format!("Exposure settled at {:.0}.", value)),
                        );
                    }

                    let combo = ui.combo_model_with_options(
                        "imui-showcase.review-mode",
                        "Review mode",
                        review_mode.model(),
                        &mode_items,
                        ComboModelOptions {
                            placeholder: Some(Arc::from("Choose a review mode")),
                            ..Default::default()
                        },
                    );
                    if combo.deactivated_after_edit() {
                        let mode = review_mode
                            .value_in(ui.cx_mut().app.models())
                            .flatten()
                            .unwrap_or_else(|| Arc::from("none"));
                        push_showcase_event(
                            ui.cx_mut().app,
                            &timeline_next_id,
                            &timeline,
                            Arc::<str>::from(format!("Review mode switched to {mode}.")),
                        );
                    }

                    let notes = ui.input_text_model_with_options(
                        draft_note.model(),
                        InputTextOptions {
                            placeholder: Some(Arc::from("Narrate what this interaction should feel like")),
                            ..Default::default()
                        },
                    );
                    if notes.deactivated_after_edit() {
                        push_showcase_event(
                            ui.cx_mut().app,
                            &timeline_next_id,
                            &timeline,
                            "Draft note committed after blur.",
                        );
                    }
                })
            })
            .w_full()
            .into_element(cx),
            draft_preview,
        ]
    })
    .gap(Space::N3)
    .w_full()
    .into_element(cx);

    showcase_card(
        cx,
        TEST_ID_LAB,
        "SHOWCASE",
        "Immediate interaction lab",
        "Polished shell, direct control flow, and a small state set that still feels reviewable.",
        body,
    )
}

#[allow(clippy::too_many_arguments)]
fn render_shell_showcase_card(
    cx: &mut ElementContext<'_, KernelApp>,
    menu_open_count: LocalState<u32>,
    submenu_toggle_count: LocalState<u32>,
    tab_switch_count: LocalState<u32>,
    context_action_count: LocalState<u32>,
    context_toggle: LocalState<bool>,
    selected_tab: LocalState<Option<Arc<str>>>,
    timeline_next_id: LocalState<u64>,
    timeline: LocalState<Vec<ShowcaseEvent>>,
    menu_open_count_value: u32,
    submenu_toggle_count_value: u32,
    tab_switch_count_value: u32,
    context_action_count_value: u32,
    context_toggle_value: bool,
    selected_tab_value: Option<Arc<str>>,
) -> AnyElement {
    let shell_summary = ui::h_flex(|cx| {
        vec![
            badge(
                cx,
                format!("menus {}", menu_open_count_value),
                shadcn::BadgeVariant::Secondary,
            ),
            badge(
                cx,
                format!("submenus {}", submenu_toggle_count_value),
                shadcn::BadgeVariant::Outline,
            ),
            badge(
                cx,
                format!("tab hops {}", tab_switch_count_value),
                shadcn::BadgeVariant::Outline,
            ),
            badge(
                cx,
                format!("quick actions {}", context_action_count_value),
                if context_toggle_value {
                    shadcn::BadgeVariant::Default
                } else {
                    shadcn::BadgeVariant::Secondary
                },
            ),
        ]
    })
    .gap(Space::N1p5)
    .items_center()
    .wrap()
    .w_full()
    .into_element(cx);

    let body = ui::v_flex(move |cx| {
        vec![
            shell_summary,
            ui::container(move |cx: &mut ElementContext<'_, KernelApp>| {
                fret_imui::imui(cx, move |ui| {
                    use fret_ui_kit::imui::{
                        BeginMenuOptions, BeginSubmenuOptions, MenuBarOptions, MenuItemOptions,
                        TabBarOptions, TabItemOptions, UiWriterImUiFacadeExt as _,
                        UiWriterUiKitExt as _,
                    };

                    let menu_open_count = menu_open_count.clone();
                    let submenu_toggle_count = submenu_toggle_count.clone();
                    let tab_switch_count = tab_switch_count.clone();
                    let context_action_count = context_action_count.clone();
                    let context_toggle = context_toggle.clone();
                    let selected_tab = selected_tab.clone();
                    let timeline_next_id = timeline_next_id.clone();
                    let timeline = timeline.clone();

                    let shell_title = ui::text("Shell preview")
                        .text_sm()
                        .font_semibold()
                        .into_element(ui.cx_mut());
                    ui.add_ui(shell_title);

                    let shell_hint = ui::text(
                        "This is the same helper response surface as the proof demo, just presented as a compact review shell.",
                    )
                    .text_xs()
                    .wrap(fret_core::TextWrap::Word)
                    .into_element(ui.cx_mut());
                    ui.add_ui(shell_hint);

                    ui.menu_bar_with_options(
                        MenuBarOptions {
                            test_id: Some(Arc::from("imui-showcase.menu.root")),
                            ..Default::default()
                        },
                        |ui| {
                            let file_menu = ui.begin_menu_with_options(
                                "imui-showcase.menu.file",
                                "File",
                                BeginMenuOptions {
                                    test_id: Some(Arc::from("imui-showcase.menu.file")),
                                    ..Default::default()
                                },
                                |ui| {
                                    let staging = ui.begin_submenu_with_options(
                                        "imui-showcase.menu.staging",
                                        "Staging",
                                        BeginSubmenuOptions {
                                            test_id: Some(Arc::from("imui-showcase.menu.staging")),
                                            ..Default::default()
                                        },
                                        |ui| {
                                            let _ = ui.menu_item_with_options(
                                                "Capture review frame",
                                                MenuItemOptions::default(),
                                            );
                                            let _ = ui.menu_item_with_options(
                                                "Queue lighting pass",
                                                MenuItemOptions::default(),
                                            );
                                        },
                                    );
                                    if staging.toggled() {
                                        let _ = submenu_toggle_count
                                            .update_in(ui.cx_mut().app.models_mut(), |value| {
                                                *value += 1
                                            });
                                        push_showcase_event(
                                            ui.cx_mut().app,
                                            &timeline_next_id,
                                            &timeline,
                                            "Submenu choreography toggled.",
                                        );
                                    }

                                    let _ = ui.menu_item_with_options(
                                        "Open recent capture",
                                        MenuItemOptions::default(),
                                    );
                                },
                            );
                            if file_menu.opened() {
                                let _ = menu_open_count
                                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                                push_showcase_event(
                                    ui.cx_mut().app,
                                    &timeline_next_id,
                                    &timeline,
                                    "Menu trigger opened with outward response feedback.",
                                );
                            }
                        },
                    );

                    let tabs = ui.tab_bar_with_options(
                        "imui-showcase.tabs",
                        TabBarOptions {
                            selected: Some(selected_tab.model().clone()),
                            test_id: Some(Arc::from("imui-showcase.tabs.root")),
                            ..Default::default()
                        },
                        |tabs| {
                            tabs.begin_tab_item_with_options(
                                "overview",
                                "Overview",
                                TabItemOptions::default(),
                                |ui| {
                                    ui.text(
                                        "Overview keeps the shell-level story compact: menus, tabs, and quick actions work as one visible slice.",
                                    );
                                },
                            );
                            tabs.begin_tab_item_with_options(
                                "scene",
                                "Scene",
                                TabItemOptions::default(),
                                |ui| {
                                    ui.text(
                                        "Scene proves per-trigger response access: click the tab, then inspect the timeline updates.",
                                    );
                                },
                            );
                            tabs.begin_tab_item_with_options(
                                "notes",
                                "Notes",
                                TabItemOptions::default(),
                                |ui| {
                                    ui.text(
                                        "Notes is intentionally plain. The shell chrome should make the story feel usable before recipe-level controls get fancier.",
                                    );
                                },
                            );
                        },
                    );
                    if tabs.selected_changed() {
                        let _ =
                            tab_switch_count.update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                        let selected = selected_tab
                            .value_in(ui.cx_mut().app.models())
                            .flatten()
                            .unwrap_or_else(|| Arc::from("overview"));
                        push_showcase_event(
                            ui.cx_mut().app,
                            &timeline_next_id,
                            &timeline,
                            Arc::<str>::from(format!("Tab focus moved to {selected}.")),
                        );
                    }

                    ui.separator_text("Quick actions");
                    let quick_actions = ui.button("Right-click this review surface");
                    ui.begin_popup_context_menu("imui-showcase.quick-actions", quick_actions, |ui| {
                        let toggle = ui.menu_item_with_options(
                            "Pin diagnostics rail",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-showcase.quick-actions.toggle")),
                                ..Default::default()
                            },
                        );
                        if toggle.clicked() {
                            let _ = context_toggle
                                .update_in(ui.cx_mut().app.models_mut(), |value| *value = !*value);
                            let _ = context_action_count
                                .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                            push_showcase_event(
                                ui.cx_mut().app,
                                &timeline_next_id,
                                &timeline,
                                "Context action flipped the diagnostics rail state.",
                            );
                        }

                        let close_popup = ui.popup_open_model("imui-showcase.quick-actions");
                        let close = ui.menu_item_with_options(
                            "Dismiss quick actions",
                            MenuItemOptions {
                                close_popup: Some(close_popup),
                                ..Default::default()
                            },
                        );
                        if close.clicked() {
                            let _ = context_action_count
                                .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                            push_showcase_event(
                                ui.cx_mut().app,
                                &timeline_next_id,
                                &timeline,
                                "Context surface dismissed cleanly.",
                            );
                        }
                    });

                    ui.child_region("imui-showcase.preview", |ui| {
                        let selected = selected_tab
                            .value_in(ui.cx_mut().app.models())
                            .flatten()
                            .unwrap_or_else(|| Arc::from("overview"));
                        let rail_state = if context_toggle
                            .value_in(ui.cx_mut().app.models())
                            .unwrap_or(false)
                        {
                            "Pinned"
                        } else {
                            "Floating"
                        };

                        ui.separator_text("Preview");
                        ui.text(format!("Active tab: {selected}"));
                        ui.text(format!("Diagnostics rail: {rail_state}"));
                        ui.text("Menu + tab helpers now return canonical outward responses directly.");
                    });
                })
            })
            .w_full()
            .into_element(cx),
        ]
    })
    .gap(Space::N3)
    .w_full()
    .into_element(cx);

    showcase_card(
        cx,
        TEST_ID_SHELL,
        "PRODUCT",
        "Menu, tab, and context choreography",
        "A compact shell slice that demonstrates the cleaned canonical helper names in a usable layout.",
        body,
    )
}

fn render_showcase_hero(
    cx: &mut ElementContext<'_, KernelApp>,
    pulse_count: u32,
    drag_count: u32,
    menu_count: u32,
    tab_switches: u32,
    active_tab: &str,
    latest_event: Option<Arc<str>>,
) -> AnyElement {
    let latest_event = latest_event.unwrap_or_else(|| Arc::from("No timeline events yet."));
    let body = ui::v_flex(move |cx| {
        vec![
            ui::text("IMUI can stay immediate without looking like a diagnostics dump.")
                .text_base()
                .font_semibold()
                .into_element(cx),
            ui::text(
                "This showcase keeps the fast control-flow story, then lifts the shell with card rhythm, badges, and clear panel ownership.",
            )
            .text_sm()
            .text_color(ColorRef::Color(Theme::global(&*cx.app).color_token(
                "muted-foreground",
            )))
            .wrap(fret_core::TextWrap::Word)
            .into_element(cx),
            ui::h_flex(move |cx| {
                vec![
                    badge(cx, format!("pulse {pulse_count}"), shadcn::BadgeVariant::Secondary),
                    badge(cx, format!("drag {drag_count}"), shadcn::BadgeVariant::Outline),
                    badge(cx, format!("menus {menu_count}"), shadcn::BadgeVariant::Outline),
                    badge(
                        cx,
                        format!("tab switches {tab_switches}"),
                        shadcn::BadgeVariant::Outline,
                    ),
                    badge(
                        cx,
                        format!("active {active_tab}"),
                        shadcn::BadgeVariant::Default,
                    ),
                ]
            })
            .gap(Space::N1p5)
            .items_center()
            .wrap()
            .w_full()
            .into_element(cx),
            ui::container(move |cx| {
                vec![
                    ui::text("Latest timeline event")
                        .text_xs()
                        .font_semibold()
                        .text_color(ColorRef::Color(Theme::global(&*cx.app).color_token(
                            "muted-foreground",
                        )))
                        .into_element(cx),
                    ui::text(latest_event)
                        .text_sm()
                        .wrap(fret_core::TextWrap::Word)
                        .into_element(cx),
                ]
            })
            .p_3()
            .rounded_md()
            .bg(ColorRef::Color(Theme::global(&*cx.app).color_token("muted")))
            .border_1()
            .border_color(ColorRef::Color(Theme::global(&*cx.app).color_token("border")))
            .w_full()
            .into_element(cx),
        ]
    })
    .gap(Space::N3)
    .w_full()
    .into_element(cx);

    showcase_card(
        cx,
        TEST_ID_HERO,
        "IMUI SHOWCASE",
        "Immediate interaction, product shell",
        "Keep `imui_response_signals_demo` as the proof surface. Use this one when you need to show what the lane can feel like.",
        body,
    )
}

fn render_timeline_card(
    cx: &mut ElementContext<'_, KernelApp>,
    events: &[ShowcaseEvent],
) -> AnyElement {
    let mut rows = Vec::with_capacity(events.len());
    for event in events {
        let event_id = event.id;
        let label = event.label.clone();
        let row = ui::h_flex(move |cx| {
            vec![
                shadcn::Badge::new(format!("#{event_id}"))
                    .variant(shadcn::BadgeVariant::Outline)
                    .into_element(cx),
                ui::text(label)
                    .text_sm()
                    .wrap(fret_core::TextWrap::Word)
                    .flex_1()
                    .min_w_0()
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_start()
        .w_full()
        .into_element(cx);
        rows.push(row);
    }

    let body = ui::v_flex(move |_cx| rows)
        .gap(Space::N2)
        .w_full()
        .into_element(cx);

    showcase_card(
        cx,
        TEST_ID_TIMELINE,
        "FEEDBACK",
        "Event timeline",
        "A simple audit trail keeps immediate interactions explainable without turning the whole screen into status text.",
        body,
    )
}

fn showcase_card(
    cx: &mut ElementContext<'_, KernelApp>,
    test_id: &'static str,
    eyebrow: &'static str,
    title: &'static str,
    description: &'static str,
    body: AnyElement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let eyebrow = ui::text(eyebrow)
        .text_xs()
        .font_semibold()
        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
        .into_element(cx);
    let header = shadcn::CardHeader::new([
        eyebrow,
        shadcn::CardTitle::new(title).into_element(cx),
        shadcn::CardDescription::new(description).into_element(cx),
    ])
    .into_element(cx);
    let content = shadcn::CardContent::new([body]).into_element(cx);
    shadcn::Card::new([header, content])
        .ui()
        .w_full()
        .into_element(cx)
        .test_id(test_id)
}

fn badge(
    cx: &mut ElementContext<'_, KernelApp>,
    label: impl Into<Arc<str>>,
    variant: shadcn::BadgeVariant,
) -> AnyElement {
    shadcn::Badge::new(label).variant(variant).into_element(cx)
}

fn push_showcase_event(
    app: &mut KernelApp,
    next_id: &LocalState<u64>,
    timeline: &LocalState<Vec<ShowcaseEvent>>,
    label: impl Into<Arc<str>>,
) {
    let id = next_id.value_in_or_default(app.models());
    let _ = next_id.set_in(app.models_mut(), id.saturating_add(1));

    let label = label.into();
    let _ = timeline.update_in(app.models_mut(), |events| {
        events.insert(0, ShowcaseEvent { id, label });
        if events.len() > 8 {
            events.truncate(8);
        }
    });
}
