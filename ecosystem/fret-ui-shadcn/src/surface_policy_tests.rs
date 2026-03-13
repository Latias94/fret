const LIB_RS: &str = include_str!("lib.rs");
const APP_RS: &str = include_str!("app.rs");
const ADVANCED_RS: &str = include_str!("advanced.rs");
const README: &str = include_str!("../README.md");
const ACCORDION_RS: &str = include_str!("accordion.rs");
const ALERT_RS: &str = include_str!("alert.rs");
const BADGE_RS: &str = include_str!("badge.rs");
const BREADCRUMB_RS: &str = include_str!("breadcrumb.rs");
const COLLAPSIBLE_RS: &str = include_str!("collapsible.rs");
const CARD_RS: &str = include_str!("card.rs");
const EMPTY_RS: &str = include_str!("empty.rs");
const FIELD_RS: &str = include_str!("field.rs");
const UI_EXT_SUPPORT_RS: &str = include_str!("ui_ext/support.rs");
const UI_EXT_DATA_RS: &str = include_str!("ui_ext/data.rs");
const ALERT_DIALOG_RS: &str = include_str!("alert_dialog.rs");
const COMMAND_RS: &str = include_str!("command.rs");
const CONTEXT_MENU_RS: &str = include_str!("context_menu.rs");
const DIALOG_RS: &str = include_str!("dialog.rs");
const DRAWER_RS: &str = include_str!("drawer.rs");
const DROPDOWN_MENU_RS: &str = include_str!("dropdown_menu.rs");
const HOVER_CARD_RS: &str = include_str!("hover_card.rs");
const INPUT_RS: &str = include_str!("input.rs");
const INPUT_GROUP_RS: &str = include_str!("input_group.rs");
const INPUT_OTP_RS: &str = include_str!("input_otp.rs");
const KBD_RS: &str = include_str!("kbd.rs");
const MENUBAR_RS: &str = include_str!("menubar.rs");
const NAVIGATION_MENU_RS: &str = include_str!("navigation_menu.rs");
const PAGINATION_RS: &str = include_str!("pagination.rs");
const POPOVER_RS: &str = include_str!("popover.rs");
const SEPARATOR_RS: &str = include_str!("separator.rs");
const SHEET_RS: &str = include_str!("sheet.rs");
const SLIDER_RS: &str = include_str!("slider.rs");
const TABLE_RS: &str = include_str!("table.rs");
const TEXTAREA_RS: &str = include_str!("textarea.rs");
const TOGGLE_RS: &str = include_str!("toggle.rs");
const TOGGLE_GROUP_RS: &str = include_str!("toggle_group.rs");
const TOOLTIP_RS: &str = include_str!("tooltip.rs");
const TYPOGRAPHY_RS: &str = include_str!("typography.rs");
const CHECKBOX_RS: &str = include_str!("checkbox.rs");
const COMBOBOX_RS: &str = include_str!("combobox.rs");
const PROGRESS_RS: &str = include_str!("progress.rs");
const RADIO_GROUP_RS: &str = include_str!("radio_group.rs");
const RESIZABLE_RS: &str = include_str!("resizable.rs");
const SWITCH_RS: &str = include_str!("switch.rs");
const TABS_RS: &str = include_str!("tabs.rs");
const UI_BUILDER_EXT_BREADCRUMB_RS: &str = include_str!("ui_builder_ext/breadcrumb.rs");
const UI_BUILDER_EXT_COLLAPSIBLE_RS: &str = include_str!("ui_builder_ext/collapsible.rs");
const UI_BUILDER_EXT_COMMAND_DIALOG_RS: &str = include_str!("ui_builder_ext/command_dialog.rs");
const UI_BUILDER_EXT_DATA_RS: &str = include_str!("ui_builder_ext/data.rs");
const UI_BUILDER_EXT_MENUS_RS: &str = include_str!("ui_builder_ext/menus.rs");
const UI_BUILDER_EXT_OVERLAY_ROOTS_RS: &str = include_str!("ui_builder_ext/overlay_roots.rs");

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

fn public_anyelement_signatures(source: &str) -> Vec<String> {
    let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = Vec::new();
    let mut rest = normalized.as_str();

    while let Some(start) = rest.find("pub fn ") {
        let tail = &rest[start..];
        let end_brace = tail.find('{');
        let end_semi = tail.find(';');
        let end = match (end_brace, end_semi) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => tail.len(),
        };
        let signature = &tail[..end];
        if signature.contains("-> AnyElement") {
            out.push(signature.trim().to_owned());
        }
        rest = &tail[end..];
    }

    out
}

#[test]
fn app_integration_stays_under_explicit_app_module() {
    assert!(README.contains("`fret_ui_shadcn::app::{install, install_with, ...}`"));
    assert!(LIB_RS.contains("pub mod app;"));
    assert!(APP_RS.contains("pub struct InstallConfig"));
    assert!(APP_RS.contains("pub fn install(app: &mut fret_app::App)"));
    assert!(APP_RS.contains("pub fn install_with("));
    assert!(APP_RS.contains("pub fn install_with_theme("));
    assert!(!APP_RS.contains("sync_theme_from_environment"));
    assert!(!APP_RS.contains("install_with_ui_services"));
    assert!(!README.contains("`fret_ui_shadcn::install_app(...)`"));
}

#[test]
fn curated_facade_keeps_app_theme_and_raw_seams_explicit() {
    assert!(README.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));
    assert!(README.contains("let _button = shadcn::Button::new(\"Save\");"));
    assert!(!README.contains("recipes/components stay under `fret_ui_shadcn::*`"));

    assert!(LIB_RS.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));
    assert!(LIB_RS.contains("crate root as a raw escape hatch"));
    assert!(LIB_RS.contains("pub mod facade {"));
    assert!(LIB_RS.contains("pub mod themes {"));
    assert!(LIB_RS.contains("pub mod raw {"));
    assert!(LIB_RS.contains(
        "pub use crate::app::{InstallConfig, install, install_with, install_with_theme};"
    ));
    assert!(LIB_RS.contains("pub use crate::shadcn_themes::{"));
    assert!(LIB_RS.contains("pub use crate::*;"));
    assert!(LIB_RS.contains("pub mod advanced;"));
    assert!(LIB_RS.contains("pub use fret_ui_kit::IntoUiElement;"));
    assert!(LIB_RS.contains("UiElementTestIdExt"));
    assert!(LIB_RS.contains("UiElementA11yExt"));
    assert!(LIB_RS.contains("UiElementKeyContextExt"));
    assert!(ADVANCED_RS.contains("pub fn sync_theme_from_environment("));
    assert!(ADVANCED_RS.contains("pub fn install_with_ui_services("));
}

