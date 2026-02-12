use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use fret_diag_protocol::builder::{ScriptV2Builder, role_and_name, test_id, text_composition_is};
use fret_diag_protocol::{
    UiActionScriptV2, UiActionStepV2, UiKeyModifiersV1, UiOverlayPlacementTraceKindV1,
    UiOverlayPlacementTraceQueryV1, UiPredicateV1, UiScriptMetaV1, UiSelectorV1,
    UiShortcutRoutingTraceQueryV1,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        return help();
    };

    match cmd.as_str() {
        "help" | "-h" | "--help" => help(),
        "list" => {
            for name in template_names() {
                println!("{name}");
            }
            Ok(())
        }
        "print" => {
            let name = args
                .next()
                .ok_or_else(|| "missing template name (try: list)".to_string())?;
            let script = template_v2(&name)?;
            println!("{}", to_pretty_json(&script)?);
            Ok(())
        }
        "write" => {
            let name = args
                .next()
                .ok_or_else(|| "missing template name (try: list)".to_string())?;

            let mut out: Option<PathBuf> = None;
            let mut overwrite = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        let path = args
                            .next()
                            .ok_or_else(|| "missing value for --out".to_string())?;
                        out = Some(PathBuf::from(path));
                    }
                    "--overwrite" => overwrite = true,
                    other => return Err(format!("unknown arg: {other}")),
                }
            }

            let workspace_root = workspace_root()?;
            let out = out.unwrap_or_else(|| {
                workspace_root
                    .join(".fret")
                    .join("diag")
                    .join("scripts")
                    .join(format!("{name}.json"))
            });

            let script = template_v2(&name)?;
            write_json_file(&out, &script, overwrite)?;
            println!("{}", out.display());
            Ok(())
        }
        "check-suite" => {
            let suite = args.next().ok_or_else(|| {
                "missing suite name (try: check-suite ui-gallery-select)".to_string()
            })?;
            let workspace_root = workspace_root()?;
            let (status, errors) = check_suite(&suite, &workspace_root)?;
            if errors > 0 {
                return Err(format!("suite check failed: {errors} mismatches"));
            }
            println!("{status}");
            Ok(())
        }
        "check" => {
            let name = args
                .next()
                .ok_or_else(|| "missing template name (try: list)".to_string())?;
            let path = args
                .next()
                .ok_or_else(|| "missing json path to compare".to_string())?;

            let script = template_v2(&name)?;
            let expected = serde_json::to_value(&script).map_err(|e| e.to_string())?;
            let actual_text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let actual_script: UiActionScriptV2 =
                serde_json::from_str(&actual_text).map_err(|e| {
                    format!(
                        "failed to parse script json at {path}: {e}",
                        path = Path::new(&path).display()
                    )
                })?;
            let actual = serde_json::to_value(&actual_script).map_err(|e| e.to_string())?;

            if expected != actual {
                return Err("json differs (compare with: print)".to_string());
            }
            Ok(())
        }
        other => Err(format!("unknown command: {other}")),
    }
}

fn help() -> Result<(), String> {
    println!(
        r#"fret-diag-scriptgen - generate typed UI diag scripts (JSON) from Rust templates

Usage:
  fret-diag-scriptgen help
  fret-diag-scriptgen list
  fret-diag-scriptgen print <template>
  fret-diag-scriptgen write <template> [--out <path>] [--overwrite]
  fret-diag-scriptgen check-suite <suite>
  fret-diag-scriptgen check <template> <json_path>

Notes:
  - The default output path is `.fret/diag/scripts/<template>.json` under the workspace root.
  - These scripts are compatible with `fretboard diag run <script.json>`.
"#
    );
    Ok(())
}

fn template_names() -> &'static [&'static str] {
    &[
        "todo-baseline-v2",
        "ui-gallery-command-palette-shortcut-primary-v2",
        "ui-gallery-input-ime-tab-suppressed-v2",
        "ui-gallery-combobox-open-select-focus-restore-v2",
        "ui-gallery-combobox-keyboard-commit-apple-v2",
        "ui-gallery-combobox-typeahead-commit-banana-v2",
        "ui-gallery-combobox-escape-dismiss-focus-restore-v2",
        "ui-gallery-combobox-dismiss-outside-press-v2",
        "ui-gallery-combobox-roving-skips-disabled-v2",
        "ui-gallery-select-commit-and-label-update-bundle-v2",
        "ui-gallery-select-keyboard-commit-apple-v2",
        "ui-gallery-select-typeahead-commit-banana-v2",
        "ui-gallery-select-disabled-item-no-commit-v2",
        "ui-gallery-select-roving-skips-disabled-orange-v2",
        "ui-gallery-select-dismiss-outside-press-v2",
        "ui-gallery-select-escape-dismiss-focus-restore-v2",
        "ui-gallery-select-trigger-toggle-close-v2",
        "ui-gallery-select-open-jitter-click-stable-v2",
        "ui-gallery-select-wheel-scroll-v2",
        "ui-gallery-select-wheel-up-from-bottom-v2",
    ]
}

