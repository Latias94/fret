use fret::app::RenderContextAccess as _;
use fret::app::prelude::*;
use fret::app::{LocalState, LocalStateTxn};
use fret::commands::{
    CommandAvailability, CommandId, CommandMeta, CommandScope, DefaultKeybinding, InputContext,
    KeyChord, KeyCode, KeymapService, Modifiers, Platform, PlatformFilter, format_sequence,
    install_command_default_keybindings_into_keymap,
};
use fret::semantics::SemanticsDecoration;
use fret::semantics::SemanticsRole;
use fret::style::{ColorRef, FontWeight, Space};
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
    value: LocalState<i32>,
    history: LocalState<UndoHistory<ValueTx<i32>>>,
    coalesce: LocalState<bool>,
}

fn install_commands(app: &mut App) {
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

    install_command_default_keybindings_into_keymap(app);
}

fn record_value_tx(
    tx: &mut LocalStateTxn<'_>,
    value: &LocalState<i32>,
    history: &LocalState<UndoHistory<ValueTx<i32>>>,
    label: &'static str,
    coalesce_key: Option<&'static str>,
    after: i32,
) -> bool {
    let before = tx.value_or(value, 0);
    if before == after {
        return false;
    }

    let value_changed = tx.set(value, after);
    let history_changed = tx.update(history, |h| {
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
    value_changed || history_changed
}

fn undo_value_tx(
    tx: &mut LocalStateTxn<'_>,
    value: &LocalState<i32>,
    history: &LocalState<UndoHistory<ValueTx<i32>>>,
) -> bool {
    let mut next_value = None;
    let history_changed = tx.update_if(history, |h| {
        h.undo_invertible(|rec| {
            next_value = Some(rec.tx.after);
            Ok::<(), ()>(())
        })
        .ok()
        .unwrap_or(false)
    });
    let Some(next_value) = next_value else {
        return false;
    };
    tx.set(value, next_value) || history_changed
}

fn redo_value_tx(
    tx: &mut LocalStateTxn<'_>,
    value: &LocalState<i32>,
    history: &LocalState<UndoHistory<ValueTx<i32>>>,
) -> bool {
    let mut next_value = None;
    let history_changed = tx.update_if(history, |h| {
        h.redo_invertible(|rec| {
            next_value = Some(rec.tx.after);
            Ok::<(), ()>(())
        })
        .ok()
        .unwrap_or(false)
    });
    let Some(next_value) = next_value else {
        return false;
    };
    tx.set(value, next_value) || history_changed
}

impl View for UndoBasicsView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        Self {
            value: app.local_state(0),
            history: app.local_state(UndoHistory::with_limit(64)),
            coalesce: app.local_state(false),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = cx.theme_snapshot();
        let undo_cmd: CommandId = act::Undo.into();
        let redo_cmd: CommandId = act::Redo.into();

        let value = self.value.paint(cx).value_or_default();
        let history = self.history.paint(cx).value_or_default();
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

        let coalesce = self.coalesce.paint(cx).value_or_default();
        let coalesce_label = if coalesce { "On" } else { "Off" };

        let undo_shortcut = cx
            .app()
            .global::<KeymapService>()
            .and_then(|svc| {
                svc.keymap
                    .display_shortcut_for_command_sequence(&InputContext::default(), &undo_cmd)
            })
            .map(|seq| format_sequence(Platform::current(), &seq))
            .unwrap_or_else(|| "Unbound".to_string());

        let redo_shortcut = cx
            .app()
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

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Undo basics"),
                        shadcn::card_description(
                            "Shows an app-owned undo/redo history wired to edit.undo/edit.redo commands.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| ui::single(cx, content)),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(760.0));

        cx.actions()
            .locals_with((&self.value, &self.history, &self.coalesce))
            .on::<act::Inc>(|tx, (value, history, coalesce)| {
                let coalesce = tx.value_or(&coalesce, false);
                let after = tx.value_or(&value, 0).saturating_add(1);
                record_value_tx(
                    tx,
                    &value,
                    &history,
                    "Increment",
                    coalesce.then_some("value"),
                    after,
                )
            });

        cx.actions()
            .locals_with((&self.value, &self.history, &self.coalesce))
            .on::<act::Dec>(|tx, (value, history, coalesce)| {
                let coalesce = tx.value_or(&coalesce, false);
                let after = tx.value_or(&value, 0).saturating_sub(1);
                record_value_tx(
                    tx,
                    &value,
                    &history,
                    "Decrement",
                    coalesce.then_some("value"),
                    after,
                )
            });

        cx.actions()
            .locals_with((&self.value, &self.history))
            .on::<act::Reset>(|tx, (value, history)| {
                record_value_tx(tx, &value, &history, "Reset", None, 0)
            });

        cx.actions()
            .locals_with((&self.value, &self.history))
            .on::<act::Undo>(|tx, (value, history)| undo_value_tx(tx, &value, &history));

        cx.actions()
            .locals_with((&self.value, &self.history))
            .on::<act::Redo>(|tx, (value, history)| redo_value_tx(tx, &value, &history));

        cx.actions()
            .locals_with(&self.history)
            .availability::<act::Undo>(|tx, history| {
                if tx.value(&history).can_undo() {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            });

        cx.actions()
            .locals_with(&self.history)
            .availability::<act::Redo>(|tx, history| {
                if tx.value(&history).can_redo() {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            });

        fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-undo-basics")
        .window("cookbook-undo-basics", (900.0, 560.0))
        .config_files(false)
        .setup(install_commands)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<UndoBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