#[test]
fn ui_ext_glue_prefers_unified_component_conversion_trait() {
    for (label, source) in [
        ("ui_ext/support.rs", UI_EXT_SUPPORT_RS),
        ("ui_ext/data.rs", UI_EXT_DATA_RS),
    ] {
        assert!(
            !source.contains("::fret_ui_kit::UiIntoElement"),
            "{label} reintroduced direct UiIntoElement glue"
        );
        assert!(
            source.contains("::fret_ui_kit::IntoUiElement<H>"),
            "{label} should use the unified IntoUiElement<H> glue"
        );
    }
}

#[test]
fn ui_builder_ext_closures_accept_unified_component_conversion_trait() {
    for (label, source) in [
        ("ui_builder_ext/breadcrumb.rs", UI_BUILDER_EXT_BREADCRUMB_RS),
        (
            "ui_builder_ext/collapsible.rs",
            UI_BUILDER_EXT_COLLAPSIBLE_RS,
        ),
        (
            "ui_builder_ext/command_dialog.rs",
            UI_BUILDER_EXT_COMMAND_DIALOG_RS,
        ),
        ("ui_builder_ext/data.rs", UI_BUILDER_EXT_DATA_RS),
        ("ui_builder_ext/menus.rs", UI_BUILDER_EXT_MENUS_RS),
        (
            "ui_builder_ext/overlay_roots.rs",
            UI_BUILDER_EXT_OVERLAY_ROOTS_RS,
        ),
    ] {
        assert!(
            source.contains("IntoUiElement<H>"),
            "{label} should accept the unified component conversion trait"
        );
    }
}

#[test]
fn ui_builder_ext_keep_into_element_as_explicit_landing_seam() {
    for (label, source, required_markers) in [
        (
            "ui_builder_ext/breadcrumb.rs",
            UI_BUILDER_EXT_BREADCRUMB_RS,
            &[
                "fn into_element<H: UiHost, I, TChild>( self, cx: &mut ElementContext<'_, H>, children: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = TChild>, TChild: IntoUiElement<H>;",
            ][..],
        ),
        (
            "ui_builder_ext/collapsible.rs",
            UI_BUILDER_EXT_COLLAPSIBLE_RS,
            &[
                "fn into_element<H: UiHost, TTrigger, TContent>( self, cx: &mut ElementContext<'_, H>, trigger: impl FnOnce(&mut ElementContext<'_, H>, bool) -> TTrigger, content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent, ) -> AnyElement where TTrigger: IntoUiElement<H>, TContent: IntoUiElement<H>;",
                "fn into_element_with_open_model<H: UiHost, TTrigger, TContent>( self, cx: &mut ElementContext<'_, H>, trigger: impl FnOnce(&mut ElementContext<'_, H>, Model<bool>, bool) -> TTrigger, content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent, ) -> AnyElement where TTrigger: IntoUiElement<H>, TContent: IntoUiElement<H>;",
            ][..],
        ),
        (
            "ui_builder_ext/command_dialog.rs",
            UI_BUILDER_EXT_COMMAND_DIALOG_RS,
            &[
                "fn into_element<H: UiHost, TTrigger>( self, cx: &mut ElementContext<'_, H>, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, ) -> AnyElement where TTrigger: IntoUiElement<H>;",
            ][..],
        ),
        (
            "ui_builder_ext/data.rs",
            UI_BUILDER_EXT_DATA_RS,
            &[
                "fn into_element<H: UiHost>( self, cx: &mut ElementContext<'_, H>, cell_text_at: impl Fn(u64, u64) -> Arc<str> + Send + Sync + 'static, ) -> AnyElement;",
                "fn into_element<H: UiHost, FRowKey, FRowState, FCell, TCell>( self, cx: &mut ElementContext<'_, H>, rows_revision: u64, cols_revision: u64, row_key_at: FRowKey, row_state_at: FRowState, cell_at: FCell, ) -> AnyElement where FRowKey: FnMut(usize) -> u64, FRowState: FnMut(usize) -> DataGridRowState, FCell: FnMut(&mut ElementContext<'_, H>, usize, usize) -> TCell, TCell: IntoUiElement<H>;",
                "fn into_element<H: UiHost, TData, TCell>( self, cx: &mut ElementContext<'_, H>, data: Arc<[TData]>, data_revision: u64, state: Model<TableState>, columns: impl Into<Arc<[ColumnDef<TData>]>>, get_row_key: impl Fn(&TData, usize, Option<&RowKey>) -> RowKey + 'static, header_label: impl Fn(&ColumnDef<TData>) -> Arc<str> + 'static, cell_at: impl Fn(&mut ElementContext<'_, H>, &ColumnDef<TData>, &TData) -> TCell + 'static, ) -> AnyElement where TData: 'static, TCell: IntoUiElement<H>;",
            ][..],
        ),
        (
            "ui_builder_ext/menus.rs",
            UI_BUILDER_EXT_MENUS_RS,
            &[
                "fn into_element<H: UiHost, I, TTrigger>( self, cx: &mut ElementContext<'_, H>, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, entries: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = DropdownMenuEntry>, TTrigger: IntoUiElement<H>;",
                "fn into_element<H: UiHost, I, TTrigger>( self, cx: &mut ElementContext<'_, H>, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, entries: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = ContextMenuEntry>, TTrigger: IntoUiElement<H>;",
            ][..],
        ),
        (
            "ui_builder_ext/overlay_roots.rs",
            UI_BUILDER_EXT_OVERLAY_ROOTS_RS,
            &[
                "fn into_element<H: UiHost, TTrigger, TContent>( self, cx: &mut ElementContext<'_, H>, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, content: impl FnOnce(&mut ElementContext<'_, H>) -> TContent, ) -> AnyElement where TTrigger: IntoUiElement<H>, TContent: IntoUiElement<H>;",
            ][..],
        ),
    ] {
        let normalized = normalize_ws(source);
        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "{label} should keep `into_element(...) -> AnyElement` as the explicit landing seam"
            );
        }
    }
}