fn template_v2(name: &str) -> Result<UiActionScriptV2, String> {
    match name {
        "todo-baseline-v2" => Ok(todo_baseline_v2()),
        "ui-gallery-command-palette-shortcut-primary-v2" => {
            Ok(ui_gallery_command_palette_shortcut_primary_v2())
        }
        "ui-gallery-input-ime-tab-suppressed-v2" => Ok(ui_gallery_input_ime_tab_suppressed_v2()),
        "ui-gallery-combobox-open-select-focus-restore-v2" => {
            Ok(ui_gallery_combobox_open_select_focus_restore_v2())
        }
        "ui-gallery-combobox-keyboard-commit-apple-v2" => {
            Ok(ui_gallery_combobox_keyboard_commit_apple_v2())
        }
        "ui-gallery-combobox-typeahead-commit-banana-v2" => {
            Ok(ui_gallery_combobox_typeahead_commit_banana_v2())
        }
        "ui-gallery-combobox-escape-dismiss-focus-restore-v2" => {
            Ok(ui_gallery_combobox_escape_dismiss_focus_restore_v2())
        }
        "ui-gallery-combobox-dismiss-outside-press-v2" => {
            Ok(ui_gallery_combobox_dismiss_outside_press_v2())
        }
        "ui-gallery-combobox-roving-skips-disabled-v2" => {
            Ok(ui_gallery_combobox_roving_skips_disabled_v2())
        }
        "ui-gallery-select-commit-and-label-update-bundle-v2" => {
            Ok(ui_gallery_select_commit_and_label_update_bundle_v2())
        }
        "ui-gallery-select-keyboard-commit-apple-v2" => {
            Ok(ui_gallery_select_keyboard_commit_apple_v2())
        }
        "ui-gallery-select-typeahead-commit-banana-v2" => {
            Ok(ui_gallery_select_typeahead_commit_banana_v2())
        }
        "ui-gallery-select-disabled-item-no-commit-v2" => {
            Ok(ui_gallery_select_disabled_item_no_commit_v2())
        }
        "ui-gallery-select-roving-skips-disabled-orange-v2" => {
            Ok(ui_gallery_select_roving_skips_disabled_orange_v2())
        }
        "ui-gallery-select-dismiss-outside-press-v2" => {
            Ok(ui_gallery_select_dismiss_outside_press_v2())
        }
        "ui-gallery-select-escape-dismiss-focus-restore-v2" => {
            Ok(ui_gallery_select_escape_dismiss_focus_restore_v2())
        }
        "ui-gallery-select-trigger-toggle-close-v2" => {
            Ok(ui_gallery_select_trigger_toggle_close_v2())
        }
        "ui-gallery-select-open-jitter-click-stable-v2" => {
            Ok(ui_gallery_select_open_jitter_click_stable_v2())
        }
        "ui-gallery-select-wheel-scroll-v2" => Ok(ui_gallery_select_wheel_scroll_v2()),
        "ui-gallery-select-wheel-up-from-bottom-v2" => {
            Ok(ui_gallery_select_wheel_up_from_bottom_v2())
        }
        other => Err(format!("unknown template: {other} (try: list)")),
    }
}

fn todo_baseline_v2() -> UiActionScriptV2 {
    ScriptV2Builder::new()
        .type_text_into(test_id("todo-input"), "Automated task")
        .wait_frames(2)
        .press_key("enter")
        .wait_exists(test_id("todo-item-4-done"), 60)
        .capture_bundle(Some("todo-after-add".to_string()))
        .click(test_id("todo-item-4-done"))
        .wait_frames(2)
        .capture_bundle(Some("todo-after-toggle-done".to_string()))
        .click(test_id("todo-item-4-remove"))
        .wait_frames(2)
        .capture_bundle(Some("todo-after-remove".to_string()))
        .build()
}

