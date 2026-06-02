use fret_app::App;
use fret_ui::Theme;
use serde::Deserialize;

use super::v30::{
    ColorSchemeOptions, DynamicVariant, SchemeMode, TypographyOptions, theme_config_with_colors,
};

const FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_token_visual_cases_v1.json"
));

#[derive(Debug, Deserialize)]
pub(super) struct Suite {
    pub schema_version: u32,
    pub cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Case {
    pub id: String,
    pub component: Component,
    pub scheme: Scheme,
    pub input: Input,
    pub assertions: Vec<Assertion>,
}

impl Case {
    pub fn is_expressive_scheme(&self) -> bool {
        matches!(self.scheme.variant, DynamicVariantFixture::Expressive)
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Component {
    Autocomplete,
    Badge,
    BottomSheet,
    Button,
    Card,
    CarouselItem,
    Checkbox,
    Chip,
    DatePicker,
    Dialog,
    Divider,
    DropdownMenu,
    ExposedDropdown,
    Fab,
    FilterChip,
    IconButton,
    InputChip,
    List,
    Menu,
    ModalNavigationDrawer,
    NavigationBar,
    NavigationDrawer,
    NavigationRail,
    ProgressIndicator,
    Radio,
    SearchBar,
    SearchView,
    SegmentedButton,
    Select,
    Slider,
    Snackbar,
    SuggestionChip,
    Switch,
    Tabs,
    TextField,
    TimePicker,
    Tooltip,
    TopAppBar,
}

#[derive(Debug, Deserialize)]
pub(super) struct Scheme {
    pub mode: SchemeModeFixture,
    pub variant: DynamicVariantFixture,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SchemeModeFixture {
    Light,
    Dark,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DynamicVariantFixture {
    TonalSpot,
    Expressive,
}

#[derive(Debug, Deserialize)]
pub(super) struct Input {
    pub variant: String,
    pub enabled: Option<bool>,
    pub interaction: Option<String>,
    #[serde(default)]
    pub hovered: bool,
    #[serde(default)]
    pub focused: bool,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub error: bool,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub toggle: bool,
    #[serde(default)]
    pub scrolled: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct Assertion {
    pub role: String,
    pub kind: String,
    pub token: Option<String>,
    pub source_token: Option<String>,
    pub color_token: Option<String>,
    pub opacity_token: Option<String>,
    pub base_color_token: Option<String>,
    pub overlay_color_token: Option<String>,
    pub value: Option<f32>,
}

pub(super) fn load_suite() -> Suite {
    serde_json::from_str(FIXTURE).expect("fixture JSON must parse")
}

pub(super) fn theme_for(scheme: &Scheme) -> Theme {
    let colors = ColorSchemeOptions {
        mode: match scheme.mode {
            SchemeModeFixture::Light => SchemeMode::Light,
            SchemeModeFixture::Dark => SchemeMode::Dark,
        },
        variant: match scheme.variant {
            DynamicVariantFixture::TonalSpot => DynamicVariant::TonalSpot,
            DynamicVariantFixture::Expressive => DynamicVariant::Expressive,
        },
        ..Default::default()
    };
    let cfg = theme_config_with_colors(TypographyOptions::default(), colors);
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));
    Theme::global(&app).clone()
}