#[test]
fn ui_builder_ext_do_not_regress_to_anyelement_typed_closure_inputs() {
    for (label, source, forbidden_markers) in [
        (
            "ui_builder_ext/breadcrumb.rs",
            UI_BUILDER_EXT_BREADCRUMB_RS,
            &[
                "children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>",
                "TChild: AnyElement",
            ][..],
        ),
        (
            "ui_builder_ext/collapsible.rs",
            UI_BUILDER_EXT_COLLAPSIBLE_RS,
            &[
                "trigger: impl FnOnce(&mut ElementContext<'_, H>, bool) -> AnyElement",
                "trigger: impl FnOnce(&mut ElementContext<'_, H>, Model<bool>, bool) -> AnyElement",
                "content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement",
            ][..],
        ),
        (
            "ui_builder_ext/command_dialog.rs",
            UI_BUILDER_EXT_COMMAND_DIALOG_RS,
            &["trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement"][..],
        ),
        (
            "ui_builder_ext/data.rs",
            UI_BUILDER_EXT_DATA_RS,
            &[
                "FCell: FnMut(&mut ElementContext<'_, H>, usize, usize) -> AnyElement",
                "cell_at: impl Fn(&mut ElementContext<'_, H>, &ColumnDef<TData>, &TData) -> AnyElement + 'static",
            ][..],
        ),
        (
            "ui_builder_ext/menus.rs",
            UI_BUILDER_EXT_MENUS_RS,
            &["trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement"][..],
        ),
        (
            "ui_builder_ext/overlay_roots.rs",
            UI_BUILDER_EXT_OVERLAY_ROOTS_RS,
            &[
                "trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement",
                "content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement",
            ][..],
        ),
    ] {
        for marker in forbidden_markers {
            assert!(
                !source.contains(marker),
                "{label} reintroduced `AnyElement`-typed closure inputs instead of `IntoUiElement<H>`"
            );
        }
    }
}

#[test]
fn overlay_and_single_child_builders_drop_legacy_child_conversion_trait() {
    for (label, source) in [
        ("alert_dialog.rs", ALERT_DIALOG_RS),
        ("dialog.rs", DIALOG_RS),
        ("drawer.rs", DRAWER_RS),
        ("dropdown_menu.rs", DROPDOWN_MENU_RS),
        ("hover_card.rs", HOVER_CARD_RS),
        ("popover.rs", POPOVER_RS),
        ("sheet.rs", SHEET_RS),
        ("table.rs", TABLE_RS),
        ("tooltip.rs", TOOLTIP_RS),
    ] {
        assert!(
            !source.contains("UiChildIntoElement"),
            "{label} reintroduced legacy child conversion vocabulary"
        );
        assert!(
            source.contains("IntoUiElement<H>"),
            "{label} should use the unified component conversion trait"
        );
    }
}

#[test]
fn internal_menu_slot_wrappers_accept_unified_component_conversion_trait() {
    for (label, source, required_markers, forbidden_markers) in [
        (
            "context_menu.rs",
            CONTEXT_MENU_RS,
            &[
                "fn menu_icon_slot<H: UiHost, B>(cx: &mut ElementContext<'_, H>, element: B) -> AnyElement where B: IntoUiElement<H>,",
            ][..],
            &[
                "fn menu_icon_slot<H: UiHost>(cx: &mut ElementContext<'_, H>, element: AnyElement) -> AnyElement",
            ][..],
        ),
        (
            "dropdown_menu.rs",
            DROPDOWN_MENU_RS,
            &[
                "fn menu_icon_slot<H: UiHost, B>(cx: &mut ElementContext<'_, H>, element: B) -> AnyElement where B: IntoUiElement<H>,",
            ][..],
            &[
                "fn menu_icon_slot<H: UiHost>(cx: &mut ElementContext<'_, H>, element: AnyElement) -> AnyElement",
            ][..],
        ),
        (
            "menubar.rs",
            MENUBAR_RS,
            &[
                "fn menu_icon_slot<H: UiHost, B>(cx: &mut ElementContext<'_, H>, element: B) -> AnyElement where B: IntoUiElement<H>,",
            ][..],
            &[
                "fn menu_icon_slot<H: UiHost>(cx: &mut ElementContext<'_, H>, element: AnyElement) -> AnyElement",
            ][..],
        ),
    ] {
        let normalized = normalize_ws(source);
        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "{label} should accept IntoUiElement<H> on internal wrapper inputs"
            );
        }
        for marker in forbidden_markers {
            let marker = normalize_ws(marker);
            assert!(
                !normalized.contains(&marker),
                "{label} reintroduced pre-landed AnyElement wrapper inputs"
            );
        }
    }
}