fn ui_gallery_command_palette_shortcut_primary_v2() -> UiActionScriptV2 {
    let dialog = role_and_name("dialog", "Command palette");
    ScriptV2Builder::new()
        .press_key("escape")
        .wait_frames(2)
        .press_shortcut("primary+p")
        .wait_exists(dialog.clone(), 240)
        .assert_exists(dialog.clone())
        .press_key("escape")
        .wait_not_exists(dialog, 240)
        .capture_bundle(Some(
            "ui-gallery-command-palette-shortcut-primary".to_string(),
        ))
        .build()
}

fn ctrl_a_step() -> UiActionStepV2 {
    UiActionStepV2::PressKey {
        key: "a".to_string(),
        modifiers: UiKeyModifiersV1 {
            ctrl: true,
            ..Default::default()
        },
        repeat: false,
    }
}

fn wait_bounds_within_window_step(target: UiSelectorV1, timeout_frames: u32) -> UiActionStepV2 {
    UiActionStepV2::WaitUntil {
        predicate: UiPredicateV1::BoundsWithinWindow {
            target,
            padding_px: 2.0,
            eps_px: 0.5,
        },
        timeout_frames,
    }
}

fn ui_gallery_nav_to_select_page() -> ScriptV2Builder {
    ScriptV2Builder::new()
        .wait_exists(test_id("ui-gallery-nav-search"), 600)
        .press_key("escape")
        .wait_frames(2)
        .click(test_id("ui-gallery-nav-search"))
        .push(ctrl_a_step())
        .press_key("backspace")
        .type_text("select")
        .wait_frames(2)
        .click(test_id("ui-gallery-nav-select"))
        .wait_exists(test_id("ui-gallery-page-select"), 600)
        .wait_exists(test_id("ui-gallery-select-trigger"), 600)
}

fn ui_gallery_nav_to_select_page_no_escape() -> ScriptV2Builder {
    ScriptV2Builder::new()
        .wait_exists(test_id("ui-gallery-nav-search"), 600)
        .click(test_id("ui-gallery-nav-search"))
        .push(ctrl_a_step())
        .press_key("backspace")
        .type_text("select")
        .wait_frames(2)
        .click(test_id("ui-gallery-nav-select"))
        .wait_exists(test_id("ui-gallery-page-select"), 600)
        .wait_exists(test_id("ui-gallery-select-trigger"), 600)
}

fn ui_gallery_nav_to_combobox_page() -> ScriptV2Builder {
    ScriptV2Builder::new()
        .wait_exists(test_id("ui-gallery-nav-search"), 600)
        .press_key("escape")
        .wait_frames(2)
        .click(test_id("ui-gallery-nav-search"))
        .push(ctrl_a_step())
        .press_key("backspace")
        .type_text("combobox")
        .wait_frames(2)
        .wait_exists(test_id("ui-gallery-nav-combobox"), 600)
        .click(test_id("ui-gallery-nav-combobox"))
        .wait_exists(test_id("ui-gallery-page-combobox"), 600)
        .wait_exists(test_id("ui-gallery-combobox-demo-trigger"), 600)
}

fn ui_gallery_nav_to_input_page() -> ScriptV2Builder {
    ScriptV2Builder::new()
        .wait_exists(test_id("ui-gallery-nav-search"), 600)
        .press_key("escape")
        .wait_frames(2)
        .click(test_id("ui-gallery-nav-search"))
        .push(ctrl_a_step())
        .press_key("backspace")
        .type_text("input")
        .wait_frames(2)
        .wait_exists(test_id("ui-gallery-nav-input"), 600)
        .click(test_id("ui-gallery-nav-input"))
        .wait_exists(test_id("ui-gallery-page-input"), 600)
        .wait_exists(test_id("ui-gallery-input-basic"), 600)
}

fn with_required_caps(mut script: UiActionScriptV2, caps: &[&str]) -> UiActionScriptV2 {
    script.meta = Some(UiScriptMetaV1 {
        required_capabilities: caps.iter().map(|s| (*s).to_string()).collect(),
        ..Default::default()
    });
    script
}

