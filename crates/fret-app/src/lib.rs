pub mod app;
pub mod app_display_name;
pub mod config_files;
pub mod config_watcher;
pub mod core_commands;
pub mod dock_layout_file;
pub mod drag;
pub mod font_catalog_cache;
pub mod keymap;
pub mod menu;
pub mod menu_bar;
pub mod plugins;
pub mod settings;
pub mod ui_host;
pub mod when_expr;

pub use app::App;
pub use app_display_name::AppDisplayName;
pub use font_catalog_cache::FontCatalogCache;

pub use fret_runtime::{
    ActivationPolicy, CommandId, CommandMeta, CommandRegistry, CommandScope, CreateWindowKind,
    CreateWindowRequest, DRAG_KIND_DOCK_PANEL, DefaultKeybinding, DockDragInversionModifier,
    DockDragInversionPolicy, DockDragInversionSettings, DockingInteractionSettings, DragKindId,
    DragPhase, DragSession, DragSessionId, Effect, InputContext, KeyChord, Keymap, KeymapService,
    Menu, MenuBar, MenuItem, MenuRole, Model, ModelCx, ModelId, ModelStore, ModelUpdateError,
    OsAction, Platform, PlatformFilter, SystemMenuType, TaskbarVisibility, WhenExpr, WindowRequest,
    WindowRole, WindowStyleRequest, WindowZLevel, format_chord, format_sequence,
};

pub use keymap::KeymapError;
pub use keymap::KeymapFileError;
pub use keymap::apply_layered_keymap;
pub use keymap::install_command_default_keybindings_into_keymap;
pub use keymap::{BindingV1, KeySpecV1, KeymapFileV1};

pub use menu_bar::{
    LayeredMenuBarConfig, MenuBarBaselineService, MenuBarFileError, apply_layered_menu_bar,
    effective_menu_bar, menu_bar_from_file_if_exists, set_menu_bar_baseline,
    should_publish_os_menu_bar, should_render_in_window_menu_bar, sync_os_menu_bar,
};

pub use plugins::{Plugin, PluginHost, PluginId, PluginRegistrar, install_plugins};

pub use settings::{
    DockDragInversionModifierV1, DockDragInversionPolicyV1, DockDragInversionSettingsV1,
    DockingSettingsV1, FontsSettingsV1, LocaleSettingsV1, MenuBarIntegrationModeV1,
    MenuBarSettingsV1, SettingsError, SettingsFileV1,
};

pub use dock_layout_file::{DockLayoutError, DockLayoutFileV1};

pub use config_files::{
    KEYMAP_JSON, LayeredConfigPaths, LayeredKeymapReport, LayeredMenuBarReport,
    LayeredSettingsReport, MENUBAR_JSON, PROJECT_CONFIG_DIR, SETTINGS_JSON,
    default_user_config_dir, load_layered_keymap, load_layered_menu_bar, load_layered_settings,
};

pub use config_watcher::{
    ConfigFilesWatcher, ConfigFilesWatcherStatus, ConfigFilesWatcherTick,
    handle_config_files_watcher_timer,
};
