use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, FrameId, KeyCode, Point, Px, Rect, Size as CoreSize};
use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use fret_runtime::{Effect, Model};
use fret_ui::ElementContext;
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_kit::primitives::dismissable_layer::{DismissReason, OnDismissRequest};
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::facade as shadcn;
use serde::Deserialize;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::{click_at, dispatch_key_press, right_click_at};

#[path = "support/shadcn_motion.rs"]
mod shadcn_motion;

#[path = "support/timers.rs"]
mod timers;
use timers::TimerQueue;

const FOCUS_RESTORE_CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/focus_restore_recipe_cases_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FocusRestoreScenario {
    DialogEscapeRestore,
    PopoverEscapeRestore,
    ComboboxEscapeRestore,
    SelectEscapeRestore,
    DropdownMenuEscapeRestore,
    ComboboxOutsidePressRestore,
    SelectOutsidePressRestore,
    DropdownMenuOutsidePressClearsFocus,
    ContextMenuEscapeClearsFocus,
    ContextMenuOutsidePressFocusesUnderlay,
    DialogOutsidePressRestore,
    PopoverOutsidePressFocusesUnderlay,
    PopoverOutsidePressPreventDefaultFocusesUnderlay,
    PopoverFocusOutsidePreservesUnderlayFocus,
    PopoverFocusOutsidePreventDefaultPreservesUnderlayFocus,
    SelectOutsidePressPreventDefaultBlocksUnderlay,
    DropdownMenuNonModalOutsidePressPreventDefaultFocusesUnderlay,
    DropdownMenuModalOutsidePressPreventDefaultBlocksUnderlay,
    DropdownMenuNonModalFocusOutsidePreservesUnderlayFocus,
    DropdownMenuNonModalFocusOutsidePreventDefaultPreservesUnderlayFocus,
    ContextMenuOutsidePressPreventDefaultFocusesUnderlay,
    ContextMenuFocusOutsidePreservesUnderlayFocus,
    ContextMenuFocusOutsidePreventDefaultPreservesUnderlayFocus,
    DropdownMenuSubmenuKeyboardRestore,
    ContextMenuSubmenuKeyboardRestore,
    MenubarSubmenuKeyboardRestore,
}

#[test]
fn mechanism_harness_focus_restore_recipe_cases_match_oracles() {
    let suite: MechanismSuite<FocusRestoreScenario> =
        MechanismSuite::from_json_str(FOCUS_RESTORE_CASES)
            .expect("focus restore recipe fixture suite");

    let mut observer: fn(
        &MechanismCase<FocusRestoreScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<FocusRestoreScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match case.scenario {
        FocusRestoreScenario::DialogEscapeRestore => observe_dialog_escape_restore(),
        FocusRestoreScenario::PopoverEscapeRestore => observe_popover_escape_restore(),
        FocusRestoreScenario::ComboboxEscapeRestore => observe_combobox_escape_restore(),
        FocusRestoreScenario::SelectEscapeRestore => observe_select_escape_restore(),
        FocusRestoreScenario::DropdownMenuEscapeRestore => observe_dropdown_menu_escape_restore(),
        FocusRestoreScenario::ComboboxOutsidePressRestore => {
            observe_combobox_outside_press_restore()
        }
        FocusRestoreScenario::SelectOutsidePressRestore => observe_select_outside_press_restore(),
        FocusRestoreScenario::DropdownMenuOutsidePressClearsFocus => {
            observe_dropdown_menu_outside_press_clears_focus()
        }
        FocusRestoreScenario::ContextMenuEscapeClearsFocus => {
            observe_context_menu_escape_clears_focus()
        }
        FocusRestoreScenario::ContextMenuOutsidePressFocusesUnderlay => {
            observe_context_menu_outside_press_focuses_underlay()
        }
        FocusRestoreScenario::DialogOutsidePressRestore => observe_dialog_outside_press_restore(),
        FocusRestoreScenario::PopoverOutsidePressFocusesUnderlay => {
            observe_popover_outside_press_focuses_underlay()
        }
        FocusRestoreScenario::PopoverOutsidePressPreventDefaultFocusesUnderlay => {
            observe_popover_outside_press_prevent_default_focuses_underlay()
        }
        FocusRestoreScenario::PopoverFocusOutsidePreservesUnderlayFocus => {
            observe_popover_focus_outside_preserves_underlay_focus()
        }
        FocusRestoreScenario::PopoverFocusOutsidePreventDefaultPreservesUnderlayFocus => {
            observe_popover_focus_outside_prevent_default_preserves_underlay_focus()
        }
        FocusRestoreScenario::SelectOutsidePressPreventDefaultBlocksUnderlay => {
            observe_select_outside_press_prevent_default_blocks_underlay()
        }
        FocusRestoreScenario::DropdownMenuNonModalOutsidePressPreventDefaultFocusesUnderlay => {
            observe_dropdown_menu_non_modal_outside_press_prevent_default_focuses_underlay()
        }
        FocusRestoreScenario::DropdownMenuModalOutsidePressPreventDefaultBlocksUnderlay => {
            observe_dropdown_menu_modal_outside_press_prevent_default_blocks_underlay()
        }
        FocusRestoreScenario::DropdownMenuNonModalFocusOutsidePreservesUnderlayFocus => {
            observe_dropdown_menu_non_modal_focus_outside_preserves_underlay_focus()
        }
        FocusRestoreScenario::DropdownMenuNonModalFocusOutsidePreventDefaultPreservesUnderlayFocus => {
            observe_dropdown_menu_non_modal_focus_outside_prevent_default_preserves_underlay_focus()
        }
        FocusRestoreScenario::ContextMenuOutsidePressPreventDefaultFocusesUnderlay => {
            observe_context_menu_outside_press_prevent_default_focuses_underlay()
        }
        FocusRestoreScenario::ContextMenuFocusOutsidePreservesUnderlayFocus => {
            observe_context_menu_focus_outside_preserves_underlay_focus()
        }
        FocusRestoreScenario::ContextMenuFocusOutsidePreventDefaultPreservesUnderlayFocus => {
            observe_context_menu_focus_outside_prevent_default_preserves_underlay_focus()
        }
        FocusRestoreScenario::DropdownMenuSubmenuKeyboardRestore => {
            observe_dropdown_menu_submenu_keyboard_restore()
        }
        FocusRestoreScenario::ContextMenuSubmenuKeyboardRestore => {
            observe_context_menu_submenu_keyboard_restore()
        }
        FocusRestoreScenario::MenubarSubmenuKeyboardRestore => {
            observe_menubar_submenu_keyboard_restore()
        }
    }
}

fn observe_dialog_escape_restore() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_dialog_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    click_trigger(&mut ui, &mut app, &mut services, "dialog-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_200() + 2,
            open.clone(),
        );
    }

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_dialog_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    observed_focus_restore_tree(&ui, &app, &open, bounds)
}

