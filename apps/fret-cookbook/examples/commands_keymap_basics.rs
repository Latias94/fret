use fret::actions::{CommandId, ElementCommandGatingExt as _};
use fret::app::prelude::*;
use fret::children::UiElementSinkExt as _;
use fret::semantics::SemanticsRole;
use fret::style::Space;
use fret_app::{
    CommandMeta, CommandScope, DefaultKeybinding, InputContext, KeyChord, KeymapService, Platform,
    PlatformFilter, format_sequence,
};
use fret_core::{KeyCode, Modifiers};
use fret_ui::CommandAvailability;

mod act {
    fret::actions!([
        TogglePanel = "cookbook.commands.toggle_panel.v1",
        ToggleAllowCommand = "cookbook.commands.toggle_allow_command.v1",
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.commands_keymap_basics.root";
const TEST_ID_ALLOW: &str = "cookbook.commands_keymap_basics.allow";
const TEST_ID_SHORTCUT: &str = "cookbook.commands_keymap_basics.shortcut";
const TEST_ID_ENABLED: &str = "cookbook.commands_keymap_basics.enabled";
const TEST_ID_DISPATCH: &str = "cookbook.commands_keymap_basics.dispatch";
const TEST_ID_PANEL_STATE: &str = "cookbook.commands_keymap_basics.panel_state";
const TEST_ID_PANEL_OPEN: &str = "cookbook.commands_keymap_basics.panel_open";
const TEST_ID_PANEL: &str = "cookbook.commands_keymap_basics.panel";

fn install_commands(app: &mut App) {
    let cmd: CommandId = act::TogglePanel.into();
    let meta = CommandMeta::new("Toggle panel")
        .with_description("Toggles a panel via a registered action ID + default keybinding.")
        .with_category("Cookbook")
        .with_scope(CommandScope::Widget)
        .with_default_keybindings([
            DefaultKeybinding::single(
                PlatformFilter::Macos,
                KeyChord::new(
                    KeyCode::KeyK,
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
                    KeyCode::KeyK,
                    Modifiers {
                        ctrl: true,
                        shift: true,
                        ..Modifiers::default()
                    },
                ),
            ),
        ]);

    app.commands_mut().register(cmd, meta);

    // Ensure keybindings are installed after registering the command (the app may have already
    // installed defaults for previously-known commands during bootstrap).
    fret_app::install_command_default_keybindings_into_keymap(app);
}

struct CommandsKeymapBasicsView;

impl View for CommandsKeymapBasicsView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let panel_open_state = cx.state().local_init(|| false);
        let allow_command_state = cx.state().local_init(|| true);

        let cmd: CommandId = act::TogglePanel.into();

        let panel_open = panel_open_state.layout_value(cx);
        let allow_command = allow_command_state.layout_value(cx);

        let enabled = cx.action_is_enabled(&cmd);
        let enabled_label = if enabled { "Enabled" } else { "Disabled" };

        let shortcut = cx
            .app
            .global::<KeymapService>()
            .and_then(|svc| {
                svc.keymap
                    .display_shortcut_for_command_sequence(&InputContext::default(), &cmd)
            })
            .map(|seq| format_sequence(Platform::current(), &seq))
            .unwrap_or_else(|| "Unbound".to_string());

        let row_shortcut = ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Label::new("Shortcut:"),
                shadcn::Badge::new(shortcut)
                    .variant(shadcn::BadgeVariant::Secondary)
                    .test_id(TEST_ID_SHORTCUT),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let row_enabled = ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Label::new("Command state:"),
                shadcn::Badge::new(enabled_label)
                    .variant(if enabled {
                        shadcn::BadgeVariant::Default
                    } else {
                        shadcn::BadgeVariant::Destructive
                    })
                    .test_id(TEST_ID_ENABLED),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let row_allow = ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Label::new("Allow command:"),
                shadcn::Switch::from_checked(allow_command)
                    .action(act::ToggleAllowCommand)
                    .test_id(TEST_ID_ALLOW),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let dispatch_button = shadcn::Button::new("Dispatch command")
            .variant(shadcn::ButtonVariant::Outline)
            .action(act::TogglePanel)
            .a11y_role(SemanticsRole::Button)
            .test_id(TEST_ID_DISPATCH);

        let left = ui::v_flex_build(|cx, out| {
            out.push_ui(cx, row_shortcut);
            out.push_ui(cx, row_enabled);
            out.push_ui(cx, row_allow);
            out.push_ui(cx, dispatch_button);
        })
        .gap(Space::N3)
        .w_full();

        let panel_state_text = cx
            .text(format!(
                "Panel: {}",
                if panel_open { "Open" } else { "Closed" }
            ))
            .test_id(TEST_ID_PANEL_STATE);
        let panel_open_indicator = shadcn::Switch::from_checked(panel_open)
            .disabled(true)
            .test_id(TEST_ID_PANEL_OPEN);

        let panel_body = ui::v_flex(|cx| {
            let desc = if panel_open {
                "This panel is toggled by the command handler."
            } else {
                "Press the shortcut or the button to open it."
            };
            ui::children![
                cx;
                panel_state_text,
                ui::h_flex(
                    |cx| ui::children![cx; shadcn::Label::new("Open:"), panel_open_indicator],
                )
                .gap(Space::N2)
                .items_center(),
                shadcn::Separator::new(),
                cx.text(desc),
            ]
        })
        .gap(Space::N2);

        let panel = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Panel"),
                        shadcn::card_description("State changes should be command-driven."),
                    ]
                }),
                shadcn::card_content(|cx| ui::children![cx; panel_body]),
            ]
        })
        .ui()
        .w_full()
        .test_id(TEST_ID_PANEL);

        let body = ui::h_flex(|cx| ui::children![cx; left, panel])
            .gap(Space::N6)
            .w_full();

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Commands + keymap basics"),
                        shadcn::card_description(
                            "Registers a command with a default keybinding, then gates availability from UI state.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| ui::children![cx; body]),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(860.0))
        .key_context("cookbook.commands_keymap_basics");

        cx.actions()
            .local(&allow_command_state)
            .toggle_bool::<act::ToggleAllowCommand>();

        cx.actions()
            .locals_with((&panel_open_state, &allow_command_state))
            .on::<act::TogglePanel>(|tx, (panel_open_state, allow_command_state)| {
                let allowed = tx.value(&allow_command_state);
                if !allowed {
                    return false;
                }

                tx.update(&panel_open_state, |value| *value = !*value)
            });

        cx.actions().availability::<act::TogglePanel>({
            let allow_command_state = allow_command_state.clone();
            move |host, _acx| {
                let allowed = allow_command_state
                    .read_in(host.models_mut(), |value| *value)
                    .unwrap_or(true);
                if allowed {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
        });

        let root = fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card);

        root.into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-commands-keymap-basics")
        .window("cookbook-commands-keymap-basics", (920.0, 560.0))
        .config_files(false)
        .setup(install_commands)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<CommandsKeymapBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