#[test]
fn public_thin_constructors_or_wrappers_prefer_typed_conversion_outputs_when_no_raw_seam_is_required()
 {
    for (label, source, required_markers, forbidden_markers) in [
        (
            "accordion.rs",
            ACCORDION_RS,
            &[
                "pub fn accordion_single<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Option<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> Accordion where I: IntoIterator<Item = AccordionItem>,",
                "pub fn accordion_single_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>( cx: &mut ElementContext<'_, H>, default_value: Option<T>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> Accordion where I: IntoIterator<Item = AccordionItem>,",
                "pub fn accordion_multiple<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Vec<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> Accordion where I: IntoIterator<Item = AccordionItem>,",
                "pub fn accordion_multiple_uncontrolled<H: UiHost, V, I>( cx: &mut ElementContext<'_, H>, default_value: V, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> Accordion where V: IntoIterator, V::Item: Into<Arc<str>>, I: IntoIterator<Item = AccordionItem>,",
            ][..],
            &[
                "pub fn accordion_single<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Option<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = AccordionItem>,",
                "pub fn accordion_single_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>( cx: &mut ElementContext<'_, H>, default_value: Option<T>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = AccordionItem>,",
                "pub fn accordion_multiple<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Vec<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = AccordionItem>,",
                "pub fn accordion_multiple_uncontrolled<H: UiHost, V, I>( cx: &mut ElementContext<'_, H>, default_value: V, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where V: IntoIterator, V::Item: Into<Arc<str>>, I: IntoIterator<Item = AccordionItem>,",
            ][..],
        ),
        (
            "badge.rs",
            BADGE_RS,
            &[
                "pub fn badge<H: UiHost, T>(label: T, variant: BadgeVariant) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
            ][..],
            &[
                "pub fn badge<H: UiHost>( cx: &mut ElementContext<'_, H>, label: impl Into<Arc<str>>, variant: BadgeVariant, ) -> AnyElement",
            ][..],
        ),
        (
            "checkbox.rs",
            CHECKBOX_RS,
            &[
                "pub fn checkbox<H: UiHost>(model: Model<bool>) -> impl IntoUiElement<H> + use<H>",
                "pub fn checkbox_opt<H: UiHost>(model: Model<Option<bool>>) -> impl IntoUiElement<H> + use<H>",
            ][..],
            &[
                "pub fn checkbox<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<bool>) -> AnyElement",
                "pub fn checkbox_opt<H: UiHost>( cx: &mut ElementContext<'_, H>, model: Model<Option<bool>>, ) -> AnyElement",
            ][..],
        ),
        (
            "command.rs",
            COMMAND_RS,
            &[
                "pub fn command<H: UiHost, I, F, T>(f: F) -> impl IntoUiElement<H> + use<H, I, F, T> where F: FnOnce(&mut ElementContext<'_, H>) -> I, I: IntoIterator<Item = T>, T: IntoUiElement<H>,",
            ][..],
            &[
                "pub fn command<H: UiHost, I, F>(cx: &mut ElementContext<'_, H>, f: F) -> AnyElement where F: FnOnce(&mut ElementContext<'_, H>) -> I, I: IntoIterator<Item = AnyElement>,",
            ][..],
        ),
        (
            "input_group.rs",
            INPUT_GROUP_RS,
            &[
                "pub fn input_group<H: UiHost>(group: InputGroup) -> impl IntoUiElement<H> + use<H> {",
            ][..],
            &[
                "pub fn input_group<H: UiHost>(cx: &mut ElementContext<'_, H>, group: InputGroup) -> AnyElement",
            ][..],
        ),
        (
            "input.rs",
            INPUT_RS,
            &["pub fn input(model: impl IntoTextValueModel) -> Input {"][..],
            &[
                "pub fn input<H: UiHost>( cx: &mut ElementContext<'_, H>, model: impl IntoTextValueModel, a11y_label: Option<Arc<str>>, a11y_role: Option<SemanticsRole>, placeholder: Option<Arc<str>>, active_descendant: Option<NodeId>, expanded: Option<bool>, submit_command: Option<CommandId>, cancel_command: Option<CommandId>, ) -> AnyElement",
            ][..],
        ),
        (
            "input_otp.rs",
            INPUT_OTP_RS,
            &["pub fn input_otp<H: UiHost>(otp: InputOtp) -> impl IntoUiElement<H> + use<H> {"][..],
            &[
                "pub fn input_otp<H: UiHost>(cx: &mut ElementContext<'_, H>, otp: InputOtp) -> AnyElement",
            ][..],
        ),
        (
            "navigation_menu.rs",
            NAVIGATION_MENU_RS,
            &[
                "pub fn navigation_menu<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Option<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> NavigationMenu where I: IntoIterator<Item = NavigationMenuItem>,",
                "pub fn navigation_menu_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>( cx: &mut ElementContext<'_, H>, default_value: Option<T>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> NavigationMenu where I: IntoIterator<Item = NavigationMenuItem>,",
            ][..],
            &[
                "pub fn navigation_menu<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Option<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = NavigationMenuItem>,",
                "pub fn navigation_menu_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>( cx: &mut ElementContext<'_, H>, default_value: Option<T>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = NavigationMenuItem>,",
            ][..],
        ),
        (
            "kbd.rs",
            KBD_RS,
            &[
                "pub fn kbd<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
            ][..],
            &[
                "pub fn kbd<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
            ][..],
        ),
        (
            "progress.rs",
            PROGRESS_RS,
            &["pub fn progress<H: UiHost>(model: Model<f32>) -> impl IntoUiElement<H> + use<H>"][..],
            &[
                "pub fn progress<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<f32>) -> AnyElement",
            ][..],
        ),
        (
            "radio_group.rs",
            RADIO_GROUP_RS,
            &[
                "pub fn radio_group(model: Model<Option<Arc<str>>>, items: Vec<RadioGroupItem>) -> RadioGroup {",
                "pub fn radio_group_uncontrolled<T: Into<Arc<str>>>( default_value: Option<T>, items: Vec<RadioGroupItem>, ) -> RadioGroup {",
            ][..],
            &[
                "pub fn radio_group<H: UiHost>( cx: &mut ElementContext<'_, H>, model: Model<Option<Arc<str>>>, items: Vec<RadioGroupItem>, ) -> AnyElement",
                "pub fn radio_group_uncontrolled<H: UiHost, T: Into<Arc<str>>>( cx: &mut ElementContext<'_, H>, default_value: Option<T>, items: Vec<RadioGroupItem>, ) -> AnyElement",
            ][..],
        ),
        (
            "resizable.rs",
            RESIZABLE_RS,
            &[
                "pub fn resizable_panel_group<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Vec<f32>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> ResizablePanelGroup where I: IntoIterator<Item = ResizableEntry>,",
            ][..],
            &[
                "pub fn resizable_panel_group<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Vec<f32>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = ResizableEntry>,",
            ][..],
        ),
        (
            "separator.rs",
            SEPARATOR_RS,
            &["pub fn separator<H: UiHost>() -> impl IntoUiElement<H> + use<H> {"][..],
            &["pub fn separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"][..],
        ),
        (
            "slider.rs",
            SLIDER_RS,
            &["pub fn slider(model: Model<Vec<f32>>) -> Slider {"][..],
            &[
                "pub fn slider<H: UiHost>( cx: &mut ElementContext<'_, H>, model: Model<Vec<f32>>, orientation: radix_slider::SliderOrientation, dir: Option<radix_direction::LayoutDirection>, inverted: bool, min: f32, max: f32, step: f32, min_steps_between_thumbs: u32, disabled: bool, control_id: Option<ControlId>, a11y_label: Option<Arc<str>>, test_id: Option<Arc<str>>, on_value_commit: Option<OnValueCommit>, chrome: ChromeRefinement, layout: LayoutRefinement, style: SliderStyle, ) -> AnyElement",
            ][..],
        ),
        (
            "switch.rs",
            SWITCH_RS,
            &[
                "pub fn switch<H: UiHost>(model: Model<bool>) -> impl IntoUiElement<H> + use<H>",
                "pub fn switch_opt<H: UiHost>(model: Model<Option<bool>>) -> impl IntoUiElement<H> + use<H>",
            ][..],
            &[
                "pub fn switch<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<bool>) -> AnyElement",
                "pub fn switch_opt<H: UiHost>( cx: &mut ElementContext<'_, H>, model: Model<Option<bool>>, ) -> AnyElement",
            ][..],
        ),
        (
            "toggle.rs",
            TOGGLE_RS,
            &[
                "pub fn toggle<H: UiHost, I, T>( cx: &mut ElementContext<'_, H>, model: Model<bool>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> Toggle where I: IntoIterator<Item = T>, T: IntoUiElement<H>,",
                "pub fn toggle_uncontrolled<H: UiHost, I, T>( cx: &mut ElementContext<'_, H>, default_pressed: bool, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> Toggle where I: IntoIterator<Item = T>, T: IntoUiElement<H>,",
            ][..],
            &[
                "pub fn toggle<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<bool>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
                "pub fn toggle_uncontrolled<H: UiHost, I>( cx: &mut ElementContext<'_, H>, default_pressed: bool, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
            ][..],
        ),
        (
            "toggle_group.rs",
            TOGGLE_GROUP_RS,
            &[
                "pub fn toggle_group_single<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Option<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> ToggleGroup where I: IntoIterator<Item = ToggleGroupItem>,",
                "pub fn toggle_group_single_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>( cx: &mut ElementContext<'_, H>, default_value: Option<T>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> ToggleGroup where I: IntoIterator<Item = ToggleGroupItem>,",
                "pub fn toggle_group_multiple<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Vec<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> ToggleGroup where I: IntoIterator<Item = ToggleGroupItem>,",
                "pub fn toggle_group_multiple_uncontrolled<H: UiHost, V, I>( cx: &mut ElementContext<'_, H>, default_value: V, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> ToggleGroup where V: IntoIterator, V::Item: Into<Arc<str>>, I: IntoIterator<Item = ToggleGroupItem>,",
            ][..],
            &[
                "pub fn toggle_group_single<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Option<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = ToggleGroupItem>,",
                "pub fn toggle_group_single_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>( cx: &mut ElementContext<'_, H>, default_value: Option<T>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = ToggleGroupItem>,",
                "pub fn toggle_group_multiple<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Vec<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = ToggleGroupItem>,",
                "pub fn toggle_group_multiple_uncontrolled<H: UiHost, V, I>( cx: &mut ElementContext<'_, H>, default_value: V, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where V: IntoIterator, V::Item: Into<Arc<str>>, I: IntoIterator<Item = ToggleGroupItem>,",
            ][..],
        ),
        (
            "textarea.rs",
            TEXTAREA_RS,
            &["pub fn textarea(model: impl IntoTextValueModel) -> Textarea {"][..],
            &[
                "pub fn textarea<H: UiHost>( cx: &mut ElementContext<'_, H>, model: impl IntoTextValueModel, a11y_label: Option<Arc<str>>, labelled_by_element: Option<GlobalElementId>, control_id: Option<ControlId>, test_id: Option<Arc<str>>, placeholder: Option<Arc<str>>, aria_invalid: bool, aria_required: bool, disabled: bool, min_height: Px, resizable: bool, stable_line_boxes: bool, size: ComponentSize, chrome: ChromeRefinement, layout: LayoutRefinement, ) -> AnyElement",
            ][..],
        ),
        (
            "tabs.rs",
            TABS_RS,
            &[
                "pub fn tabs<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Option<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> Tabs where I: IntoIterator<Item = TabsItem>,",
                "pub fn tabs_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>( cx: &mut ElementContext<'_, H>, default_value: Option<T>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> Tabs where I: IntoIterator<Item = TabsItem>,",
            ][..],
            &[
                "pub fn tabs<H: UiHost, I>( cx: &mut ElementContext<'_, H>, model: Model<Option<Arc<str>>>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = TabsItem>,",
                "pub fn tabs_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>( cx: &mut ElementContext<'_, H>, default_value: Option<T>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = TabsItem>,",
            ][..],
        ),
    ] {
        let normalized = normalize_ws(source);
        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "{label} should expose a typed constructor or wrapper when no raw seam is conceptually required"
            );
        }
        for marker in forbidden_markers {
            let marker = normalize_ws(marker);
            assert!(
                !normalized.contains(&marker),
                "{label} reintroduced a pre-landed AnyElement constructor or wrapper"
            );
        }
    }
}