fn observe_dialog_outside_press_restore() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_dialog_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    click_trigger(&mut ui, &mut app, &mut services, "dialog-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_dialog_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
        );
    }

    click_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(Px(2.0), Px(2.0)),
    );
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_dialog_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    observed_focus_restore_tree(&ui, &app, &open, bounds)
}

fn observe_popover_escape_restore() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_popover_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    click_trigger(&mut ui, &mut app, &mut services, "popover-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
        );
    }

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
        );
    }
    observed_focus_restore_tree(&ui, &app, &open, bounds)
}

fn observe_popover_outside_press_focuses_underlay() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_popover_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        underlay_activated.clone(),
    );
    click_trigger(&mut ui, &mut app, &mut services, "popover-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_popover_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            underlay_activated.clone(),
        );
    }

    click_trigger(&mut ui, &mut app, &mut services, "underlay")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_popover_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            underlay_activated.clone(),
        );
    }

    let mut observed = observed_focus_restore_tree(&ui, &app, &open, bounds)?;
    observed.set_metric(
        "underlay.activated",
        if app.models().get_copied(&underlay_activated) == Some(true) {
            1.0
        } else {
            0.0
        },
    );
    Ok(observed)
}

fn observe_popover_outside_press_prevent_default_focuses_underlay()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let dismiss_calls: Model<u32> = app.models_mut().insert(0);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let handler = prevent_outside_press_handler(dismiss_calls.clone());
    render_popover_with_underlay_frame_with_dismiss_handler(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        underlay_activated.clone(),
        Some(handler.clone()),
    );
    let underlay_point = point_for_test_id(&ui, "underlay")?;
    click_trigger(&mut ui, &mut app, &mut services, "popover-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_popover_with_underlay_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            underlay_activated.clone(),
            Some(handler.clone()),
        );
    }

    click_at_with_pointer_id(&mut ui, &mut app, &mut services, 1, underlay_point);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_popover_with_underlay_frame_with_dismiss_handler(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        underlay_activated.clone(),
        Some(handler),
    );

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    set_u32_model_metric(&mut observed, &app, &dismiss_calls, "dismiss.calls");
    Ok(observed)
}

fn observe_popover_focus_outside_preserves_underlay_focus()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_popover_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        underlay_activated.clone(),
    );
    click_trigger(&mut ui, &mut app, &mut services, "popover-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_popover_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            underlay_activated.clone(),
        );
    }

    focus_test_id(&mut ui, "underlay")?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_popover_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            underlay_activated.clone(),
        );
    }

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    Ok(observed)
}

fn observe_popover_focus_outside_prevent_default_preserves_underlay_focus()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let dismiss_calls: Model<u32> = app.models_mut().insert(0);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let handler = prevent_focus_outside_handler(dismiss_calls.clone());
    render_popover_with_underlay_frame_with_dismiss_handler(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        underlay_activated.clone(),
        Some(handler.clone()),
    );
    click_trigger(&mut ui, &mut app, &mut services, "popover-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_popover_with_underlay_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            underlay_activated.clone(),
            Some(handler.clone()),
        );
    }

    focus_test_id(&mut ui, "underlay")?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_popover_with_underlay_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            underlay_activated.clone(),
            Some(handler.clone()),
        );
    }

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    set_u32_model_metric(&mut observed, &app, &dismiss_calls, "dismiss.calls");
    Ok(observed)
}

fn observe_combobox_escape_restore() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_combobox_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value.clone(),
        open.clone(),
    );
    click_trigger(&mut ui, &mut app, &mut services, "combobox-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_combobox_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            value.clone(),
            open.clone(),
        );
    }

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_combobox_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value,
        open.clone(),
    );
    observed_focus_restore_tree(&ui, &app, &open, bounds)
}

fn observe_select_escape_restore() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_select_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value.clone(),
        open.clone(),
    );
    click_trigger(&mut ui, &mut app, &mut services, "select-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_select_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            value.clone(),
            open.clone(),
        );
    }

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_select_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value,
        open.clone(),
    );
    observed_focus_restore_tree(&ui, &app, &open, bounds)
}

fn observe_dropdown_menu_escape_restore() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_dropdown_menu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    click_trigger(&mut ui, &mut app, &mut services, "menu-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_dropdown_menu_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
        );
    }

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_dropdown_menu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    observed_focus_restore_tree(&ui, &app, &open, bounds)
}

fn observe_combobox_outside_press_restore() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_combobox_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value.clone(),
        open.clone(),
        underlay_activated.clone(),
    );
    click_trigger(&mut ui, &mut app, &mut services, "combobox-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_combobox_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            value.clone(),
            open.clone(),
            underlay_activated.clone(),
        );
    }

    click_trigger(&mut ui, &mut app, &mut services, "underlay")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_combobox_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            value.clone(),
            open.clone(),
            underlay_activated.clone(),
        );
    }

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    Ok(observed)
}