fn ui_gallery_input_ime_tab_suppressed_v2() -> UiActionScriptV2 {
    let input = test_id("ui-gallery-input-basic");
    let script = ui_gallery_nav_to_input_page()
        .click(input.clone())
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: input.clone(),
            },
            timeout_frames: 240,
        })
        .ime_preedit("東京", Some((0, 6)))
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::TextCompositionIs {
                target: input.clone(),
                composing: true,
            },
            timeout_frames: 240,
        })
        .press_key("tab")
        .wait_shortcut_routing_trace(
            UiShortcutRoutingTraceQueryV1 {
                outcome: Some("reserved_for_ime".to_string()),
                key: Some("Tab".to_string()),
                ime_composing: Some(true),
                focus_is_text_input: Some(true),
                ..UiShortcutRoutingTraceQueryV1::default()
            },
            120,
        )
        .assert_focus_is(input.clone())
        .assert(text_composition_is(input.clone(), true))
        .ime_commit("東京")
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::TextCompositionIs {
                target: input.clone(),
                composing: false,
            },
            timeout_frames: 240,
        })
        .capture_bundle(Some("ui-gallery-input-ime-tab-suppressed".to_string()))
        .build();
    with_required_caps(
        script,
        &[
            "diag.script_v2",
            "diag.inject_ime",
            "diag.shortcut_routing_trace",
        ],
    )
}