#[test]
fn collapsible_helpers_prefer_typed_build_outputs_when_no_raw_slot_storage_is_required() {
    let normalized = normalize_ws(COLLAPSIBLE_RS);
    let required_markers = [
        "pub fn collapsible<H: UiHost, Trigger, Content, TriggerEl, ContentEl>( open: Model<bool>, trigger: Trigger, content: Content, ) -> CollapsibleBuild<H, Trigger, Content>",
        "pub fn collapsible_uncontrolled<H: UiHost, Trigger, Content, TriggerEl, ContentEl>( default_open: bool, trigger: Trigger, content: Content, ) -> CollapsibleUncontrolledBuild<H, Trigger, Content>",
    ];
    let forbidden_markers = [
        "pub fn collapsible<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Model<bool>, trigger: impl FnOnce(&mut ElementContext<'_, H>, bool) -> AnyElement, content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement, ) -> AnyElement",
        "pub fn collapsible_uncontrolled<H: UiHost>( cx: &mut ElementContext<'_, H>, default_open: bool, trigger: impl FnOnce(&mut ElementContext<'_, H>, Model<bool>, bool) -> AnyElement, content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement, ) -> AnyElement",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "collapsible.rs should expose typed helper outputs for the default authoring surface"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "collapsible.rs reintroduced pre-landed AnyElement helper signatures on the public surface"
        );
    }
}