fn observe_select_outside_press_restore() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_select_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value.clone(),
        open.clone(),
        underlay_activated.clone(),
    );
    let underlay_point = point_for_test_id(&ui, "underlay")?;
    click_trigger(&mut ui, &mut app, &mut services, "select-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_200() + 2) {
        render_select_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            value.clone(),
            open.clone(),
            underlay_activated.clone(),
        );
    }

    click_at_with_pointer_id(&mut ui, &mut app, &mut services, 1, underlay_point);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_select_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value,
        open.clone(),
        underlay_activated.clone(),
    );

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    Ok(observed)
}

fn observe_select_outside_press_prevent_default_blocks_underlay()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let dismiss_calls: Model<u32> = app.models_mut().insert(0);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let handler = prevent_outside_press_handler(dismiss_calls.clone());
    render_select_with_underlay_frame_with_dismiss_handler(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value.clone(),
        open.clone(),
        underlay_activated.clone(),
        Some(handler.clone()),
    );
    let underlay_point = point_for_test_id(&ui, "underlay")?;
    click_trigger(&mut ui, &mut app, &mut services, "select-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_200() + 2) {
        render_select_with_underlay_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            value.clone(),
            open.clone(),
            underlay_activated.clone(),
            Some(handler.clone()),
        );
    }

    click_at_with_pointer_id(&mut ui, &mut app, &mut services, 1, underlay_point);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_select_with_underlay_frame_with_dismiss_handler(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        value,
        open.clone(),
        underlay_activated.clone(),
        Some(handler),
    );

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    set_u32_model_metric(&mut observed, &app, &dismiss_calls, "dismiss.calls");
    Ok(observed)
}

fn observe_dropdown_menu_outside_press_clears_focus() -> Result<ObservedTree, ScenarioObserveError>
{
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_dropdown_menu_non_modal_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    click_trigger(&mut ui, &mut app, &mut services, "menu-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_dropdown_menu_non_modal_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
        );
    }

    click_at_with_pointer_id(
        &mut ui,
        &mut app,
        &mut services,
        1,
        Point::new(Px(620.0), Px(460.0)),
    );
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_dropdown_menu_non_modal_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    observed_focus_policy_tree(&ui, &app, &open, bounds)
}

fn observe_dropdown_menu_non_modal_outside_press_prevent_default_focuses_underlay()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let dismiss_calls: Model<u32> = app.models_mut().insert(0);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let handler = prevent_outside_press_handler(dismiss_calls.clone());
    render_dropdown_menu_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        false,
        underlay_activated.clone(),
        Some(handler.clone()),
    );
    let underlay_point = point_for_test_id(&ui, "underlay")?;
    click_trigger(&mut ui, &mut app, &mut services, "menu-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_dropdown_menu_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            false,
            underlay_activated.clone(),
            Some(handler.clone()),
        );
    }

    click_at_with_pointer_id(&mut ui, &mut app, &mut services, 1, underlay_point);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_dropdown_menu_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        false,
        underlay_activated.clone(),
        Some(handler),
    );

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    set_u32_model_metric(&mut observed, &app, &dismiss_calls, "dismiss.calls");
    Ok(observed)
}

fn observe_dropdown_menu_modal_outside_press_prevent_default_blocks_underlay()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let dismiss_calls: Model<u32> = app.models_mut().insert(0);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let handler = prevent_outside_press_handler(dismiss_calls.clone());
    render_dropdown_menu_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        true,
        underlay_activated.clone(),
        Some(handler.clone()),
    );
    let underlay_point = point_for_test_id(&ui, "underlay")?;
    click_trigger(&mut ui, &mut app, &mut services, "menu-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_dropdown_menu_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            true,
            underlay_activated.clone(),
            Some(handler.clone()),
        );
    }

    click_at_with_pointer_id(&mut ui, &mut app, &mut services, 1, underlay_point);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_dropdown_menu_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        true,
        underlay_activated.clone(),
        Some(handler),
    );

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    set_u32_model_metric(&mut observed, &app, &dismiss_calls, "dismiss.calls");
    Ok(observed)
}

fn observe_dropdown_menu_non_modal_focus_outside_preserves_underlay_focus()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_dropdown_menu_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        false,
        underlay_activated.clone(),
        None,
    );
    click_trigger(&mut ui, &mut app, &mut services, "menu-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_dropdown_menu_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            false,
            underlay_activated.clone(),
            None,
        );
    }

    focus_test_id(&mut ui, "underlay")?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_dropdown_menu_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            false,
            underlay_activated.clone(),
            None,
        );
    }

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    Ok(observed)
}

fn observe_dropdown_menu_non_modal_focus_outside_prevent_default_preserves_underlay_focus()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let dismiss_calls: Model<u32> = app.models_mut().insert(0);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let handler = prevent_focus_outside_handler(dismiss_calls.clone());
    render_dropdown_menu_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        false,
        underlay_activated.clone(),
        Some(handler.clone()),
    );
    click_trigger(&mut ui, &mut app, &mut services, "menu-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_dropdown_menu_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            false,
            underlay_activated.clone(),
            Some(handler.clone()),
        );
    }

    focus_test_id(&mut ui, "underlay")?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_dropdown_menu_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            false,
            underlay_activated.clone(),
            Some(handler.clone()),
        );
    }

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    set_u32_model_metric(&mut observed, &app, &dismiss_calls, "dismiss.calls");
    Ok(observed)
}

fn observe_context_menu_escape_clears_focus() -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_context_menu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    right_click_trigger(&mut ui, &mut app, &mut services, "context-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_context_menu_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
        );
    }

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_context_menu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );
    observed_focus_policy_tree(&ui, &app, &open, bounds)
}

fn observe_context_menu_outside_press_focuses_underlay()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_context_menu_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        underlay_activated.clone(),
    );
    let underlay_point = point_for_test_id(&ui, "underlay")?;
    right_click_trigger(&mut ui, &mut app, &mut services, "context-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_context_menu_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            underlay_activated.clone(),
        );
    }

    click_at_with_pointer_id(&mut ui, &mut app, &mut services, 1, underlay_point);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_context_menu_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        underlay_activated.clone(),
    );

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    Ok(observed)
}