fn ui_gallery_combobox_open_select_focus_restore_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_combobox_page()
        .click(test_id("ui-gallery-combobox-demo-trigger"))
        .wait_exists(test_id("ui-gallery-combobox-demo-input"), 240)
        .wait_exists(test_id("ui-gallery-combobox-demo-listbox"), 240)
        .push(wait_bounds_within_window_step(
            test_id("ui-gallery-combobox-demo-listbox"),
            240,
        ))
        .wait_bounds_stable(test_id("ui-gallery-combobox-demo-listbox"))
        .click(test_id("ui-gallery-combobox-demo-item-apple"))
        .wait_not_exists(test_id("ui-gallery-combobox-demo-listbox"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-combobox-demo-trigger"),
            },
            timeout_frames: 240,
        })
        .click(test_id("ui-gallery-combobox-demo-trigger"))
        .wait_exists(test_id("ui-gallery-combobox-demo-item-apple"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::SelectedIs {
                target: test_id("ui-gallery-combobox-demo-item-apple"),
                selected: true,
            },
            timeout_frames: 240,
        })
        .press_key("escape")
        .wait_not_exists(test_id("ui-gallery-combobox-demo-listbox"), 240)
        .capture_bundle(Some(
            "ui-gallery-combobox-open-select-focus-restore".to_string(),
        ))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_combobox_keyboard_commit_apple_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_combobox_page()
        .click(test_id("ui-gallery-combobox-demo-trigger"))
        .wait_exists(test_id("ui-gallery-combobox-demo-input"), 240)
        .wait_exists(test_id("ui-gallery-combobox-demo-listbox"), 240)
        .push(wait_bounds_within_window_step(
            test_id("ui-gallery-combobox-demo-listbox"),
            240,
        ))
        .wait_bounds_stable(test_id("ui-gallery-combobox-demo-listbox"))
        .press_key("home")
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::ActiveItemIs {
                container: test_id("ui-gallery-combobox-demo-input"),
                item: test_id("ui-gallery-combobox-demo-item-apple"),
            },
            timeout_frames: 240,
        })
        .press_key("enter")
        .wait_not_exists(test_id("ui-gallery-combobox-demo-item-apple"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-combobox-demo-trigger"),
            },
            timeout_frames: 240,
        })
        .click(test_id("ui-gallery-combobox-demo-trigger"))
        .wait_exists(test_id("ui-gallery-combobox-demo-item-apple"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::SelectedIs {
                target: test_id("ui-gallery-combobox-demo-item-apple"),
                selected: true,
            },
            timeout_frames: 240,
        })
        .press_key("escape")
        .wait_not_exists(test_id("ui-gallery-combobox-demo-item-apple"), 240)
        .capture_bundle(Some(
            "ui-gallery-combobox-keyboard-commit-apple".to_string(),
        ))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_combobox_typeahead_commit_banana_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_combobox_page()
        .click(test_id("ui-gallery-combobox-demo-trigger"))
        .wait_exists(test_id("ui-gallery-combobox-demo-input"), 240)
        .wait_exists(test_id("ui-gallery-combobox-demo-listbox"), 240)
        .push(wait_bounds_within_window_step(
            test_id("ui-gallery-combobox-demo-listbox"),
            240,
        ))
        .wait_bounds_stable(test_id("ui-gallery-combobox-demo-listbox"))
        .type_text("ban")
        .wait_exists(test_id("ui-gallery-combobox-demo-item-banana"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::ActiveItemIs {
                container: test_id("ui-gallery-combobox-demo-input"),
                item: test_id("ui-gallery-combobox-demo-item-banana"),
            },
            timeout_frames: 240,
        })
        .press_key("enter")
        .wait_not_exists(test_id("ui-gallery-combobox-demo-item-banana"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-combobox-demo-trigger"),
            },
            timeout_frames: 240,
        })
        .click(test_id("ui-gallery-combobox-demo-trigger"))
        .wait_exists(test_id("ui-gallery-combobox-demo-item-banana"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::SelectedIs {
                target: test_id("ui-gallery-combobox-demo-item-banana"),
                selected: true,
            },
            timeout_frames: 240,
        })
        .press_key("escape")
        .wait_not_exists(test_id("ui-gallery-combobox-demo-item-banana"), 240)
        .capture_bundle(Some(
            "ui-gallery-combobox-typeahead-commit-banana".to_string(),
        ))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_combobox_escape_dismiss_focus_restore_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_combobox_page()
        .click(test_id("ui-gallery-combobox-demo-trigger"))
        .wait_exists(test_id("ui-gallery-combobox-demo-input"), 240)
        .wait_exists(test_id("ui-gallery-combobox-demo-listbox"), 240)
        .push(wait_bounds_within_window_step(
            test_id("ui-gallery-combobox-demo-listbox"),
            240,
        ))
        .wait_bounds_stable(test_id("ui-gallery-combobox-demo-listbox"))
        .press_key("escape")
        .wait_not_exists(test_id("ui-gallery-combobox-demo-input"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-combobox-demo-trigger"),
            },
            timeout_frames: 240,
        })
        .capture_bundle(Some(
            "ui-gallery-combobox-escape-dismiss-focus-restore".to_string(),
        ))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_combobox_dismiss_outside_press_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_combobox_page()
        .click(test_id("ui-gallery-combobox-demo-trigger"))
        .wait_exists(test_id("ui-gallery-combobox-demo-input"), 240)
        .wait_exists(test_id("ui-gallery-combobox-demo-listbox"), 240)
        .push(wait_bounds_within_window_step(
            test_id("ui-gallery-combobox-demo-listbox"),
            240,
        ))
        .wait_bounds_stable(test_id("ui-gallery-combobox-demo-listbox"))
        .click(test_id("ui-gallery-nav-search"))
        .wait_not_exists(test_id("ui-gallery-combobox-demo-input"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-combobox-demo-trigger"),
            },
            timeout_frames: 240,
        })
        .capture_bundle(Some(
            "ui-gallery-combobox-dismiss-outside-press".to_string(),
        ))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_combobox_roving_skips_disabled_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_combobox_page()
        .click(test_id("ui-gallery-combobox-demo-trigger"))
        .wait_exists(test_id("ui-gallery-combobox-demo-input"), 240)
        .wait_exists(test_id("ui-gallery-combobox-demo-listbox"), 240)
        .push(wait_bounds_within_window_step(
            test_id("ui-gallery-combobox-demo-listbox"),
            240,
        ))
        .wait_bounds_stable(test_id("ui-gallery-combobox-demo-listbox"))
        .wait_exists(test_id("ui-gallery-combobox-demo-item-disabled"), 240)
        .press_key("end")
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::ActiveItemIs {
                container: test_id("ui-gallery-combobox-demo-input"),
                item: test_id("ui-gallery-combobox-demo-item-orange"),
            },
            timeout_frames: 240,
        })
        .press_key("escape")
        .wait_not_exists(test_id("ui-gallery-combobox-demo-input"), 240)
        .capture_bundle(Some(
            "ui-gallery-combobox-roving-skips-disabled".to_string(),
        ))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_select_open_jitter_click_stable_v2() -> UiActionScriptV2 {
    let script = ScriptV2Builder::new()
        .wait_exists(test_id("ui-gallery-nav-search"), 600)
        .press_key("escape")
        .wait_frames(2)
        .click(test_id("ui-gallery-nav-search"))
        .push(UiActionStepV2::PressKey {
            key: "a".to_string(),
            modifiers: UiKeyModifiersV1 {
                ctrl: true,
                ..Default::default()
            },
            repeat: false,
        })
        .press_key("backspace")
        .type_text("select")
        .wait_frames(2)
        .click(test_id("ui-gallery-nav-select"))
        .wait_exists(test_id("ui-gallery-page-select"), 600)
        .wait_exists(test_id("ui-gallery-select-trigger"), 600)
        .click_stable(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .wait_overlay_placement_trace(
            UiOverlayPlacementTraceQueryV1 {
                kind: Some(UiOverlayPlacementTraceKindV1::AnchoredPanel),
                anchor_test_id: Some("ui-gallery-select-trigger".to_string()),
                content_test_id: Some("select-scroll-viewport".to_string()),
                ..UiOverlayPlacementTraceQueryV1::default()
            },
            240,
        )
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::BoundsWithinWindow {
                target: test_id("select-scroll-viewport"),
                padding_px: 2.0,
                eps_px: 0.5,
            },
            timeout_frames: 240,
        })
        .push(UiActionStepV2::WaitBoundsStable {
            target: test_id("select-scroll-viewport"),
            stable_frames: 4,
            max_move_px: 0.5,
            timeout_frames: 180,
        })
        .click_stable(test_id("ui-gallery-select-item-banana"))
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::SelectedIs {
                target: test_id("ui-gallery-select-item-banana"),
                selected: true,
            },
            timeout_frames: 240,
        })
        .press_key("escape")
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .capture_bundle(Some(
            "ui-gallery-select-open-jitter-click-stable-v2".to_string(),
        ))
        .build();

    with_required_caps(script, &["diag.script_v2", "diag.overlay_placement_trace"])
}

