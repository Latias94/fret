use fret::app::prelude::*;
use fret_app::Effect;
use fret_app::{
    CommandMeta, CommandScope, DefaultKeybinding, InputContext, KeyChord, KeymapService, Platform,
    PlatformFilter, format_sequence,
};
use fret_core::{FontWeight, KeyCode, Modifiers};
use fret_runtime::Model;
use fret_ui::{CommandAvailability, element::SemanticsDecoration};
use fret_undo::{CMD_EDIT_REDO, CMD_EDIT_UNDO, UndoHistory, UndoRecord, ValueTx};

mod act {
    fret::actions!([
        Inc = "cookbook.undo_basics.inc.v1",
        Dec = "cookbook.undo_basics.dec.v1",
        Reset = "cookbook.undo_basics.reset.v1",
        Undo = "edit.undo",
        Redo = "edit.redo"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.undo_basics.root";
const TEST_ID_VALUE: &str = "cookbook.undo_basics.value";
const TEST_ID_COALESCE: &str = "cookbook.undo_basics.coalesce";
const TEST_ID_INC: &str = "cookbook.undo_basics.inc";
const TEST_ID_DEC: &str = "cookbook.undo_basics.dec";
const TEST_ID_RESET: &str = "cookbook.undo_basics.reset";
const TEST_ID_UNDO: &str = "cookbook.undo_basics.undo";
const TEST_ID_REDO: &str = "cookbook.undo_basics.redo";
const TEST_ID_UNDO_SHORTCUT: &str = "cookbook.undo_basics.undo_shortcut";
const TEST_ID_REDO_SHORTCUT: &str = "cookbook.undo_basics.redo_shortcut";
const TEST_ID_NEXT_UNDO: &str = "cookbook.undo_basics.next_undo";
const TEST_ID_NEXT_REDO: &str = "cookbook.undo_basics.next_redo";

struct UndoBasicsView {
    value: Model<i32>,
    history: Model<UndoHistory<ValueTx<i32>>>,
    coalesce: Model<bool>,
}

fn install_commands(app: &mut KernelApp) {
    let undo_cmd = CommandId::from(CMD_EDIT_UNDO);
    let redo_cmd = CommandId::from(CMD_EDIT_REDO);

    app.commands_mut().register(
        undo_cmd.clone(),
        CommandMeta::new("Undo")
            .with_description("Undo the last committed edit (app-owned history).")
            .with_category("Edit")
            .with_scope(CommandScope::Widget)
            .with_default_keybindings([
                DefaultKeybinding::single(
                    PlatformFilter::Macos,
                    KeyChord::new(
                        KeyCode::KeyZ,
                        Modifiers {
                            meta: true,
                            ..Modifiers::default()
                        },
                    ),
                ),
                DefaultKeybinding::single(
                    PlatformFilter::All,
                    KeyChord::new(
                        KeyCode::KeyZ,
                        Modifiers {
                            ctrl: true,
                            ..Modifiers::default()
                        },
                    ),
                ),
            ]),
    );

    app.commands_mut().register(
        redo_cmd.clone(),
        CommandMeta::new("Redo")
            .with_description("Redo the last undone edit (app-owned history).")
            .with_category("Edit")
            .with_scope(CommandScope::Widget)
            .with_default_keybindings([
                DefaultKeybinding::single(
                    PlatformFilter::Macos,
                    KeyChord::new(
                        KeyCode::KeyZ,
                        Modifiers {
                            meta: true,
                            shift: true,
                            ..Modifiers::default()
                        },
                    ),
                ),
                DefaultKeybinding::single(
                    PlatformFilter::Macos,
                    KeyChord::new(
                        KeyCode::KeyY,
                        Modifiers {
                            meta: true,
                            ..Modifiers::default()
                        },
                    ),
                ),
                DefaultKeybinding::single(
                    PlatformFilter::All,
                    KeyChord::new(
                        KeyCode::KeyZ,
                        Modifiers {
                            ctrl: true,
                            shift: true,
                            ..Modifiers::default()
                        },
                    ),
                ),
                DefaultKeybinding::single(
                    PlatformFilter::All,
                    KeyChord::new(
                        KeyCode::KeyY,
                        Modifiers {
                            ctrl: true,
                            ..Modifiers::default()
                        },
                    ),
                ),
            ]),
    );

    fret_app::install_command_default_keybindings_into_keymap(app);
}

fn record_value_tx(
    models: &mut fret_runtime::ModelStore,
    value: &Model<i32>,
    history: &Model<UndoHistory<ValueTx<i32>>>,
    label: &'static str,
    coalesce_key: Option<&'static str>,
    after: i32,
) {
    let before = models.read(value, |v| *v).ok().unwrap_or_default();
    if before == after {
        return;
    }

    let _ = models.update(value, |v| *v = after);
    let _ = models.update(history, |h| {
        let record = UndoRecord::new(ValueTx::new(before, after)).label(label);
        if let Some(k) = coalesce_key {
            let mut record = record.coalesce_key(k);
            if !h.can_redo()
                && let Some(prev) = h.peek_undo()
                && prev.coalesce_key == record.coalesce_key
            {
                record.tx.before = prev.tx.before;
            }
            h.record_or_coalesce(record);
        } else {
            h.record(record);
        }
    });
}

impl View for UndoBasicsView {
    fn init(app: &mut KernelApp, _window: WindowId) -> Self {
        Self {
            value: app.models_mut().insert(0),
            history: app.models_mut().insert(UndoHistory::with_limit(64)),
            coalesce: app.models_mut().insert(false),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();
        let undo_cmd: CommandId = act::Undo.into();
        let redo_cmd: CommandId = act::Redo.into();

        let value = self.value.watch(cx).paint().value_or_default();
        let history = self.history.watch(cx).paint().value_or_default();
        let can_undo = history.can_undo();
        let can_redo = history.can_redo();

        let next_undo = history
            .peek_undo()
            .and_then(|rec| rec.label.as_deref())
            .unwrap_or("None");
        let next_redo = history
            .peek_redo()
            .and_then(|rec| rec.label.as_deref())
            .unwrap_or("None");

        let coalesce = self.coalesce.watch(cx).paint().value_or_default();
        let coalesce_label = if coalesce { "On" } else { "Off" };

        let undo_shortcut = cx
            .app
            .global::<KeymapService>()
            .and_then(|svc| {
                svc.keymap
                    .display_shortcut_for_command_sequence(&InputContext::default(), &undo_cmd)
            })
            .map(|seq| format_sequence(Platform::current(), &seq))
            .unwrap_or_else(|| "Unbound".to_string());

        let redo_shortcut = cx
            .app
            .global::<KeymapService>()
            .and_then(|svc| {
                svc.keymap
                    .display_shortcut_for_command_sequence(&InputContext::default(), &redo_cmd)
            })
            .map(|seq| format_sequence(Platform::current(), &seq))
            .unwrap_or_else(|| "Unbound".to_string());

        let row_shortcuts = ui::v_flex(|cx| {
            ui::children![cx;
                ui::h_flex(|cx| {
                    ui::children![cx;
                        shadcn::Label::new("Undo shortcut:"),
                        shadcn::Badge::new(undo_shortcut)
                            .variant(shadcn::BadgeVariant::Secondary)
                            .test_id(TEST_ID_UNDO_SHORTCUT),
                    ]
                })
                .gap(Space::N2)
                .items_center(),
                ui::h_flex(|cx| {
                    ui::children![cx;
                        shadcn::Label::new("Redo shortcut:"),
                        shadcn::Badge::new(redo_shortcut)
                            .variant(shadcn::BadgeVariant::Secondary)
                            .test_id(TEST_ID_REDO_SHORTCUT),
                    ]
                })
                .gap(Space::N2)
                .items_center(),
            ]
        })
        .gap(Space::N2);

        let value_el = ui::text(format!("{value}"))
            .text_base()
            .tabular_nums()
            .font_weight(FontWeight::SEMIBOLD)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::SpinButton)
                    .test_id(TEST_ID_VALUE)
                    .numeric_value(value as f64),
            );

        let row_value = ui::v_flex(|cx| ui::children![cx; shadcn::Label::new("Value"), value_el])
            .gap(Space::N1);

        let row_edits = ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Button::new("-1")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::Dec)
                    .test_id(TEST_ID_DEC),
                shadcn::Button::new("+1")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::Inc)
                    .test_id(TEST_ID_INC),
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::Reset)
                    .test_id(TEST_ID_RESET),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let row_undo = ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Button::new("Undo")
                    .disabled(!can_undo)
                    .variant(shadcn::ButtonVariant::Default)
                    .action(act::Undo)
                    .test_id(TEST_ID_UNDO),
                shadcn::Button::new("Redo")
                    .disabled(!can_redo)
                    .variant(shadcn::ButtonVariant::Default)
                    .action(act::Redo)
                    .test_id(TEST_ID_REDO),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let row_coalesce = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Label::new("Coalesce nudges (key = \"value\"):"),
                shadcn::Switch::new(self.coalesce.clone()).test_id(TEST_ID_COALESCE),
                shadcn::Badge::new(coalesce_label).variant(shadcn::BadgeVariant::Secondary),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let row_next = ui::v_flex(|cx| {
            ui::children![cx;
                ui::text(format!("Next undo: {next_undo}"))
                    .text_sm()
                    .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                    .test_id(TEST_ID_NEXT_UNDO),
                ui::text(format!("Next redo: {next_redo}"))
                    .text_sm()
                    .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                    .test_id(TEST_ID_NEXT_REDO),
            ]
        })
        .gap(Space::N1);