fn observe_context_menu_outside_press_prevent_default_focuses_underlay()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let dismiss_calls: Model<u32> = app.models_mut().insert(0);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let handler = prevent_outside_press_handler(dismiss_calls.clone());
    render_context_menu_with_underlay_frame_with_dismiss_handler(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        false,
        underlay_activated.clone(),
        Some(handler.clone()),
    );
    let underlay_point = point_for_test_id(&ui, "underlay")?;
    right_click_trigger(&mut ui, &mut app, &mut services, "context-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_context_menu_with_underlay_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            false,
            underlay_activated.clone(),
            Some(handler.clone()),
        );
    }

    click_at_with_pointer_id(&mut ui, &mut app, &mut services, 1, underlay_point);
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);

    render_context_menu_with_underlay_frame_with_dismiss_handler(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        false,
        underlay_activated.clone(),
        Some(handler),
    );

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    set_u32_model_metric(&mut observed, &app, &dismiss_calls, "dismiss.calls");
    Ok(observed)
}

fn observe_context_menu_focus_outside_preserves_underlay_focus()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    render_context_menu_with_underlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        underlay_activated.clone(),
    );
    right_click_trigger(&mut ui, &mut app, &mut services, "context-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_context_menu_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            underlay_activated.clone(),
        );
    }

    focus_test_id(&mut ui, "underlay")?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_context_menu_with_underlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            underlay_activated.clone(),
        );
    }

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    Ok(observed)
}

fn observe_context_menu_focus_outside_prevent_default_preserves_underlay_focus()
-> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let underlay_activated: Model<bool> = app.models_mut().insert(false);
    let dismiss_calls: Model<u32> = app.models_mut().insert(0);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let handler = prevent_focus_outside_handler(dismiss_calls.clone());
    render_context_menu_with_underlay_frame_with_dismiss_handler(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
        false,
        underlay_activated.clone(),
        Some(handler.clone()),
    );
    right_click_trigger(&mut ui, &mut app, &mut services, "context-trigger")?;
    flush_timers(&mut ui, &mut app, &mut services, &mut timers);
    expect_open(&app, &open, true)?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_context_menu_with_underlay_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            false,
            underlay_activated.clone(),
            Some(handler.clone()),
        );
    }

    focus_test_id(&mut ui, "underlay")?;

    for tick in 0..(shadcn_motion::ticks_100() + 2) {
        render_context_menu_with_underlay_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == shadcn_motion::ticks_100() + 2,
            open.clone(),
            false,
            underlay_activated.clone(),
            Some(handler.clone()),
        );
    }

    let mut observed = observed_focus_policy_tree(&ui, &app, &open, bounds)?;
    set_bool_model_metric(
        &mut observed,
        &app,
        &underlay_activated,
        "underlay.activated",
    );
    set_u32_model_metric(&mut observed, &app, &dismiss_calls, "dismiss.calls");
    Ok(observed)
}

fn observe_dropdown_menu_submenu_keyboard_restore() -> Result<ObservedTree, ScenarioObserveError> {
    use std::time::Duration;

    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_dropdown_menu_submenu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        open.clone(),
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    expect_open(&app, &open, true)?;

    render_dropdown_menu_submenu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|node| {
            node.role == fret_core::SemanticsRole::MenuItem
                && node.test_id.as_deref() == Some("submenu-trigger")
        })
        .expect("submenu trigger");
    ui.set_focus(Some(submenu_trigger.id));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    render_dropdown_menu_submenu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        open.clone(),
    );

    let effects = app.flush_effects();
    let focus_timer = effects.iter().find_map(|effect| match effect {
        Effect::SetTimer { token, after, .. } if *after <= Duration::from_millis(250) => {
            Some(*token)
        }
        _ => None,
    });
    let Some(focus_timer) = focus_timer else {
        panic!("expected submenu focus timer effect");
    };

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Timer { token: focus_timer },
    );

    render_dropdown_menu_submenu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let submenu_first = snap
        .nodes
        .iter()
        .find(|node| {
            node.role == fret_core::SemanticsRole::MenuItem
                && node.test_id.as_deref() == Some("submenu-first-item")
        })
        .expect("submenu first item");
    assert_eq!(
        ui.focus(),
        Some(submenu_first.id),
        "expected keyboard-open to transfer focus into the submenu"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: KeyCode::ArrowLeft,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    render_dropdown_menu_submenu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        open.clone(),
    );

    let settle_frames = shadcn_motion::ticks_for_duration(Duration::from_millis(150));
    for tick in 0..settle_frames {
        render_dropdown_menu_submenu_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == settle_frames,
            open.clone(),
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let submenu_trigger_after_close = snap
        .nodes
        .iter()
        .find(|node| {
            node.role == fret_core::SemanticsRole::MenuItem
                && node.test_id.as_deref() == Some("submenu-trigger")
        })
        .expect("submenu trigger after close");
    assert_eq!(
        ui.focus(),
        Some(submenu_trigger_after_close.id),
        "ArrowLeft should restore focus to the submenu trigger"
    );

    let mut observed = observed_focus_restore_tree(&ui, &app, &open, bounds)?;
    observed.set_metric("submenu.opened", 1.0);
    observed.set_metric(
        "submenu.closed",
        if snap.nodes.iter().any(|node| {
            node.role == fret_core::SemanticsRole::MenuItem
                && node.test_id.as_deref() == Some("submenu-first-item")
        }) {
            0.0
        } else {
            1.0
        },
    );
    Ok(observed)
}

