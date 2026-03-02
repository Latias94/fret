use fret::prelude::*;
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

struct OverlayBasicsState {
    dialog_open: Model<bool>,
    underlay_bumps: Model<u32>,
}

struct OverlayBasicsProgram;

impl MvuProgram for OverlayBasicsProgram {
    type State = OverlayBasicsState;
    type Message = ();

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        Self::State {
            dialog_open: app.models_mut().insert(false),
            underlay_bumps: app.models_mut().insert(0),
        }
    }

    fn update(_app: &mut App, _state: &mut Self::State, _message: Self::Message) {}

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        _msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let base_root = cx.root_id();
        let cmd_bump: CommandId = act::BumpUnderlay.into();

        let theme = Theme::global(&*cx.app).snapshot();
        let dialog_open_for_dialog = state.dialog_open.clone();
        let dialog_open_for_footer = state.dialog_open.clone();
        let dialog_open_for_close = state.dialog_open.clone();

        let bumps = cx.watch_model(&state.underlay_bumps).layout().copied_or(0);
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

        let dialog_open_for_action_open = state.dialog_open.clone();
        let underlay_bumps_for_action = state.underlay_bumps.clone();
        let (on_command, on_command_availability) = fret::actions::ActionHandlerTable::new()
            .on::<act::OpenDialog>(move |host, acx| {
                let _ = host
                    .models_mut()
                    .update(&dialog_open_for_action_open, |v| *v = true);
                host.request_redraw(acx.window);
                true
            })
            .on::<act::BumpUnderlay>(move |host, acx| {
                let _ = host.models_mut().update(&underlay_bumps_for_action, |v| {
                    *v = v.saturating_add(1);
                });
                host.request_redraw(acx.window);
                true
            })
            .availability::<act::OpenDialog>(|_host, _acx| CommandAvailability::Available)
            .availability::<act::BumpUnderlay>(|_host, _acx| CommandAvailability::Available)
            .build();

        cx.command_on_command_for(base_root, on_command);
        cx.command_on_command_availability_for(base_root, on_command_availability);

        let dialog = shadcn::Dialog::new(dialog_open_for_dialog).into_element(
            cx,
            move |cx| {
                let content = ui::v_flex(cx, |cx| {
                    let bump = shadcn::Button::new("Bump underlay")
                        .variant(shadcn::ButtonVariant::Outline)
                        .action(act::BumpUnderlay)
                        .into_element(cx)
                        .a11y_role(SemanticsRole::Button)
                        .test_id(TEST_ID_UNDERLAY_BUMP);

                    [
                        shadcn::Button::new("Open dialog")
                            .variant(shadcn::ButtonVariant::Outline)
                            .action(act::OpenDialog)
                            .into_element(cx)
                            .a11y_role(SemanticsRole::Button)
                            .test_id(TEST_ID_DIALOG_TRIGGER),
                        ui::v_flex(cx, |cx| {
                            [
                                cx.text(format!("Shortcut: {shortcut}"))
                                    .test_id(TEST_ID_UNDERLAY_SHORTCUT),
                                cx.text(format!("Underlay: {enabled_label}")),
                                cx.text(format!("Underlay bumps: {bumps}"))
                                    .test_id(TEST_ID_UNDERLAY_BUMPS),
                                bump,
                            ]
                        })
                        .gap(Space::N2)
                        .into_element(cx),
                    ]
                })
                .gap(Space::N3)
                .items_center()
                .into_element(cx);

                shadcn::Card::new([
                    shadcn::CardHeader::new([
                        shadcn::CardTitle::new("Overlay basics").into_element(cx),
                        shadcn::CardDescription::new(
                            "A minimal Dialog example with stable test IDs.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new([content]).into_element(cx),
                ])
                .ui()
                .w_full()
                .max_w(Px(520.0))
                .into_element(cx)
            },
            move |cx| {
                shadcn::DialogContent::new([
                    shadcn::DialogHeader::new([
                        shadcn::DialogTitle::new("Hello overlays").into_element(cx),
                        shadcn::DialogDescription::new(
                            "This is a minimal dialog example with stable test IDs.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::DialogFooter::new([shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(dialog_open_for_footer.clone())
                        .into_element(cx)])
                    .into_element(cx),
                    // Order matters: `DialogClose` is absolutely-positioned and should be the last
                    // child so it stays above the Dialog content in hit testing.
                    shadcn::DialogClose::new(dialog_open_for_close.clone())
                        .into_element(cx)
                        .test_id(TEST_ID_DIALOG_CLOSE),
                ])
                .into_element(cx)
                .test_id(TEST_ID_DIALOG_CONTENT)
            },
        );

        ui::container(cx, |cx| {
            [ui::v_flex(cx, |_cx| [dialog])
                .gap(Space::N6)
                .items_center()
                .justify_center()
                .size_full()
                .into_element(cx)]
        })
        .bg(ColorRef::Color(theme.color_token("muted")))
        .p(Space::N6)
        .into_element(cx)
        .test_id(TEST_ID_ROOT)
        .into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-overlay-basics")
        .window("cookbook-overlay-basics", (560.0, 420.0))
        .config_files(false)
        .install_app(install_commands)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_mvu::<OverlayBasicsProgram>()
        .map_err(anyhow::Error::from)
}