        let content = ui::v_flex(|cx| {
            ui::children![cx; row_shortcuts, row_value, row_edits, row_undo, row_coalesce, row_next]
        })
        .gap(Space::N4);

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Undo basics"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "Shows an app-owned undo/redo history wired to edit.undo/edit.redo commands.",
                        ),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(cx, content);
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(760.0));

        cx.actions().models::<act::Inc>({
            let value = self.value.clone();
            let history = self.history.clone();
            let coalesce = self.coalesce.clone();
            move |models| {
                let coalesce = models.read(&coalesce, |v| *v).ok().unwrap_or(false);
                let after = models
                    .read(&value, |v| v.saturating_add(1))
                    .ok()
                    .unwrap_or(1);
                record_value_tx(
                    models,
                    &value,
                    &history,
                    "Increment",
                    coalesce.then_some("value"),
                    after,
                );
                true
            }
        });

        cx.actions().models::<act::Dec>({
            let value = self.value.clone();
            let history = self.history.clone();
            let coalesce = self.coalesce.clone();
            move |models| {
                let coalesce = models.read(&coalesce, |v| *v).ok().unwrap_or(false);
                let after = models
                    .read(&value, |v| v.saturating_sub(1))
                    .ok()
                    .unwrap_or(-1);
                record_value_tx(
                    models,
                    &value,
                    &history,
                    "Decrement",
                    coalesce.then_some("value"),
                    after,
                );
                true
            }
        });

        cx.actions().models::<act::Reset>({
            let value = self.value.clone();
            let history = self.history.clone();
            move |models| {
                record_value_tx(models, &value, &history, "Reset", None, 0);
                true
            }
        });

        // `Undo`/`Redo` stay on the advanced helper because history traversal is coupled
        // to a host-side RAF effect for immediate visual refresh.
        cx.on_action_notify::<act::Undo>({
            let value = self.value.clone();
            let history = self.history.clone();
            move |host, acx| {
                let next_value = host
                    .models_mut()
                    .update(&history, |h| {
                        let mut next = None;
                        let _ = h.undo_invertible(|rec| {
                            next = Some(rec.tx.after);
                            Ok::<(), ()>(())
                        });
                        next
                    })
                    .ok()
                    .flatten();

                let Some(next_value) = next_value else {
                    return false;
                };

                let _ = host.models_mut().update(&value, |v| *v = next_value);
                host.push_effect(Effect::RequestAnimationFrame(acx.window));
                true
            }
        });

        cx.on_action_notify::<act::Redo>({
            let value = self.value.clone();
            let history = self.history.clone();
            move |host, acx| {
                let next_value = host
                    .models_mut()
                    .update(&history, |h| {
                        let mut next = None;
                        let _ = h.redo_invertible(|rec| {
                            next = Some(rec.tx.after);
                            Ok::<(), ()>(())
                        });
                        next
                    })
                    .ok()
                    .flatten();

                let Some(next_value) = next_value else {
                    return false;
                };

                let _ = host.models_mut().update(&value, |v| *v = next_value);
                host.push_effect(Effect::RequestAnimationFrame(acx.window));
                true
            }
        });

        cx.actions().availability::<act::Undo>({
            let history = self.history.clone();
            move |host, _acx| {
                let can = host
                    .models_mut()
                    .read(&history, |h| h.can_undo())
                    .ok()
                    .unwrap_or(false);
                if can {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
        });

        cx.actions().availability::<act::Redo>({
            let history = self.history.clone();
            move |host, _acx| {
                let can = host
                    .models_mut()
                    .read(&history, |h| h.can_redo())
                    .ok()
                    .unwrap_or(false);
                if can {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
        });

        fret_cookbook::scaffold::centered_page_background_ui(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-undo-basics")
        .window("cookbook-undo-basics", (900.0, 560.0))
        .config_files(false)
        .install_app(install_commands)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<UndoBasicsView>()
        .map_err(anyhow::Error::from)
}