fn observe_context_menu_submenu_keyboard_restore() -> Result<ObservedTree, ScenarioObserveError> {
    use std::time::Duration;

    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(false);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_context_menu_submenu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        open.clone(),
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    expect_open(&app, &open, true)?;

    render_context_menu_submenu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|node| {
            node.role == fret_core::SemanticsRole::MenuItem
                && node.test_id.as_deref() == Some("submenu-trigger")
        })
        .expect("submenu trigger");
    ui.set_focus(Some(submenu_trigger.id));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    render_context_menu_submenu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        open.clone(),
    );

    let effects = app.flush_effects();
    let focus_timer = effects.iter().find_map(|effect| match effect {
        Effect::SetTimer { token, after, .. } if *after <= Duration::from_millis(250) => {
            Some(*token)
        }
        _ => None,
    });
    let Some(focus_timer) = focus_timer else {
        panic!("expected submenu focus timer effect");
    };

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Timer { token: focus_timer },
    );

    render_context_menu_submenu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        open.clone(),
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let submenu_first = snap
        .nodes
        .iter()
        .find(|node| {
            node.role == fret_core::SemanticsRole::MenuItem
                && node.test_id.as_deref() == Some("submenu-first-item")
        })
        .expect("submenu first item");
    assert_eq!(
        ui.focus(),
        Some(submenu_first.id),
        "expected keyboard-open to transfer focus into the submenu"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: KeyCode::ArrowLeft,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    render_context_menu_submenu_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        open.clone(),
    );

    let settle_frames = shadcn_motion::ticks_for_duration(Duration::from_millis(150));
    for tick in 0..settle_frames {
        render_context_menu_submenu_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == settle_frames,
            open.clone(),
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let submenu_trigger_after_close = snap
        .nodes
        .iter()
        .find(|node| {
            node.role == fret_core::SemanticsRole::MenuItem
                && node.test_id.as_deref() == Some("submenu-trigger")
        })
        .expect("submenu trigger after close");
    assert_eq!(
        ui.focus(),
        Some(submenu_trigger_after_close.id),
        "ArrowLeft should restore focus to the submenu trigger"
    );

    let mut observed = observed_focus_restore_tree(&ui, &app, &open, bounds)?;
    observed.set_metric("submenu.opened", 1.0);
    observed.set_metric(
        "submenu.closed",
        if snap.nodes.iter().any(|node| {
            node.role == fret_core::SemanticsRole::MenuItem
                && node.test_id.as_deref() == Some("submenu-first-item")
        }) {
            0.0
        } else {
            1.0
        },
    );
    Ok(observed)
}

fn observe_menubar_submenu_keyboard_restore() -> Result<ObservedTree, ScenarioObserveError> {
    use std::time::Duration;

    let window = AppWindowId::default();
    let bounds = default_bounds();
    let mut app = themed_app();
    let open: Model<bool> = app.models_mut().insert(true);
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_menubar_submenu_frame(&mut ui, &mut app, &mut services, window, bounds, false);
    expect_open(&app, &open, true)?;

    render_menubar_submenu_frame(&mut ui, &mut app, &mut services, window, bounds, true);

    let file_point = point_for_label(&ui, "File")?;
    click_at(&mut ui, &mut app, &mut services, file_point);

    render_menubar_submenu_frame(&mut ui, &mut app, &mut services, window, bounds, true);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let more = snap
        .nodes
        .iter()
        .find(|node| {
            node.role == fret_core::SemanticsRole::MenuItem && node.label.as_deref() == Some("More")
        })
        .expect("More submenu trigger");
    ui.set_focus(Some(more.id));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    render_menubar_submenu_frame(&mut ui, &mut app, &mut services, window, bounds, false);

    let effects = app.flush_effects();
    let focus_timer = effects.iter().find_map(|effect| match effect {
        Effect::SetTimer { token, after, .. } if *after <= Duration::from_millis(250) => {
            Some(*token)
        }
        _ => None,
    });
    let Some(focus_timer) = focus_timer else {
        panic!("expected submenu focus timer effect");
    };

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Timer { token: focus_timer },
    );

    render_menubar_submenu_frame(&mut ui, &mut app, &mut services, window, bounds, true);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let submenu_first = snap
        .nodes
        .iter()
        .find(|node| {
            node.role == fret_core::SemanticsRole::MenuItem
                && node.test_id.as_deref() == Some("submenu-first-item")
        })
        .expect("submenu first item");
    assert_eq!(
        ui.focus(),
        Some(submenu_first.id),
        "expected keyboard-open to transfer focus into the submenu"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: KeyCode::ArrowLeft,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    render_menubar_submenu_frame(&mut ui, &mut app, &mut services, window, bounds, false);

    let settle_frames = shadcn_motion::ticks_for_duration(Duration::from_millis(150));
    for tick in 0..settle_frames {
        render_menubar_submenu_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            tick + 1 == settle_frames,
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let submenu_trigger_after_close = snap
        .nodes
        .iter()
        .find(|node| {
            node.role == fret_core::SemanticsRole::MenuItem
                && node.test_id.as_deref() == Some("submenu-trigger")
        })
        .expect("submenu trigger after close");
    assert_eq!(
        ui.focus(),
        Some(submenu_trigger_after_close.id),
        "ArrowLeft should restore focus to the submenu trigger"
    );

    let mut observed = observed_focus_restore_tree(&ui, &app, &open, bounds)?;
    observed.set_metric("submenu.opened", 1.0);
    observed.set_metric(
        "submenu.closed",
        if snap.nodes.iter().any(|node| {
            node.role == fret_core::SemanticsRole::MenuItem
                && node.test_id.as_deref() == Some("submenu-first-item")
        }) {
            0.0
        } else {
            1.0
        },
    );
    Ok(observed)
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    name: &str,
    request_semantics: bool,
    root: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);
    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, name, root);
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn render_dialog_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-dialog",
        request_semantics,
        move |cx| {
            vec![shadcn::Dialog::new(open).into_element_parts(
                cx,
                |cx| {
                    shadcn::DialogTrigger::new(
                        shadcn::Button::new("Open")
                            .test_id("dialog-trigger")
                            .into_element(cx),
                    )
                },
                shadcn::DialogPortal::new(),
                shadcn::DialogOverlay::new(),
                |cx| shadcn::DialogContent::new([cx.text("Content")]).into_element(cx),
            )]
        },
    );
}

