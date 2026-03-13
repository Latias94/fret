use fret::app::prelude::*;
use fret_app::{
    CommandMeta, CommandScope, DefaultKeybinding, InputContext, KeyChord, KeymapService, Platform,
    PlatformFilter, format_sequence,
};
use fret_core::{KeyCode, Modifiers};
use fret_ui::CommandAvailability;

mod act {
    fret::actions!([
        OpenDialog = "cookbook.overlays.dialog.open.v1",
        BumpUnderlay = "cookbook.overlays.underlay.bump.v1"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.overlays.root";
const TEST_ID_DIALOG_TRIGGER: &str = "cookbook.overlays.dialog.trigger";
const TEST_ID_DIALOG_CONTENT: &str = "cookbook.overlays.dialog.content";
const TEST_ID_DIALOG_CLOSE: &str = "cookbook.overlays.dialog.close";
const TEST_ID_UNDERLAY_SHORTCUT: &str = "cookbook.overlays.underlay.shortcut";
const TEST_ID_UNDERLAY_BUMPS: &str = "cookbook.overlays.underlay.bumps";
const TEST_ID_UNDERLAY_BUMP: &str = "cookbook.overlays.underlay.bump";

fn install_commands(app: &mut App) {
    let cmd: CommandId = act::BumpUnderlay.into();
    let meta = CommandMeta::new("Bump underlay")
        .with_description(
            "Increments a counter. Intended to be blocked while a modal dialog is open.",
        )
        .with_category("Cookbook")
        .with_scope(CommandScope::Widget)
        .with_default_keybindings([
            DefaultKeybinding::single(
                PlatformFilter::Macos,
                KeyChord::new(
                    KeyCode::KeyU,
                    Modifiers {
                        meta: true,
                        shift: true,
                        ..Modifiers::default()
                    },
                ),
            ),
            DefaultKeybinding::single(
                PlatformFilter::All,
                KeyChord::new(
                    KeyCode::KeyU,
                    Modifiers {
                        ctrl: true,
                        shift: true,
                        ..Modifiers::default()
                    },
                ),
            ),
        ]);

    app.commands_mut().register(cmd, meta);
    fret_app::install_command_default_keybindings_into_keymap(app);
}

struct OverlayBasicsView;

impl View for OverlayBasicsView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let cmd_bump: CommandId = act::BumpUnderlay.into();

        let dialog_open_state = cx.state().local::<bool>();
        let underlay_bumps_state = cx.state().local::<u32>();

        cx.actions()
            .local_set::<act::OpenDialog, bool>(&dialog_open_state, true);
        cx.actions()
            .local_update::<act::BumpUnderlay, u32>(&underlay_bumps_state, |v| {
                *v = v.saturating_add(1);
            });
        cx.actions()
            .availability::<act::OpenDialog>(|_host, _acx| CommandAvailability::Available);
        cx.actions()
            .availability::<act::BumpUnderlay>(|_host, _acx| CommandAvailability::Available);

        let dialog_open_for_dialog = dialog_open_state.clone_model();
        let dialog_open_for_footer = dialog_open_state.clone_model();
        let dialog_open_for_close = dialog_open_state.clone_model();

        let bumps = cx.state().watch(&underlay_bumps_state).layout().value_or(0);
        let enabled = cx.action_is_enabled(&cmd_bump);
        let enabled_label = if enabled {
            "Enabled"
        } else {
            "Disabled (blocked by modal barrier)"
        };

        let shortcut = cx
            .app
            .global::<KeymapService>()
            .and_then(|svc| {
                svc.keymap
                    .display_shortcut_for_command_sequence(&InputContext::default(), &cmd_bump)
            })
            .map(|seq| format_sequence(Platform::current(), &seq))
            .unwrap_or_else(|| "Unbound".to_string());

        let dialog = shadcn::Dialog::new(dialog_open_for_dialog).into_element(
            cx,
            move |cx| {
                let content = ui::v_flex(|cx| {
                    ui::children![
                        cx;
                        shadcn::Button::new("Open dialog")
                            .variant(shadcn::ButtonVariant::Outline)
                            .action(act::OpenDialog)
                            .a11y_role(SemanticsRole::Button)
                            .test_id(TEST_ID_DIALOG_TRIGGER),
                        ui::v_flex(|cx| {
                            ui::children![
                                cx;
                                cx.text(format!("Shortcut: {shortcut}"))
                                    .test_id(TEST_ID_UNDERLAY_SHORTCUT),
                                cx.text(format!("Underlay: {enabled_label}")),
                                cx.text(format!("Underlay bumps: {bumps}"))
                                    .test_id(TEST_ID_UNDERLAY_BUMPS),
                                shadcn::Button::new("Bump underlay")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .action(act::BumpUnderlay)
                                    .a11y_role(SemanticsRole::Button)
                                    .test_id(TEST_ID_UNDERLAY_BUMP),
                            ]
                        })
                        .gap(Space::N2),
                    ]
                })
                .gap(Space::N3)
                .items_center();

                shadcn::card(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_header(|cx| {
                            ui::children![
                                cx;
                                shadcn::card_title("Overlay basics"),
                                shadcn::card_description(
                                    "A minimal Dialog example with stable test IDs.",
                                ),
                            ]
                        }),
                        shadcn::card_content(|cx| ui::children![cx; content]),
                    ]
                })
                .ui()
                .w_full()
                .max_w(Px(520.0))
                .into_element(cx)
            },
            move |cx| {
                shadcn::DialogContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        shadcn::DialogHeader::build(|cx, out| {
                            out.push_ui(cx, shadcn::DialogTitle::new("Hello overlays"));
                            out.push_ui(
                                cx,
                                shadcn::DialogDescription::new(
                                    "This is a minimal dialog example with stable test IDs.",
                                ),
                            );
                        }),
                    );
                    out.push_ui(
                        cx,
                        shadcn::DialogFooter::build(|cx, out| {
                            out.push_ui(
                                cx,
                                shadcn::Button::new("Close")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .toggle_model(dialog_open_for_footer.clone()),
                            );
                        }),
                    );
                    // Order matters: `DialogClose` is absolutely-positioned and should be the last
                    // child so it stays above the Dialog content in hit testing.
                    out.push_ui(
                        cx,
                        shadcn::DialogClose::new(dialog_open_for_close.clone())
                            .test_id(TEST_ID_DIALOG_CLOSE),
                    );
                })
                .show_close_button(false)
                .ui()
                .test_id(TEST_ID_DIALOG_CONTENT)
                .into_element(cx)
            },
        );

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, dialog).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-overlay-basics")
        .window("cookbook-overlay-basics", (560.0, 420.0))
        .config_files(false)
        .setup(install_commands)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<OverlayBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