fn ui_gallery_select_commit_and_label_update_bundle_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_select_page()
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("ui-gallery-select-item-banana"), 240)
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(wait_bounds_within_window_step(
            test_id("select-scroll-viewport"),
            240,
        ))
        .wait_frames(30)
        .click(test_id("ui-gallery-select-item-banana"))
        .wait_not_exists(test_id("ui-gallery-select-item-banana"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-select-trigger"),
            },
            timeout_frames: 240,
        })
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::SelectedIs {
                target: test_id("ui-gallery-select-item-banana"),
                selected: true,
            },
            timeout_frames: 240,
        })
        .press_key("escape")
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .capture_bundle(Some(
            "ui-gallery-select-commit-and-label-update".to_string(),
        ))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_select_keyboard_commit_apple_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_select_page()
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(wait_bounds_within_window_step(
            test_id("select-scroll-viewport"),
            240,
        ))
        .press_key("home")
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::ActiveItemIs {
                container: test_id("select-scroll-viewport"),
                item: test_id("ui-gallery-select-item-apple"),
            },
            timeout_frames: 240,
        })
        .press_key("enter")
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-select-trigger"),
            },
            timeout_frames: 240,
        })
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::SelectedIs {
                target: test_id("ui-gallery-select-item-apple"),
                selected: true,
            },
            timeout_frames: 240,
        })
        .press_key("escape")
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .capture_bundle(Some("ui-gallery-select-keyboard-commit-apple".to_string()))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_select_typeahead_commit_banana_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_select_page()
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(wait_bounds_within_window_step(
            test_id("select-scroll-viewport"),
            240,
        ))
        .press_key("b")
        .wait_frames(2)
        .press_key("enter")
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-select-trigger"),
            },
            timeout_frames: 240,
        })
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::SelectedIs {
                target: test_id("ui-gallery-select-item-banana"),
                selected: true,
            },
            timeout_frames: 240,
        })
        .press_key("escape")
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .capture_bundle(Some(
            "ui-gallery-select-typeahead-commit-banana".to_string(),
        ))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_select_disabled_item_no_commit_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_select_page()
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(wait_bounds_within_window_step(
            test_id("select-scroll-viewport"),
            240,
        ))
        .wait_exists(test_id("ui-gallery-select-item-item-15"), 240)
        .click(test_id("ui-gallery-select-item-banana"))
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-select-trigger"),
            },
            timeout_frames: 240,
        })
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .wait_exists(test_id("ui-gallery-select-item-item-15"), 240)
        .click(test_id("ui-gallery-select-item-item-15"))
        .wait_frames(5)
        .wait_exists(test_id("select-scroll-viewport"), 120)
        .press_key("escape")
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-select-trigger"),
            },
            timeout_frames: 240,
        })
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::SelectedIs {
                target: test_id("ui-gallery-select-item-banana"),
                selected: true,
            },
            timeout_frames: 240,
        })
        .press_key("escape")
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .capture_bundle(Some(
            "ui-gallery-select-disabled-item-no-commit".to_string(),
        ))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_select_roving_skips_disabled_orange_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_select_page()
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(wait_bounds_within_window_step(
            test_id("select-scroll-viewport"),
            240,
        ))
        .press_key("home")
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::ActiveItemIs {
                container: test_id("select-scroll-viewport"),
                item: test_id("ui-gallery-select-item-apple"),
            },
            timeout_frames: 240,
        })
        .press_key("arrow_down")
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::ActiveItemIs {
                container: test_id("select-scroll-viewport"),
                item: test_id("ui-gallery-select-item-banana"),
            },
            timeout_frames: 240,
        })
        .press_key("arrow_down")
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::ActiveItemIs {
                container: test_id("select-scroll-viewport"),
                item: test_id("ui-gallery-select-item-item-01"),
            },
            timeout_frames: 240,
        })
        .capture_bundle(Some(
            "ui-gallery-select-roving-skips-disabled-orange".to_string(),
        ))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_select_dismiss_outside_press_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_select_page()
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("ui-gallery-select-item-apple"), 240)
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(wait_bounds_within_window_step(
            test_id("select-scroll-viewport"),
            240,
        ))
        .wait_frames(10)
        .click(test_id("ui-gallery-nav-search"))
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-select-trigger"),
            },
            timeout_frames: 240,
        })
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .wait_not_exists(test_id("ui-gallery-select-item-apple"), 240)
        .capture_bundle(Some("ui-gallery-select-dismiss-outside-press".to_string()))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_select_escape_dismiss_focus_restore_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_select_page()
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(wait_bounds_within_window_step(
            test_id("select-scroll-viewport"),
            240,
        ))
        .press_key("escape")
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-select-trigger"),
            },
            timeout_frames: 240,
        })
        .capture_bundle(Some(
            "ui-gallery-select-escape-dismiss-focus-restore".to_string(),
        ))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_select_trigger_toggle_close_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_select_page()
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(wait_bounds_within_window_step(
            test_id("select-scroll-viewport"),
            240,
        ))
        .wait_frames(5)
        .click(test_id("ui-gallery-select-trigger"))
        .wait_not_exists(test_id("select-scroll-viewport"), 240)
        .push(UiActionStepV2::WaitUntil {
            predicate: UiPredicateV1::FocusIs {
                target: test_id("ui-gallery-select-trigger"),
            },
            timeout_frames: 240,
        })
        .capture_bundle(Some("ui-gallery-select-trigger-toggle-close".to_string()))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_select_wheel_scroll_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_select_page_no_escape()
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("ui-gallery-select-item-apple"), 60)
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(wait_bounds_within_window_step(
            test_id("select-scroll-viewport"),
            240,
        ))
        .push(UiActionStepV2::MovePointer {
            target: test_id("ui-gallery-select-item-apple"),
        })
        .push(UiActionStepV2::Wheel {
            target: test_id("ui-gallery-select-item-apple"),
            delta_x: 0.0,
            delta_y: -120.0,
        })
        .wait_frames(5)
        .push(UiActionStepV2::Wheel {
            target: test_id("ui-gallery-select-item-apple"),
            delta_x: 0.0,
            delta_y: -240.0,
        })
        .wait_frames(5)
        .push(UiActionStepV2::Wheel {
            target: test_id("ui-gallery-select-item-apple"),
            delta_x: 0.0,
            delta_y: 120.0,
        })
        .wait_frames(5)
        .capture_bundle(Some("ui-gallery-select-wheel-scroll".to_string()))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn ui_gallery_select_wheel_up_from_bottom_v2() -> UiActionScriptV2 {
    let script = ui_gallery_nav_to_select_page_no_escape()
        .click(test_id("ui-gallery-select-trigger"))
        .wait_exists(test_id("ui-gallery-select-item-apple"), 240)
        .wait_exists(test_id("select-scroll-viewport"), 240)
        .push(wait_bounds_within_window_step(
            test_id("select-scroll-viewport"),
            240,
        ))
        .press_key("end")
        .wait_frames(10)
        .wait_exists(test_id("ui-gallery-select-item-item-40"), 120)
        .push(UiActionStepV2::MovePointer {
            target: test_id("ui-gallery-select-item-item-40"),
        })
        .wait_frames(5)
        .push(UiActionStepV2::Wheel {
            target: test_id("ui-gallery-select-item-item-40"),
            delta_x: 0.0,
            delta_y: 2400.0,
        })
        .wait_frames(5)
        .capture_bundle(Some("ui-gallery-select-wheel-up-from-bottom".to_string()))
        .build();
    with_required_caps(script, &["diag.script_v2"])
}