fn render_popover_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-popover",
        request_semantics,
        move |cx| {
            vec![shadcn::Popover::from_open(open).into_element_with(
                cx,
                |cx| {
                    shadcn::PopoverTrigger::new(
                        shadcn::Button::new("Open")
                            .test_id("popover-trigger")
                            .into_element(cx),
                    )
                    .into_element(cx)
                },
                |cx| shadcn::PopoverContent::new([cx.text("Content")]).into_element(cx),
            )]
        },
    );
}

fn render_popover_with_underlay_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
    underlay_activated: Model<bool>,
) {
    render_popover_with_underlay_frame_with_dismiss_handler(
        ui,
        app,
        services,
        window,
        bounds,
        request_semantics,
        open,
        underlay_activated,
        None,
    );
}

fn render_popover_with_underlay_frame_with_dismiss_handler(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
    underlay_activated: Model<bool>,
    on_dismiss_request: Option<OnDismissRequest>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-popover-outside",
        request_semantics,
        move |cx| {
            let on_underlay_activate: OnActivate = Arc::new(move |host, acx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&underlay_activated, |activated| *activated = true);
                host.request_redraw(acx.window);
            });
            let underlay = shadcn::Button::new("Underlay")
                .test_id("underlay")
                .on_activate(on_underlay_activate)
                .refine_layout(
                    LayoutRefinement::default()
                        .absolute()
                        .right(Space::N4)
                        .bottom(Space::N4),
                )
                .into_element(cx);
            let popover = shadcn::Popover::from_open(open)
                .on_dismiss_request(on_dismiss_request)
                .into_element_with(
                    cx,
                    |cx| {
                        shadcn::PopoverTrigger::new(
                            shadcn::Button::new("Open")
                                .test_id("popover-trigger")
                                .into_element(cx),
                        )
                        .into_element(cx)
                    },
                    |cx| shadcn::PopoverContent::new([cx.text("Content")]).into_element(cx),
                );
            vec![underlay, popover]
        },
    );
}

fn underlay_button(
    cx: &mut ElementContext<'_, App>,
    underlay_activated: Model<bool>,
) -> AnyElement {
    let on_underlay_activate: OnActivate = Arc::new(move |host, acx, _reason| {
        let _ = host
            .models_mut()
            .update(&underlay_activated, |activated| *activated = true);
        host.request_redraw(acx.window);
    });
    shadcn::Button::new("Underlay")
        .test_id("underlay")
        .on_activate(on_underlay_activate)
        .refine_layout(
            LayoutRefinement::default()
                .absolute()
                .right(Space::N4)
                .bottom(Space::N4),
        )
        .into_element(cx)
}

fn render_combobox_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-combobox",
        request_semantics,
        move |cx| {
            let items = [
                shadcn::ComboboxItem::new("next", "Next.js"),
                shadcn::ComboboxItem::new("svelte", "SvelteKit"),
                shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            ];

            vec![
                shadcn::Combobox::new(value, open)
                    .a11y_label("Combobox")
                    .trigger_test_id("combobox-trigger")
                    .test_id_prefix("combobox-test")
                    .into_element_parts(cx, |_cx| {
                        vec![
                            shadcn::ComboboxPart::from(
                                shadcn::ComboboxInput::new().placeholder("Select a framework"),
                            ),
                            shadcn::ComboboxPart::from(shadcn::ComboboxContent::new([
                                shadcn::ComboboxContentPart::from(shadcn::ComboboxEmpty::new(
                                    "No items found.",
                                )),
                                shadcn::ComboboxContentPart::from(
                                    shadcn::ComboboxList::new().items(items),
                                ),
                            ])),
                        ]
                    }),
            ]
        },
    );
}

fn render_combobox_with_underlay_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    underlay_activated: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-combobox-outside",
        request_semantics,
        move |cx| {
            let underlay = underlay_button(cx, underlay_activated);
            let items = [
                shadcn::ComboboxItem::new("next", "Next.js"),
                shadcn::ComboboxItem::new("svelte", "SvelteKit"),
                shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            ];
            let combobox = shadcn::Combobox::new(value, open)
                .a11y_label("Combobox")
                .trigger_test_id("combobox-trigger")
                .test_id_prefix("combobox-test")
                .into_element_parts(cx, |_cx| {
                    vec![
                        shadcn::ComboboxPart::from(
                            shadcn::ComboboxInput::new().placeholder("Select a framework"),
                        ),
                        shadcn::ComboboxPart::from(shadcn::ComboboxContent::new([
                            shadcn::ComboboxContentPart::from(shadcn::ComboboxEmpty::new(
                                "No items found.",
                            )),
                            shadcn::ComboboxContentPart::from(
                                shadcn::ComboboxList::new().items(items),
                            ),
                        ])),
                    ]
                });
            vec![underlay, combobox]
        },
    );
}

fn render_select_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-select",
        request_semantics,
        move |cx| {
            let items: Vec<shadcn::SelectItem> = ["apple", "banana", "blueberry"]
                .into_iter()
                .map(|value| shadcn::SelectItem::new(value, value))
                .collect();
            vec![
                shadcn::Select::new(value, open)
                    .value(shadcn::SelectValue::new().placeholder("Select"))
                    .a11y_label("Select")
                    .trigger_test_id("select-trigger")
                    .items(items)
                    .into_element(cx),
            ]
        },
    );
}

fn render_select_with_underlay_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    underlay_activated: Model<bool>,
) {
    render_select_with_underlay_frame_with_dismiss_handler(
        ui,
        app,
        services,
        window,
        bounds,
        request_semantics,
        value,
        open,
        underlay_activated,
        None,
    );
}

