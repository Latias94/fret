const LIB_RS: &str = include_str!("lib.rs");
const APP_RS: &str = include_str!("app.rs");
const ADVANCED_RS: &str = include_str!("advanced.rs");
const README: &str = include_str!("../README.md");
const UI_EXT_SUPPORT_RS: &str = include_str!("ui_ext/support.rs");
const UI_EXT_DATA_RS: &str = include_str!("ui_ext/data.rs");
const ALERT_DIALOG_RS: &str = include_str!("alert_dialog.rs");
const CONTEXT_MENU_RS: &str = include_str!("context_menu.rs");
const DIALOG_RS: &str = include_str!("dialog.rs");
const DRAWER_RS: &str = include_str!("drawer.rs");
const DROPDOWN_MENU_RS: &str = include_str!("dropdown_menu.rs");
const HOVER_CARD_RS: &str = include_str!("hover_card.rs");
const KBD_RS: &str = include_str!("kbd.rs");
const MENUBAR_RS: &str = include_str!("menubar.rs");
const POPOVER_RS: &str = include_str!("popover.rs");
const SEPARATOR_RS: &str = include_str!("separator.rs");
const SHEET_RS: &str = include_str!("sheet.rs");
const TABLE_RS: &str = include_str!("table.rs");
const TOOLTIP_RS: &str = include_str!("tooltip.rs");
const UI_BUILDER_EXT_BREADCRUMB_RS: &str = include_str!("ui_builder_ext/breadcrumb.rs");
const UI_BUILDER_EXT_COLLAPSIBLE_RS: &str = include_str!("ui_builder_ext/collapsible.rs");
const UI_BUILDER_EXT_COMMAND_DIALOG_RS: &str = include_str!("ui_builder_ext/command_dialog.rs");
const UI_BUILDER_EXT_DATA_RS: &str = include_str!("ui_builder_ext/data.rs");
const UI_BUILDER_EXT_MENUS_RS: &str = include_str!("ui_builder_ext/menus.rs");
const UI_BUILDER_EXT_OVERLAY_ROOTS_RS: &str = include_str!("ui_builder_ext/overlay_roots.rs");

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
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
fn public_leaf_constructors_prefer_typed_conversion_outputs_when_no_raw_seam_is_required() {
    for (label, source, required_markers, forbidden_markers) in [
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
            "separator.rs",
            SEPARATOR_RS,
            &["pub fn separator<H: UiHost>() -> impl IntoUiElement<H> + use<H> {"][..],
            &["pub fn separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"][..],
        ),
    ] {
        let normalized = normalize_ws(source);
        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "{label} should expose a typed leaf constructor when no raw seam is conceptually required"
            );
        }
        for marker in forbidden_markers {
            let marker = normalize_ws(marker);
            assert!(
                !normalized.contains(&marker),
                "{label} reintroduced a pre-landed AnyElement leaf constructor"
            );
        }
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