#[test]
fn breadcrumb_primitives_prefer_typed_child_conversion_before_the_landing_seam() {
    let normalized = normalize_ws(BREADCRUMB_RS);
    let required_markers = [
        "pub fn into_element<H: UiHost, I, TChild>( self, cx: &mut ElementContext<'_, H>, children: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = TChild>, TChild: IntoUiElement<H>,",
    ];
    let forbidden_markers = [
        "pub fn into_element<H: UiHost, I>( self, cx: &mut ElementContext<'_, H>, children: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "breadcrumb.rs should accept typed child conversion on primitive breadcrumb containers"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "breadcrumb.rs reintroduced primitive breadcrumb container closures that land children as AnyElement too early"
        );
    }
}

#[test]
fn breadcrumb_link_and_page_keep_raw_children_as_an_explicit_escape_hatch() {
    let normalized = normalize_ws(BREADCRUMB_RS);
    let required_markers = [
        "pub struct BreadcrumbLinkBuild<H, Children> {",
        "pub fn children<H: UiHost, I, TChild, Children>( self, children: Children, ) -> BreadcrumbLinkBuild<H, Children> where Children: FnOnce(&mut ElementContext<'_, H>) -> I, I: IntoIterator<Item = TChild>, TChild: IntoUiElement<H>,",
        "pub fn children_raw(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {",
        "pub struct BreadcrumbPageBuild<H, Children> {",
        "pub fn children<H: UiHost, I, TChild, Children>( self, children: Children, ) -> BreadcrumbPageBuild<H, Children> where Children: FnOnce(&mut ElementContext<'_, H>) -> I, I: IntoIterator<Item = TChild>, TChild: IntoUiElement<H>,",
    ];
    let forbidden_markers =
        ["pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {"];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "breadcrumb.rs should keep `children(...)` typed and reserve `children_raw(...)` for the explicit landed-child seam"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "breadcrumb.rs reintroduced raw `children(...)` as the default primitive surface"
        );
    }
}

#[test]
fn popover_root_promotes_typed_new_and_keeps_advanced_and_raw_root_seams_explicit() {
    let normalized = normalize_ws(POPOVER_RS);
    let required_markers = [
        "pub fn from_open(open: Model<bool>) -> Self {",
        "pub fn new<H: UiHost>( cx: &mut ElementContext<'_, H>, trigger: impl IntoUiElement<H>, content: impl IntoUiElement<H>, ) -> Self {",
        "pub fn new_raw<H: UiHost>( cx: &mut ElementContext<'_, H>, trigger: AnyElement, content: AnyElement, ) -> Self {",
        "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, trigger: impl IntoUiElement<H>, content: impl IntoUiElement<H>, ) -> Self {",
        "pub fn from_open_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, ) -> Self {",
        "pub fn new_controllable_raw<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, trigger: AnyElement, content: AnyElement, ) -> Self {",
    ];
    let forbidden_markers = [
        "pub fn build<H: UiHost>(",
        "pub fn new(open: Model<bool>) -> Self {",
        "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, ) -> Self {",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "popover.rs should promote `new(...)` / `new_controllable(...)` as the typed root constructors while keeping `from_open(...)` and raw root seams explicit"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "popover.rs reintroduced legacy root constructors after promoting the typed `new(...)` surface"
        );
    }
}

#[test]
fn hover_card_root_promotes_typed_new_and_keeps_raw_root_seams_explicit() {
    let normalized = normalize_ws(HOVER_CARD_RS);
    let required_markers = [
        "pub fn new<H: UiHost>( cx: &mut ElementContext<'_, H>, trigger: impl IntoUiElement<H>, content: impl Into<HoverCardContentArg>, ) -> Self {",
        "pub fn new_raw(trigger: AnyElement, content: impl Into<HoverCardContentArg>) -> Self {",
        "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, trigger: impl IntoUiElement<H>, content: impl Into<HoverCardContentArg>, ) -> Self {",
        "pub fn new_controllable_raw<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, trigger: AnyElement, content: impl Into<HoverCardContentArg>, ) -> Self {",
    ];
    let forbidden_markers = [
        "pub fn build<H: UiHost>(",
        "pub fn build_controllable<H: UiHost>(",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "hover_card.rs should promote `new(...)` / `new_controllable(...)` as the typed root constructors and keep raw root seams explicit"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "hover_card.rs reintroduced legacy `build(...)` root constructors after promoting the typed `new(...)` surface"
        );
    }
}

#[test]
fn tooltip_root_promotes_typed_new_and_keeps_raw_root_seams_explicit() {
    let normalized = normalize_ws(TOOLTIP_RS);
    let required_markers = [
        "pub fn new<H: UiHost>( cx: &mut ElementContext<'_, H>, trigger: impl IntoUiElement<H>, content: impl Into<TooltipContentArg>, ) -> Self {",
        "pub fn new_raw(trigger: AnyElement, content: impl Into<TooltipContentArg>) -> Self {",
    ];
    let forbidden_markers = ["pub fn build<H: UiHost>("];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "tooltip.rs should promote `new(...)` as the typed root constructor and keep the raw root seam explicit"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "tooltip.rs reintroduced legacy `build(...)` after promoting the typed `new(...)` surface"
        );
    }
}