fn render_select_with_underlay_frame_with_dismiss_handler(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    underlay_activated: Model<bool>,
    on_dismiss_request: Option<OnDismissRequest>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-select-outside",
        request_semantics,
        move |cx| {
            let underlay = underlay_button(cx, underlay_activated);
            let items: Vec<shadcn::SelectItem> = ["apple", "banana", "blueberry"]
                .into_iter()
                .map(|value| shadcn::SelectItem::new(value, value))
                .collect();
            let select = shadcn::Select::new(value, open)
                .value(shadcn::SelectValue::new().placeholder("Select"))
                .a11y_label("Select")
                .trigger_test_id("select-trigger")
                .on_dismiss_request(on_dismiss_request)
                .items(items)
                .into_element(cx);
            vec![underlay, select]
        },
    );
}

fn render_dropdown_menu_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-dropdown-menu",
        request_semantics,
        move |cx| {
            vec![shadcn::DropdownMenu::from_open(open).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open")
                        .test_id("menu-trigger")
                        .into_element(cx)
                },
                |_cx| {
                    vec![
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("My Account").value("my-account"),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Profile").value("profile"),
                        ),
                    ]
                },
            )]
        },
    );
}

fn render_dropdown_menu_submenu_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-dropdown-menu-submenu",
        request_semantics,
        move |cx| {
            vec![shadcn::DropdownMenu::from_open(open).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open")
                        .test_id("menu-trigger")
                        .into_element(cx)
                },
                |_cx| {
                    vec![shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("More")
                            .test_id("submenu-trigger")
                            .submenu(vec![
                                shadcn::DropdownMenuEntry::Item(
                                    shadcn::DropdownMenuItem::new("Sub Alpha")
                                        .test_id("submenu-first-item"),
                                ),
                                shadcn::DropdownMenuEntry::Item(
                                    shadcn::DropdownMenuItem::new("Sub Beta")
                                        .test_id("submenu-second-item"),
                                ),
                            ]),
                    )]
                },
            )]
        },
    );
}

fn render_dropdown_menu_non_modal_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-dropdown-menu-outside",
        request_semantics,
        move |cx| {
            vec![
                shadcn::DropdownMenu::from_open(open)
                    .modal(false)
                    .into_element(
                        cx,
                        |cx| {
                            shadcn::Button::new("Open")
                                .test_id("menu-trigger")
                                .into_element(cx)
                        },
                        |_cx| {
                            vec![
                                shadcn::DropdownMenuEntry::Item(
                                    shadcn::DropdownMenuItem::new("My Account").value("my-account"),
                                ),
                                shadcn::DropdownMenuEntry::Item(
                                    shadcn::DropdownMenuItem::new("Profile").value("profile"),
                                ),
                            ]
                        },
                    ),
            ]
        },
    );
}

fn render_dropdown_menu_with_underlay_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
    modal: bool,
    underlay_activated: Model<bool>,
    on_dismiss_request: Option<OnDismissRequest>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-dropdown-menu-outside-underlay",
        request_semantics,
        move |cx| {
            let underlay = underlay_button(cx, underlay_activated);
            let menu = shadcn::DropdownMenu::from_open(open)
                .modal(modal)
                .on_dismiss_request(on_dismiss_request)
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Open")
                            .test_id("menu-trigger")
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("My Account").value("my-account"),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Profile").value("profile"),
                            ),
                        ]
                    },
                );
            vec![underlay, menu]
        },
    );
}

fn render_context_menu_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-context-menu",
        request_semantics,
        move |cx| {
            vec![shadcn::ContextMenu::from_open(open).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Right click")
                        .test_id("context-trigger")
                        .into_element(cx)
                },
                |_cx| {
                    vec![
                        shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Copy")),
                        shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Cut")),
                    ]
                },
            )]
        },
    );
}

fn render_context_menu_submenu_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-context-menu-submenu",
        request_semantics,
        move |cx| {
            vec![shadcn::ContextMenu::from_open(open).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Right click")
                        .test_id("context-trigger")
                        .into_element(cx)
                },
                |_cx| {
                    vec![shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("More")
                            .test_id("submenu-trigger")
                            .submenu(vec![
                                shadcn::ContextMenuEntry::Item(
                                    shadcn::ContextMenuItem::new("Sub Alpha")
                                        .test_id("submenu-first-item"),
                                ),
                                shadcn::ContextMenuEntry::Item(
                                    shadcn::ContextMenuItem::new("Sub Beta")
                                        .test_id("submenu-second-item"),
                                ),
                            ]),
                    )]
                },
            )]
        },
    );
}

fn render_menubar_submenu_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-menubar-submenu",
        request_semantics,
        move |cx| {
            vec![
                shadcn::Menubar::new(vec![
                    shadcn::MenubarMenu::new("File").entries(vec![
                        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("New")),
                        shadcn::MenubarEntry::Submenu(
                            shadcn::MenubarItem::new("More")
                                .test_id("submenu-trigger")
                                .submenu(vec![
                                    shadcn::MenubarEntry::Item(
                                        shadcn::MenubarItem::new("Sub Alpha")
                                            .test_id("submenu-first-item"),
                                    ),
                                    shadcn::MenubarEntry::Item(
                                        shadcn::MenubarItem::new("Sub Beta")
                                            .test_id("submenu-second-item"),
                                    ),
                                ]),
                        ),
                        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Exit")),
                    ]),
                    shadcn::MenubarMenu::new("Edit").entries(vec![shadcn::MenubarEntry::Item(
                        shadcn::MenubarItem::new("Undo"),
                    )]),
                ])
                .into_element(cx),
            ]
        },
    );
}

fn render_context_menu_with_underlay_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
    underlay_activated: Model<bool>,
) {
    render_context_menu_with_underlay_frame_with_dismiss_handler(
        ui,
        app,
        services,
        window,
        bounds,
        request_semantics,
        open,
        false,
        underlay_activated,
        None,
    );
}

