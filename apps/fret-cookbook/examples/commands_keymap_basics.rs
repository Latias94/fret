use fret::prelude::*;
use fret_app::{
    CommandMeta, CommandScope, DefaultKeybinding, InputContext, KeyChord, KeymapService, Platform,
    PlatformFilter, format_sequence,
};
use fret_core::{KeyCode, Modifiers};
use fret_ui::CommandAvailability;
use fret_ui::action::UiActionHost;

mod act {
    fret::actions!([TogglePanel = "cookbook.commands.toggle_panel.v1"]);
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

fn toggle_panel(host: &mut dyn UiActionHost, window: AppWindowId, panel_open: &Model<bool>) {
    let _ = host.models_mut().update(panel_open, |v| *v = !*v);
    host.request_redraw(window);
    host.push_effect(Effect::RequestAnimationFrame(window));
}

struct CommandsKeymapBasicsView {
    panel_open: Model<bool>,
    allow_command: Model<bool>,
}

impl View for CommandsKeymapBasicsView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        Self {
            panel_open: app.models_mut().insert(false),
            allow_command: app.models_mut().insert(true),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let cmd: CommandId = act::TogglePanel.into();

        let panel_open = cx.watch_model(&self.panel_open).layout().copied_or(false);

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

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Commands + keymap basics").into_element(cx),
            shadcn::CardDescription::new(
                "Registers a command with a default keybinding, then gates availability from UI state.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let row_shortcut = ui::h_flex(|cx| {
            [
                shadcn::Label::new("Shortcut:").into_element(cx),
                shadcn::Badge::new(shortcut)
                    .variant(shadcn::BadgeVariant::Secondary)
                    .test_id(TEST_ID_SHORTCUT)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let row_enabled = ui::h_flex(|cx| {
            [
                shadcn::Label::new("Command state:").into_element(cx),
                shadcn::Badge::new(enabled_label)
                    .variant(if enabled {
                        shadcn::BadgeVariant::Default
                    } else {
                        shadcn::BadgeVariant::Destructive
                    })
                    .test_id(TEST_ID_ENABLED)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let row_allow = ui::h_flex(|cx| {
            [
                shadcn::Label::new("Allow command:").into_element(cx),
                shadcn::Switch::new(self.allow_command.clone())
                    .test_id(TEST_ID_ALLOW)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let dispatch_button = shadcn::Button::new("Dispatch command")
            .variant(shadcn::ButtonVariant::Outline)
            .action(act::TogglePanel)
            .a11y_role(SemanticsRole::Button)
            .test_id(TEST_ID_DISPATCH)
            .into_element(cx);

        let left = ui::v_flex(|_cx| [row_shortcut, row_enabled, row_allow, dispatch_button])
            .gap(Space::N3)
            .w_full()
            .into_element(cx);

        let panel_state_text = cx
            .text(format!(
                "Panel: {}",
                if panel_open { "Open" } else { "Closed" }
            ))
            .test_id(TEST_ID_PANEL_STATE);
        let panel_open_indicator = shadcn::Switch::new(self.panel_open.clone())
            .disabled(true)
            .test_id(TEST_ID_PANEL_OPEN)
            .into_element(cx);

        let panel_body = ui::v_flex(|cx| {
            let desc = if panel_open {
                "This panel is toggled by the command handler."
            } else {
                "Press the shortcut or the button to open it."
            };
            [
                panel_state_text,
                ui::h_flex(|cx| {
                    [
                        shadcn::Label::new("Open:").into_element(cx),
                        panel_open_indicator,
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
                shadcn::Separator::new().into_element(cx),
                cx.text(desc),
            ]
        })
        .gap(Space::N2)
        .into_element(cx);

        let panel = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Panel").into_element(cx),
                shadcn::CardDescription::new("State changes should be command-driven.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([panel_body]).into_element(cx),
        ])
        .ui()
        .w_full()
        .into_element(cx)
        .test_id(TEST_ID_PANEL);

        let body = ui::h_flex(|_cx| [left, panel])
            .gap(Space::N6)
            .w_full()
            .into_element(cx);

        let card = shadcn::Card::new([header, shadcn::CardContent::new([body]).into_element(cx)])
            .ui()
            .w_full()
            .max_w(Px(860.0))
            .into_element(cx)
            .key_context("cookbook.commands_keymap_basics");

        cx.on_action_notify::<act::TogglePanel>({
            let panel_open = self.panel_open.clone();
            let allow = self.allow_command.clone();
            move |host, acx| {
                let allowed = host.models_mut().get_copied(&allow).unwrap_or(true);
                if !allowed {
                    return false;
                }

                toggle_panel(host, acx.window, &panel_open);
                true
            }
        });

        cx.on_action_availability::<act::TogglePanel>({
            let allow = self.allow_command.clone();
            move |host, _acx| {
                let allowed = host.models_mut().get_copied(&allow).unwrap_or(true);
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
        .install_app(install_commands)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<CommandsKeymapBasicsView>()
        .map_err(anyhow::Error::from)
}