#[test]
fn card_helpers_prefer_typed_wrapper_outputs_when_no_raw_slot_storage_is_required() {
    let normalized = normalize_ws(CARD_RS);
    let required_markers = [
        "pub fn card<H: UiHost, I, F, T>( f: F, ) -> CardBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn card_sized<H: UiHost, I, F, T>( size: CardSize, f: F, ) -> CardBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn card_header<H: UiHost, I, F, T>( f: F, ) -> CardHeaderBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn card_action<H: UiHost, I, F, T>( f: F, ) -> CardActionBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn card_title<T>(text: T) -> CardTitle where T: Into<Arc<str>>,",
        "pub fn card_description<T>(text: T) -> CardDescription where T: Into<Arc<str>>,",
        "pub fn card_description_children<H: UiHost, I, F, T>( f: F, ) -> CardDescriptionBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn card_content<H: UiHost, I, F, T>( f: F, ) -> CardContentBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn card_footer<H: UiHost, I, F, T>( f: F, ) -> CardFooterBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
    ];
    let forbidden_markers = [
        "pub fn card<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn card_sized<H: UiHost, I>(cx: &mut ElementContext<'_, H>, size: CardSize, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn card_header<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn card_action<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn card_title<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn card_description<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn card_description_children<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn card_content<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn card_footer<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "card.rs should expose typed wrapper outputs for the default authoring surface"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "card.rs reintroduced pre-landed AnyElement wrapper signatures on the public surface"
        );
    }
}

#[test]
fn alert_helpers_prefer_typed_wrapper_outputs_when_no_raw_slot_storage_is_required() {
    let normalized = normalize_ws(ALERT_RS);
    let required_markers = [
        "pub fn alert<H: UiHost, I, F, T>( f: F, ) -> AlertBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
    ];
    let forbidden_markers = [
        "pub fn alert<H: UiHost>(cx: &mut ElementContext<'_, H>, variant: AlertVariant, children: impl IntoIterator<Item = AnyElement>) -> AnyElement",
        "pub fn alert<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "alert.rs should expose typed wrapper outputs for the default authoring surface"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "alert.rs reintroduced pre-landed AnyElement wrapper signatures on the public surface"
        );
    }
}

#[test]
fn table_helpers_prefer_typed_wrapper_outputs_when_no_raw_slot_storage_is_required() {
    let normalized = normalize_ws(TABLE_RS);
    let required_markers = [
        "pub fn table<H: UiHost, I, F, T>( f: F, ) -> TableBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn table_header<H: UiHost, I, F, T>( f: F, ) -> TableHeaderBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn table_body<H: UiHost, I, F, T>( f: F, ) -> TableBodyBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn table_footer<H: UiHost, I, F, T>( f: F, ) -> TableFooterBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn table_row<H: UiHost, I, F, T>( cols: u16, f: F, ) -> TableRowBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn table_head<T>(text: T) -> TableHead where T: Into<Arc<str>>,",
        "pub fn table_cell<H: UiHost, T>(child: T) -> TableCellBuild<H, T> where T: IntoUiElement<H>,",
        "pub fn table_caption<T>(text: T) -> TableCaption where T: Into<Arc<str>>,",
    ];
    let forbidden_markers = [
        "pub fn table<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn table_header<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn table_body<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn table_footer<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn table_row<H: UiHost, I>(cx: &mut ElementContext<'_, H>, cols: u16, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn table_head<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn table_cell<H: UiHost>(cx: &mut ElementContext<'_, H>, child: impl IntoUiElement<H>) -> AnyElement",
        "pub fn table_caption<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "table.rs should expose typed wrapper outputs for the default authoring surface"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "table.rs reintroduced pre-landed AnyElement wrapper signatures on the public surface"
        );
    }
}

#[test]
fn field_helpers_prefer_typed_wrapper_outputs_when_no_raw_slot_storage_is_required() {
    let normalized = normalize_ws(FIELD_RS);
    let required_markers = [
        "pub fn field_set<H: UiHost, I, F, T>( f: F, ) -> FieldSetBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn field_group<H: UiHost, I, F, T>( f: F, ) -> FieldGroupBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
    ];
    let forbidden_markers = [
        "pub fn field_set<H: UiHost, I>( cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn field_group<H: UiHost, I>( cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "field.rs should expose typed wrapper outputs for the default authoring surface"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "field.rs reintroduced pre-landed AnyElement wrapper signatures on the public surface"
        );
    }
}

#[test]
fn empty_helpers_prefer_typed_wrapper_outputs_when_no_raw_slot_storage_is_required() {
    let normalized = normalize_ws(EMPTY_RS);
    let required_markers = [
        "pub fn empty<H: UiHost, I, F, T>( f: F, ) -> EmptyBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn empty_header<H: UiHost, I, F, T>( f: F, ) -> EmptyHeaderBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn empty_media<H: UiHost, I, F, T>( f: F, ) -> EmptyMediaBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn empty_title<T>(text: T) -> EmptyTitle where T: Into<Arc<str>>,",
        "pub fn empty_description<T>(text: T) -> EmptyDescription where T: Into<Arc<str>>,",
        "pub fn empty_content<H: UiHost, I, F, T>( f: F, ) -> EmptyContentBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
    ];
    let forbidden_markers = [
        "pub fn empty<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn empty_header<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn empty_media<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn empty_title<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn empty_description<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn empty_content<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "empty.rs should expose typed wrapper outputs for the default authoring surface"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "empty.rs reintroduced pre-landed AnyElement wrapper signatures on the public surface"
        );
    }
}

