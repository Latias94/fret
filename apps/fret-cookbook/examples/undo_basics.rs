use std::sync::Arc;

use fret::prelude::*;
use fret_app::{
    CommandMeta, CommandScope, DefaultKeybinding, InputContext, KeyChord, KeymapService, Platform,
    PlatformFilter, format_sequence,
};
use fret_core::{FontWeight, KeyCode, Modifiers};
use fret_ui::{
    CommandAvailability,
    action::{OnCommand, OnCommandAvailability},
    element::SemanticsDecoration,
};
use fret_undo::{CMD_EDIT_REDO, CMD_EDIT_UNDO, UndoHistory, UndoRecord, ValueTx};

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

#[derive(Debug, Clone)]
enum Msg {
    Inc,
    Dec,
    Reset,
}

struct UndoBasicsState {
    value: Model<i32>,
    history: Model<UndoHistory<ValueTx<i32>>>,
    coalesce: Model<bool>,
}

struct UndoBasicsProgram;

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

    fret_app::install_command_default_keybindings_into_keymap(app);
}

fn record_value_tx(
    app: &mut App,
    value: &Model<i32>,
    history: &Model<UndoHistory<ValueTx<i32>>>,
    label: &'static str,
    coalesce_key: Option<&'static str>,
    after: i32,
) {
    let before = app.models().read(value, |v| *v).ok().unwrap_or_default();
    if before == after {
        return;
    }

    let _ = app.models_mut().update(value, |v| *v = after);
    let _ = app.models_mut().update(history, |h| {
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

fn command_handlers(
    value: Model<i32>,
    history: Model<UndoHistory<ValueTx<i32>>>,
) -> (OnCommand, OnCommandAvailability) {
    let undo_value = value.clone();
    let undo_history = history.clone();

    let on_command: OnCommand = Arc::new(move |host, acx, command| {
        let cmd = command.as_str();
        if cmd != CMD_EDIT_UNDO && cmd != CMD_EDIT_REDO {
            return false;
        }

        let next_value = host
            .models_mut()
            .update(&undo_history, |h| {
                let mut next = None;
                match cmd {
                    CMD_EDIT_UNDO => {
                        let _ = h.undo_invertible(|rec| {
                            next = Some(rec.tx.after);
                            Ok::<(), ()>(())
                        });
                    }
                    CMD_EDIT_REDO => {
                        let _ = h.redo_invertible(|rec| {
                            next = Some(rec.tx.after);
                            Ok::<(), ()>(())
                        });
                    }
                    _ => {}
                }
                next
            })
            .ok()
            .flatten();

        let Some(next_value) = next_value else {
            return false;
        };

        let _ = host.models_mut().update(&undo_value, |v| *v = next_value);
        host.request_redraw(acx.window);
        host.push_effect(Effect::RequestAnimationFrame(acx.window));
        true
    });

    let on_command_availability: OnCommandAvailability =
        Arc::new(move |host, _acx, command| match command.as_str() {
            CMD_EDIT_UNDO => {
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
            CMD_EDIT_REDO => {
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
            _ => CommandAvailability::NotHandled,
        });

    (on_command, on_command_availability)
}

impl MvuProgram for UndoBasicsProgram {
    type State = UndoBasicsState;
    type Message = Msg;

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        Self::State {
            value: app.models_mut().insert(0),
            history: app.models_mut().insert(UndoHistory::with_limit(64)),
            coalesce: app.models_mut().insert(false),
        }
    }

    fn update(app: &mut App, st: &mut Self::State, msg: Self::Message) {
        let coalesce = app
            .models()
            .read(&st.coalesce, |v| *v)
            .ok()
            .unwrap_or(false);

        match msg {
            Msg::Inc => {
                let after = app
                    .models()
                    .read(&st.value, |v| v.saturating_add(1))
                    .ok()
                    .unwrap_or(1);
                record_value_tx(
                    app,
                    &st.value,
                    &st.history,
                    "Increment",
                    coalesce.then_some("value"),
                    after,
                );
            }
            Msg::Dec => {
                let after = app
                    .models()
                    .read(&st.value, |v| v.saturating_sub(1))
                    .ok()
                    .unwrap_or(-1);
                record_value_tx(
                    app,
                    &st.value,
                    &st.history,
                    "Decrement",
                    coalesce.then_some("value"),
                    after,
                );
            }
            Msg::Reset => {
                record_value_tx(app, &st.value, &st.history, "Reset", None, 0);
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        // Attach command handlers to the window's declarative root so shortcuts work even when
        // nothing inside the view has focus (the dispatch path doesn't walk descendants).
        let base_root = cx.root_id();

        let theme = Theme::global(&*cx.app).snapshot();
        let undo_cmd = CommandId::from(CMD_EDIT_UNDO);
        let redo_cmd = CommandId::from(CMD_EDIT_REDO);

        let value = cx.watch_model(&state.value).paint().copied_or_default();
        let history = cx.watch_model(&state.history).paint().cloned_or_default();
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

        let coalesce = cx.watch_model(&state.coalesce).paint().copied_or_default();
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

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Undo basics").into_element(cx),
            shadcn::CardDescription::new(
                "Shows an app-owned undo/redo history wired to edit.undo/edit.redo commands.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let row_shortcuts = ui::v_flex(cx, |cx| {
            [
                ui::h_flex(cx, |cx| {
                    [
                        shadcn::Label::new("Undo shortcut:").into_element(cx),
                        shadcn::Badge::new(undo_shortcut)
                            .variant(shadcn::BadgeVariant::Secondary)
                            .into_element(cx)
                            .test_id(TEST_ID_UNDO_SHORTCUT),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
                ui::h_flex(cx, |cx| {
                    [
                        shadcn::Label::new("Redo shortcut:").into_element(cx),
                        shadcn::Badge::new(redo_shortcut)
                            .variant(shadcn::BadgeVariant::Secondary)
                            .into_element(cx)
                            .test_id(TEST_ID_REDO_SHORTCUT),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .into_element(cx);

        let value_el = ui::text(cx, format!("{value}"))
            .text_base()
            .tabular_nums()
            .font_weight(FontWeight::SEMIBOLD)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::SpinButton)
                    .test_id(TEST_ID_VALUE)
                    .numeric_value(value as f64),
            );

        let row_value = ui::v_flex(cx, |cx| {
            [shadcn::Label::new("Value").into_element(cx), value_el]
        })
        .gap(Space::N1)
        .into_element(cx);

        let inc_cmd = msg.cmd(Msg::Inc);
        let dec_cmd = msg.cmd(Msg::Dec);
        let reset_cmd = msg.cmd(Msg::Reset);

        let row_edits = ui::h_flex(cx, |cx| {
            [
                shadcn::Button::new("-1")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .on_click(dec_cmd)
                    .into_element(cx)
                    .test_id(TEST_ID_DEC),
                shadcn::Button::new("+1")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .on_click(inc_cmd)
                    .into_element(cx)
                    .test_id(TEST_ID_INC),
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(reset_cmd)
                    .into_element(cx)
                    .test_id(TEST_ID_RESET),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let row_undo = ui::h_flex(cx, |cx| {
            [
                shadcn::Button::new("Undo")
                    .disabled(!can_undo)
                    .variant(shadcn::ButtonVariant::Default)
                    .on_click(undo_cmd.clone())
                    .into_element(cx)
                    .test_id(TEST_ID_UNDO),
                shadcn::Button::new("Redo")
                    .disabled(!can_redo)
                    .variant(shadcn::ButtonVariant::Default)
                    .on_click(redo_cmd.clone())
                    .into_element(cx)
                    .test_id(TEST_ID_REDO),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let row_coalesce = ui::h_flex(cx, |cx| {
            [
                shadcn::Label::new("Coalesce nudges (key = \"value\"):").into_element(cx),
                shadcn::Switch::new(state.coalesce.clone())
                    .test_id(TEST_ID_COALESCE)
                    .into_element(cx),
                shadcn::Badge::new(coalesce_label)
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let row_next = ui::v_flex(cx, |cx| {
            [
                ui::text(cx, format!("Next undo: {next_undo}"))
                    .text_sm()
                    .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                    .into_element(cx)
                    .test_id(TEST_ID_NEXT_UNDO),
                ui::text(cx, format!("Next redo: {next_redo}"))
                    .text_sm()
                    .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                    .into_element(cx)
                    .test_id(TEST_ID_NEXT_REDO),
            ]
        })
        .gap(Space::N1)
        .into_element(cx);

        let content = ui::v_flex(cx, |_cx| {
            [
                row_shortcuts,
                row_value,
                row_edits,
                row_undo,
                row_coalesce,
                row_next,
            ]
        })
        .gap(Space::N4)
        .into_element(cx);

        let card =
            shadcn::Card::new([header, shadcn::CardContent::new([content]).into_element(cx)])
                .ui()
                .w_full()
                .max_w(Px(760.0))
                .into_element(cx);

        let (on_command, on_command_availability) =
            command_handlers(state.value.clone(), state.history.clone());

        let root = fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card);

        cx.command_on_command_for(base_root, on_command);
        cx.command_on_command_availability_for(base_root, on_command_availability);

        root.into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-undo-basics")
        .window("cookbook-undo-basics", (900.0, 560.0))
        .config_files(false)
        .install_app(install_commands)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_mvu::<UndoBasicsProgram>()
        .map_err(anyhow::Error::from)
}