fn render_context_menu_with_underlay_frame_with_dismiss_handler(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    open: Model<bool>,
    modal: bool,
    underlay_activated: Model<bool>,
    on_dismiss_request: Option<OnDismissRequest>,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-focus-restore-context-menu-outside",
        request_semantics,
        move |cx| {
            let underlay = underlay_button(cx, underlay_activated);
            let menu = shadcn::ContextMenu::from_open(open)
                .modal(modal)
                .on_dismiss_request(on_dismiss_request)
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Right click")
                            .test_id("context-trigger")
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Copy")),
                            shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Cut")),
                        ]
                    },
                );
            vec![underlay, menu]
        },
    );
}

fn click_trigger(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    test_id: &str,
) -> Result<(), ScenarioObserveError> {
    let point = point_for_test_id(ui, test_id)?;
    click_at(ui, app, services, point);
    Ok(())
}

fn right_click_trigger(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    test_id: &str,
) -> Result<(), ScenarioObserveError> {
    let point = point_for_test_id(ui, test_id)?;
    right_click_at(ui, app, services, point);
    Ok(())
}

fn prevent_outside_press_handler(dismiss_calls: Model<u32>) -> OnDismissRequest {
    Arc::new(move |host, _cx, req| match req.reason {
        DismissReason::OutsidePress { .. } => {
            let _ = host.models_mut().update(&dismiss_calls, |count| {
                *count += 1;
            });
            req.prevent_default();
        }
        DismissReason::FocusOutside => {
            req.prevent_default();
        }
        _ => {}
    })
}

fn prevent_focus_outside_handler(dismiss_calls: Model<u32>) -> OnDismissRequest {
    Arc::new(move |host, _cx, req| {
        if matches!(req.reason, DismissReason::FocusOutside) {
            let _ = host.models_mut().update(&dismiss_calls, |count| {
                *count += 1;
            });
            req.prevent_default();
        }
    })
}

fn click_at_with_pointer_id(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    pointer_id: u64,
    position: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(pointer_id),
            position,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: fret_core::PointerId(pointer_id),
            position,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
}

fn focus_test_id(ui: &mut UiTree<App>, test_id: &str) -> Result<(), ScenarioObserveError> {
    let snap = ui.semantics_snapshot().cloned().ok_or_else(|| {
        ScenarioObserveError::new("missing semantics snapshot before focus lookup")
    })?;
    let node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .ok_or_else(|| ScenarioObserveError::new(format!("missing test id {test_id:?}")))?;
    ui.set_focus(Some(node.id));
    Ok(())
}

fn point_for_test_id(ui: &UiTree<App>, test_id: &str) -> Result<Point, ScenarioObserveError> {
    let snap = ui.semantics_snapshot().cloned().ok_or_else(|| {
        ScenarioObserveError::new("missing semantics snapshot before point lookup")
    })?;
    let trigger = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .ok_or_else(|| ScenarioObserveError::new(format!("missing test id {test_id:?}")))?;
    Ok(Point::new(
        Px(trigger.bounds.origin.x.0 + 5.0),
        Px(trigger.bounds.origin.y.0 + 5.0),
    ))
}

fn point_for_label(ui: &UiTree<App>, label: &str) -> Result<Point, ScenarioObserveError> {
    let snap = ui.semantics_snapshot().cloned().ok_or_else(|| {
        ScenarioObserveError::new("missing semantics snapshot before label lookup")
    })?;
    let node = snap
        .nodes
        .iter()
        .find(|node| node.label.as_deref() == Some(label))
        .ok_or_else(|| ScenarioObserveError::new(format!("missing label {label:?}")))?;
    Ok(Point::new(
        Px(node.bounds.origin.x.0 + 5.0),
        Px(node.bounds.origin.y.0 + 5.0),
    ))
}

fn observed_focus_restore_tree(
    ui: &UiTree<App>,
    app: &App,
    open: &Model<bool>,
    bounds: Rect,
) -> Result<ObservedTree, ScenarioObserveError> {
    let snapshot = ui
        .semantics_snapshot()
        .cloned()
        .ok_or_else(|| ScenarioObserveError::new("missing final semantics snapshot"))?;
    let mut observed = ObservedTree::from_semantics_snapshot(&snapshot, bounds);
    observed.set_metric(
        "recipe.open",
        if app.models().get_copied(open) == Some(true) {
            1.0
        } else {
            0.0
        },
    );
    Ok(observed)
}

fn observed_focus_policy_tree(
    ui: &UiTree<App>,
    app: &App,
    open: &Model<bool>,
    bounds: Rect,
) -> Result<ObservedTree, ScenarioObserveError> {
    let mut observed = observed_focus_restore_tree(ui, app, open, bounds)?;
    observed.set_metric(
        "focus.none",
        if observed.focus_node_id.is_none() {
            1.0
        } else {
            0.0
        },
    );
    Ok(observed)
}

fn set_bool_model_metric(
    observed: &mut ObservedTree,
    app: &App,
    model: &Model<bool>,
    metric_id: &'static str,
) {
    observed.set_metric(
        metric_id,
        if app.models().get_copied(model) == Some(true) {
            1.0
        } else {
            0.0
        },
    );
}

fn set_u32_model_metric(
    observed: &mut ObservedTree,
    app: &App,
    model: &Model<u32>,
    metric_id: &'static str,
) {
    observed.set_metric(
        metric_id,
        app.models().get_copied(model).unwrap_or_default() as f32,
    );
}

fn expect_open(app: &App, open: &Model<bool>, expected: bool) -> Result<(), ScenarioObserveError> {
    let actual = app.models().get_copied(open);
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(ScenarioObserveError::new(format!(
            "unexpected open state expected={expected} actual={actual:?}"
        )))
    }
}

fn flush_timers(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    timers: &mut TimerQueue,
) {
    timers.ingest_effects(app);
    timers.fire_all(ui, app, services);
}

fn default_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    )
}

fn themed_app() -> App {
    let mut app = App::new();
    shadcn::themes::apply_shadcn_new_york(
        &mut app,
        shadcn::themes::ShadcnBaseColor::Neutral,
        shadcn::themes::ShadcnColorScheme::Light,
    );
    app
}