#[test]
fn pagination_helpers_prefer_typed_wrapper_outputs_when_no_raw_slot_storage_is_required() {
    let normalized = normalize_ws(PAGINATION_RS);
    let required_markers = [
        "pub fn pagination<H: UiHost, I, F, T>( f: F, ) -> PaginationBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn pagination_content<H: UiHost, I, F, T>( f: F, ) -> PaginationContentBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
        "pub fn pagination_item<H: UiHost, T>(child: T) -> PaginationItemBuild<H, T> where T: IntoUiElement<H>,",
        "pub fn pagination_link<H: UiHost, I, F, T>( f: F, ) -> PaginationLinkBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>",
    ];
    let forbidden_markers = [
        "pub fn pagination<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn pagination_content<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
        "pub fn pagination_item<H: UiHost>(cx: &mut ElementContext<'_, H>, child: impl IntoUiElement<H>) -> AnyElement",
        "pub fn pagination_link<H: UiHost, I>(cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I,) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "pagination.rs should expose typed wrapper outputs for the default authoring surface"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "pagination.rs reintroduced pre-landed AnyElement wrapper signatures on the public surface"
        );
    }
}

#[test]
fn kbd_icon_stays_an_explicit_raw_helper_for_kbd_child_lists() {
    assert!(
        KBD_RS.contains(
            "This intentionally stays raw because `Kbd::from_children(...)` stores an explicit"
        ),
        "kbd.rs should keep the raw `kbd_icon(...)` seam explicitly documented"
    );

    let normalized = normalize_ws(KBD_RS);
    let required_markers =
        ["pub fn kbd_icon<H: UiHost>(cx: &mut ElementContext<'_, H>, icon: IconId) -> AnyElement"];
    let forbidden_markers = [
        "pub fn kbd_icon<H: UiHost>(icon: IconId) -> impl IntoUiElement<H>",
        "pub fn kbd_icon<H: UiHost>(cx: &mut ElementContext<'_, H>, icon: IconId) -> impl IntoUiElement<H>",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "kbd.rs should keep the raw `kbd_icon(...)` signature explicit"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "kbd.rs should not pretend the `kbd_icon(...)` helper is typed while `Kbd::from_children(...)` still owns a raw child list"
        );
    }
}

#[test]
fn combobox_anchor_stays_an_explicit_raw_wrapper_until_anchor_storage_is_typed() {
    assert!(
        COMBOBOX_RS.contains(
            "This intentionally stays raw because `PopoverAnchor::new(...)` stores a concrete landed child."
        ),
        "combobox.rs should keep the raw `use_combobox_anchor(...)` seam explicitly documented"
    );

    let normalized = normalize_ws(COMBOBOX_RS);
    let required_markers = ["pub fn use_combobox_anchor(child: AnyElement) -> PopoverAnchor"];
    let forbidden_markers = [
        "pub fn use_combobox_anchor<H: UiHost>(cx: &mut ElementContext<'_, H>, child: impl IntoUiElement<H>) -> PopoverAnchor",
        "pub fn use_combobox_anchor<H: UiHost, T>(cx: &mut ElementContext<'_, H>, child: T) -> PopoverAnchor where T: IntoUiElement<H>",
        "pub fn use_combobox_anchor(child: impl IntoUiElement<_>) -> PopoverAnchor",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "combobox.rs should keep the raw `use_combobox_anchor(...)` signature explicit"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "combobox.rs should not pretend the anchor wrapper is typed while `PopoverAnchor::new(...)` still owns a raw landed child"
        );
    }
}

#[test]
fn thin_constructor_trial_modules_keep_public_anyelement_free_functions_explicit_and_rare() {
    let mut hits = Vec::new();
    for (label, source) in [
        ("src/badge.rs", BADGE_RS),
        ("src/checkbox.rs", CHECKBOX_RS),
        ("src/command.rs", COMMAND_RS),
        ("src/empty.rs", EMPTY_RS),
        ("src/input_group.rs", INPUT_GROUP_RS),
        ("src/input_otp.rs", INPUT_OTP_RS),
        ("src/kbd.rs", KBD_RS),
        ("src/pagination.rs", PAGINATION_RS),
        ("src/progress.rs", PROGRESS_RS),
        ("src/separator.rs", SEPARATOR_RS),
        ("src/switch.rs", SWITCH_RS),
    ] {
        for signature in public_anyelement_signatures(source) {
            if signature.contains("(self")
                || signature.contains("( self")
                || signature.contains("(mut self")
                || signature.contains("( mut self")
            {
                continue;
            }
            hits.push(format!("{label}: {signature}"));
        }
    }

    assert_eq!(
        hits,
        vec![String::from(
            "src/kbd.rs: pub fn kbd_icon<H: UiHost>(cx: &mut ElementContext<'_, H>, icon: IconId) -> AnyElement"
        )],
        "thin-constructor trial modules should not grow new public free-function `-> AnyElement` helpers without an explicit raw-seam decision"
    );
}

#[test]
fn typography_helpers_keep_raw_namespace_but_expose_typed_conversion_outputs() {
    let normalized = normalize_ws(TYPOGRAPHY_RS);

    let required_markers = [
        "pub fn h1<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
        "pub fn h2<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
        "pub fn h3<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
        "pub fn h4<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
        "pub fn p<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
        "pub fn lead<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
        "pub fn large<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
        "pub fn small<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
        "pub fn muted<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
        "pub fn inline_code<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
        "pub fn blockquote<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
        "pub fn list<H: UiHost, I, T>(items: I) -> impl IntoUiElement<H> + use<H, I, T> where I: IntoIterator<Item = T>, T: Into<Arc<str>>,",
    ];
    let forbidden_markers = [
        "pub fn h1<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn h2<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn h3<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn h4<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn p<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn lead<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn large<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn small<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn muted<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
        "pub fn inline_code<H: UiHost>( cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>, ) -> AnyElement",
        "pub fn blockquote<H: UiHost>( cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>, ) -> AnyElement",
        "pub fn list<H: UiHost>( cx: &mut ElementContext<'_, H>, items: impl IntoIterator<Item = Arc<str>>, ) -> AnyElement",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "typography.rs should keep the raw namespace while exposing typed helper outputs"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "typography.rs reintroduced pre-landed AnyElement helper signatures"
        );
    }
}