fn check_suite(suite: &str, workspace_root: &Path) -> Result<(String, u64), String> {
    let mut errors: u64 = 0;
    let mut checked: u64 = 0;

    let items: &[(&str, &str)] = match suite {
        "ui-gallery-text-ime" => &[(
            "ui-gallery-input-ime-tab-suppressed-v2",
            "tools/diag-scripts/ui-gallery-input-ime-tab-suppressed.json",
        )],
        "ui-gallery-combobox" => &[
            (
                "ui-gallery-combobox-open-select-focus-restore-v2",
                "tools/diag-scripts/ui-gallery-combobox-open-select-focus-restore.json",
            ),
            (
                "ui-gallery-combobox-keyboard-commit-apple-v2",
                "tools/diag-scripts/ui-gallery-combobox-keyboard-commit-apple.json",
            ),
            (
                "ui-gallery-combobox-typeahead-commit-banana-v2",
                "tools/diag-scripts/ui-gallery-combobox-typeahead-commit-banana.json",
            ),
            (
                "ui-gallery-combobox-escape-dismiss-focus-restore-v2",
                "tools/diag-scripts/ui-gallery-combobox-escape-dismiss-focus-restore.json",
            ),
            (
                "ui-gallery-combobox-dismiss-outside-press-v2",
                "tools/diag-scripts/ui-gallery-combobox-dismiss-outside-press.json",
            ),
            (
                "ui-gallery-combobox-roving-skips-disabled-v2",
                "tools/diag-scripts/ui-gallery-combobox-roving-skips-disabled.json",
            ),
        ],
        "ui-gallery-select" => &[
            (
                "ui-gallery-select-commit-and-label-update-bundle-v2",
                "tools/diag-scripts/ui-gallery-select-commit-and-label-update-bundle.json",
            ),
            (
                "ui-gallery-select-keyboard-commit-apple-v2",
                "tools/diag-scripts/ui-gallery-select-keyboard-commit-apple.json",
            ),
            (
                "ui-gallery-select-typeahead-commit-banana-v2",
                "tools/diag-scripts/ui-gallery-select-typeahead-commit-banana.json",
            ),
            (
                "ui-gallery-select-disabled-item-no-commit-v2",
                "tools/diag-scripts/ui-gallery-select-disabled-item-no-commit.json",
            ),
            (
                "ui-gallery-select-roving-skips-disabled-orange-v2",
                "tools/diag-scripts/ui-gallery-select-roving-skips-disabled-orange.json",
            ),
            (
                "ui-gallery-select-dismiss-outside-press-v2",
                "tools/diag-scripts/ui-gallery-select-dismiss-outside-press.json",
            ),
            (
                "ui-gallery-select-escape-dismiss-focus-restore-v2",
                "tools/diag-scripts/ui-gallery-select-escape-dismiss-focus-restore.json",
            ),
            (
                "ui-gallery-select-open-jitter-click-stable-v2",
                "tools/diag-scripts/ui-gallery-select-open-jitter-click-stable-v2.json",
            ),
            (
                "ui-gallery-select-wheel-scroll-v2",
                "tools/diag-scripts/ui-gallery-select-wheel-scroll.json",
            ),
            (
                "ui-gallery-select-wheel-up-from-bottom-v2",
                "tools/diag-scripts/ui-gallery-select-wheel-up-from-bottom.json",
            ),
        ],
        other => return Err(format!("unknown suite: {other}")),
    };

    for (template, rel_path) in items {
        checked += 1;

        let script = template_v2(template)?;
        let expected = serde_json::to_value(&script).map_err(|e| e.to_string())?;

        let abs = workspace_root.join(rel_path);
        let actual_text = fs::read_to_string(&abs).map_err(|e| e.to_string())?;
        let actual_script: UiActionScriptV2 = serde_json::from_str(&actual_text).map_err(|e| {
            format!(
                "failed to parse script json at {path}: {e}",
                path = abs.display()
            )
        })?;
        let actual = serde_json::to_value(&actual_script).map_err(|e| e.to_string())?;

        if expected != actual {
            errors += 1;
            eprintln!("MISMATCH template={template} path={}", abs.display());
        }
    }

    let status = if errors == 0 {
        format!("passed ({checked} checked)")
    } else {
        format!("failed ({errors} mismatches; {checked} checked)")
    };
    Ok((status, errors))
}

fn to_pretty_json(script: &UiActionScriptV2) -> Result<String, String> {
    serde_json::to_string_pretty(script).map_err(|e| e.to_string())
}

fn write_json_file(path: &Path, script: &UiActionScriptV2, overwrite: bool) -> Result<(), String> {
    if path.exists() && !overwrite {
        return Err(format!(
            "refusing to overwrite existing file: {path} (pass --overwrite)",
            path = path.display()
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut text = to_pretty_json(script)?;
    text.push('\n');
    fs::write(path, text).map_err(|e| e.to_string())?;
    Ok(())
}

fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for dir in cwd.ancestors() {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Err("failed to locate workspace root (Cargo.toml not found in ancestors)".to_string())
}
