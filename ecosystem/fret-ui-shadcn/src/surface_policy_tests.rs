const LIB_RS: &str = include_str!("lib.rs");
const APP_RS: &str = include_str!("app.rs");
const ADVANCED_RS: &str = include_str!("advanced.rs");
const README: &str = include_str!("../README.md");
const ACCORDION_RS: &str = include_str!("accordion.rs");
const ALERT_RS: &str = include_str!("alert.rs");
const AVATAR_RS: &str = include_str!("avatar.rs");
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
const ITEM_RS: &str = include_str!("item.rs");
const KBD_RS: &str = include_str!("kbd.rs");
const MENUBAR_RS: &str = include_str!("menubar.rs");
const NAVIGATION_MENU_RS: &str = include_str!("navigation_menu.rs");
const NATIVE_SELECT_RS: &str = include_str!("native_select.rs");
const PAGINATION_RS: &str = include_str!("pagination.rs");
const POPOVER_RS: &str = include_str!("popover.rs");
const SEPARATOR_RS: &str = include_str!("separator.rs");
const SHEET_RS: &str = include_str!("sheet.rs");
const SLIDER_RS: &str = include_str!("slider.rs");
const SIDEBAR_RS: &str = include_str!("sidebar.rs");
const SONNER_RS: &str = include_str!("sonner.rs");
const STATE_RS: &str = include_str!("state.rs");
const TABLE_RS: &str = include_str!("table.rs");
const TEXTAREA_RS: &str = include_str!("textarea.rs");
const TEXT_EDIT_CONTEXT_MENU_RS: &str = include_str!("text_edit_context_menu.rs");
const TOGGLE_RS: &str = include_str!("toggle.rs");
const TOGGLE_GROUP_RS: &str = include_str!("toggle_group.rs");
const TOOLTIP_RS: &str = include_str!("tooltip.rs");
const TYPOGRAPHY_RS: &str = include_str!("typography.rs");
const CHECKBOX_RS: &str = include_str!("checkbox.rs");
const COMBOBOX_RS: &str = include_str!("combobox.rs");
const PROGRESS_RS: &str = include_str!("progress.rs");
const RADIO_GROUP_RS: &str = include_str!("radio_group.rs");
const RESIZABLE_RS: &str = include_str!("resizable.rs");
const SCROLL_AREA_RS: &str = include_str!("scroll_area.rs");
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

fn normalized_reexport_block(source: &str, marker: &str) -> String {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing re-export marker `{marker}`"));
    let tail = &source[start + marker.len()..];
    let end = tail
        .find("};")
        .unwrap_or_else(|| panic!("unterminated re-export block for `{marker}`"));
    normalize_ws(&tail[..end])
}

fn assert_root_and_facade_reexports(module: &str, names: &[&str]) {
    let root_marker = format!("pub use {module}::{{");
    let facade_marker = format!("pub use crate::{module}::{{");
    let root_block = normalized_reexport_block(LIB_RS, &root_marker);
    let facade_block = normalized_reexport_block(LIB_RS, &facade_marker);

    for name in names {
        assert!(
            root_block.contains(name),
            "{module} should re-export `{name}` from the crate root"
        );
        assert!(
            facade_block.contains(name),
            "{module} should re-export `{name}` from the curated facade"
        );
    }
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

fn public_non_method_anyelement_signatures(source: &str) -> Vec<String> {
    public_anyelement_signatures(source)
        .into_iter()
        .filter(|signature| {
            ![
                "(self",
                "( self",
                "(mut self",
                "( mut self",
                "(&self",
                "( &self",
                "(&mut self",
                "( &mut self",
            ]
            .iter()
            .any(|marker| signature.contains(marker))
        })
        .collect()
}

fn visit_rust_files(dir: &std::path::Path, f: &mut impl FnMut(&std::path::Path, &str)) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|err| {
        panic!(
            "failed to read source-policy directory {}: {err}",
            dir.display()
        )
    }) {
        let entry = entry.unwrap_or_else(|err| {
            panic!(
                "failed to read source-policy entry in {}: {err}",
                dir.display()
            )
        });
        let path = entry.path();
        if path.is_dir() {
            visit_rust_files(&path, f);
            continue;
        }
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!(
                "failed to read source-policy file {}: {err}",
                path.display()
            )
        });
        f(&path, &source);
    }
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
fn authoring_critical_family_exports_stay_on_root_and_curated_facade() {
    assert_root_and_facade_reexports(
        "select",
        &[
            "Select",
            "SelectTrigger",
            "SelectValue",
            "SelectContent",
            "SelectItem",
        ],
    );
    assert_root_and_facade_reexports(
        "combobox",
        &[
            "Combobox",
            "ComboboxTrigger",
            "ComboboxInput",
            "ComboboxClear",
            "ComboboxContent",
            "ComboboxValue",
        ],
    );
    assert_root_and_facade_reexports("combobox_chips", &["ComboboxChips", "ComboboxChipsPart"]);
    assert_root_and_facade_reexports(
        "command",
        &[
            "Command",
            "CommandPalette",
            "CommandInput",
            "CommandList",
            "CommandItem",
            "command",
        ],
    );
    assert_root_and_facade_reexports(
        "navigation_menu",
        &[
            "NavigationMenu",
            "NavigationMenuRoot",
            "NavigationMenuList",
            "NavigationMenuItem",
            "NavigationMenuTrigger",
            "NavigationMenuContent",
            "NavigationMenuLink",
            "NavigationMenuViewport",
            "NavigationMenuIndicator",
            "navigation_menu",
            "navigation_menu_uncontrolled",
        ],
    );
    assert_root_and_facade_reexports(
        "pagination",
        &[
            "Pagination",
            "PaginationContent",
            "PaginationItem",
            "PaginationLink",
            "PaginationPrevious",
            "PaginationNext",
            "pagination",
            "pagination_content",
            "pagination_item",
            "pagination_link",
        ],
    );
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
            "avatar.rs",
            AVATAR_RS,
            &[
                "pub fn avatar_sized<H: UiHost, I>( cx: &mut ElementContext<'_, H>, size: AvatarSize, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> Avatar where I: IntoIterator<Item = AnyElement>,",
            ][..],
            &[
                "pub fn avatar_sized<H: UiHost, I>( cx: &mut ElementContext<'_, H>, size: AvatarSize, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
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
            "item.rs",
            ITEM_RS,
            &[
                "pub fn item_sized<H: UiHost, I>( cx: &mut ElementContext<'_, H>, size: ItemSize, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> Item where I: IntoIterator<Item = AnyElement>,",
                "pub fn item_group<H: UiHost>( cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>, ) -> ItemGroup",
            ][..],
            &[
                "pub fn item_sized<H: UiHost, I>( cx: &mut ElementContext<'_, H>, size: ItemSize, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
                "pub fn item_group<H: UiHost>( cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>, ) -> AnyElement",
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
            "native_select.rs",
            NATIVE_SELECT_RS,
            &[
                "pub fn native_select(model: Model<Option<Arc<str>>>, open: Model<bool>) -> NativeSelect {",
            ][..],
            &[
                "pub fn native_select<H: UiHost>( cx: &mut ElementContext<'_, H>, model: Model<Option<Arc<str>>>, open: Model<bool>, placeholder: Arc<str>, options: &[NativeSelectOption], optgroups: &[NativeSelectOptGroup], control_id: Option<ControlId>, test_id_prefix: Option<Arc<str>>, trigger_test_id: Option<Arc<str>>, a11y_label: Option<Arc<str>>, aria_invalid: bool, disabled: bool, size: NativeSelectSize, chrome: ChromeRefinement, layout: LayoutRefinement, ) -> AnyElement",
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
            "scroll_area.rs",
            SCROLL_AREA_RS,
            &[
                "pub fn scroll_area<H: UiHost, I>( cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> ScrollArea where I: IntoIterator<Item = AnyElement>,",
            ][..],
            &[
                "pub fn scroll_area<H: UiHost, I>( cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> I, ) -> AnyElement where I: IntoIterator<Item = AnyElement>,",
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
fn dropdown_menu_root_promotes_uncontrolled_builder_path_and_keeps_managed_open_seams_explicit() {
    let normalized = normalize_ws(DROPDOWN_MENU_RS);
    let required_markers = [
        "pub fn from_open(open: Model<bool>) -> Self {",
        "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {",
        "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, ) -> Self {",
        "pub fn compose<H: UiHost>(self) -> DropdownMenuComposition<H> {",
        "pub fn build<H: UiHost, I>(",
        "pub fn build_parts<H: UiHost, I>(",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "dropdown_menu.rs should promote `uncontrolled(...)` + `compose(...)` for the default authoring path while keeping `build(...)` / `build_parts(...)` and managed-open seams explicit"
        );
    }
}

#[test]
fn context_menu_root_promotes_uncontrolled_builder_path_and_keeps_managed_open_seams_explicit() {
    let normalized = normalize_ws(CONTEXT_MENU_RS);
    let required_markers = [
        "pub fn build<H: UiHost, T>(child: T) -> ContextMenuTriggerBuild<H, T>",
        "pub fn from_open(open: Model<bool>) -> Self {",
        "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {",
        "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, ) -> Self {",
        "pub fn compose<H: UiHost>(self) -> ContextMenuComposition<H> {",
        "pub fn build<H: UiHost, I>(",
        "pub fn build_parts<H: UiHost, I>(",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "context_menu.rs should promote `uncontrolled(...)` + `compose(...)` for the default authoring path while keeping `build(...)` / `build_parts(...)` and managed-open seams explicit"
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
fn tooltip_content_helpers_prefer_typed_build_and_text_outputs_when_slot_scope_is_required() {
    let normalized = normalize_ws(TOOLTIP_RS);
    let required_markers = [
        "pub fn build<H: UiHost, I, F, T>( cx: &mut ElementContext<'_, H>, f: F ) -> Self where F: FnOnce(&mut ElementContext<'_, H>) -> I, I: IntoIterator<Item = T>, T: IntoUiElement<H>,",
        "pub fn text<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T> where T: Into<Arc<str>>,",
    ];
    let forbidden_markers = [
        "pub fn with<H: UiHost>( cx: &mut ElementContext<'_, H>, f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>, ) -> AnyElement",
        "pub fn text<H: UiHost>( cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>, ) -> AnyElement",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "tooltip.rs should expose typed content helper outputs while keeping slot-scoped child construction available"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "tooltip.rs reintroduced eager `AnyElement` helper surfaces for tooltip content authoring"
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
fn state_helpers_prefer_typed_badge_outputs_when_no_runtime_landing_seam_is_required() {
    let normalized = normalize_ws(STATE_RS);
    let required_markers = [
        "pub fn use_selector_badge<H, Deps, TValue>( cx: &mut ElementContext<'_, H>, variant: BadgeVariant, deps: impl FnOnce(&mut ElementContext<'_, H>) -> Deps, compute: impl FnOnce(&mut ElementContext<'_, H>) -> TValue, ) -> Badge where H: UiHost, Deps: Any + PartialEq, TValue: Any + Clone + ToString,",
        "pub fn query_status_badge<H: UiHost, T>( _cx: &mut ElementContext<'_, H>, state: &QueryState<T>, ) -> Badge",
        "pub fn query_error_alert<H: UiHost, T>( cx: &mut ElementContext<'_, H>, state: &QueryState<T>, ) -> Option<Alert>",
    ];
    let forbidden_markers = [
        "pub fn use_selector_badge<H, Deps, TValue>( cx: &mut ElementContext<'_, H>, variant: BadgeVariant, deps: impl FnOnce(&mut ElementContext<'_, H>) -> Deps, compute: impl FnOnce(&mut ElementContext<'_, H>) -> TValue, ) -> AnyElement where H: UiHost, Deps: Any + PartialEq, TValue: Any + Clone + ToString,",
        "pub fn query_status_badge<H: UiHost, T>( cx: &mut ElementContext<'_, H>, state: &QueryState<T>, ) -> AnyElement",
        "pub fn query_error_alert<H: UiHost, T>( cx: &mut ElementContext<'_, H>, state: &QueryState<T>, ) -> Option<AnyElement>",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "state.rs should expose typed badge helper outputs where the public helper does not need to own a landing seam"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "state.rs reintroduced eager `AnyElement` helper outputs for badge-only state helpers"
        );
    }
}

#[test]
fn selector_and_query_helpers_stay_isolated_to_opt_in_state_module() {
    let normalized_lib = normalize_ws(LIB_RS);
    assert!(
        normalized_lib.contains(
            "#[cfg(any(feature=\"state-selector\",feature=\"state-query\"))]pubmodstate;"
        ),
        "lib.rs should keep the state helper module behind explicit opt-in features"
    );
    assert!(
        normalized_lib
            .contains("#[cfg(feature=\"state-selector\")]pubusecrate::state::use_selector_badge;"),
        "lib.rs should re-export selector helpers only behind the selector feature gate"
    );
    assert!(
        normalized_lib.contains(
            "#[cfg(feature=\"state-query\")]pubusecrate::state::{query_error_alert,query_status_badge};"
        ),
        "lib.rs should re-export query helpers only behind the query feature gate"
    );
    assert!(
        STATE_RS.contains("use fret_query::{QueryState, QueryStatus};"),
        "state.rs should remain the explicit query-aware helper seam"
    );
    assert!(
        STATE_RS.contains("use fret_selector::ui::SelectorElementContextExt as _;"),
        "state.rs should remain the explicit selector-aware helper seam"
    );

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    visit_rust_files(&root, &mut |path, source| {
        let rel = path.strip_prefix(&root).unwrap_or(path);
        if rel == std::path::Path::new("state.rs")
            || rel == std::path::Path::new("surface_policy_tests.rs")
        {
            return;
        }

        for marker in [
            "use fret_query",
            "fret_query::",
            "use fret_selector",
            "fret_selector::",
        ] {
            assert!(
                !source.contains(marker),
                "{} reintroduced state-stack marker `{marker}` outside the explicit `state.rs` seam",
                rel.display()
            );
        }
    });
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
fn combobox_surface_uses_generic_popover_anchor_builder_not_combobox_specific_raw_alias() {
    let combobox_normalized = normalize_ws(COMBOBOX_RS);
    let popover_normalized = normalize_ws(POPOVER_RS);
    let alias_marker =
        normalize_ws("pub fn use_combobox_anchor(child: AnyElement) -> PopoverAnchor");

    assert!(
        !combobox_normalized.contains(&alias_marker),
        "combobox.rs should not reintroduce a combobox-specific raw anchor alias now that `PopoverAnchor::build(...)` exists"
    );
    assert!(
        !LIB_RS.contains("use_combobox_anchor"),
        "lib.rs should not re-export the removed combobox-specific anchor alias"
    );

    let required_markers = [
        "pub fn build<H: UiHost, T>(child: T) -> PopoverAnchorBuild<H, T> where T: IntoUiElement<H>",
        "pub fn into_anchor(self, cx: &mut ElementContext<'_, H>) -> PopoverAnchor",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            popover_normalized.contains(&marker),
            "popover.rs should keep the generic builder-first anchor path available for combobox-style anchor overrides"
        );
    }
}

#[test]
fn explicit_raw_or_bridge_public_anyelement_helpers_stay_small_and_reviewable() {
    let mut hits = Vec::new();
    for (label, source) in [
        ("src/kbd.rs", KBD_RS),
        ("src/text_edit_context_menu.rs", TEXT_EDIT_CONTEXT_MENU_RS),
    ] {
        for signature in public_non_method_anyelement_signatures(source) {
            hits.push(format!("{label}: {signature}"));
        }
    }

    assert_eq!(
        hits,
        vec![
            String::from(
                "src/kbd.rs: pub fn kbd_icon<H: UiHost>(cx: &mut ElementContext<'_, H>, icon: IconId) -> AnyElement"
            ),
            String::from(
                "src/text_edit_context_menu.rs: pub fn text_edit_context_menu<H: UiHost, TTrigger>( cx: &mut ElementContext<'_, H>, open: Model<bool>, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, ) -> AnyElement where TTrigger: IntoUiElement<H>,"
            ),
            String::from(
                "src/text_edit_context_menu.rs: pub fn text_selection_context_menu<H: UiHost, TTrigger>( cx: &mut ElementContext<'_, H>, open: Model<bool>, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, ) -> AnyElement where TTrigger: IntoUiElement<H>,"
            ),
            String::from(
                "src/text_edit_context_menu.rs: pub fn text_edit_context_menu_controllable<H: UiHost, TTrigger>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, ) -> AnyElement where TTrigger: IntoUiElement<H>,"
            ),
            String::from(
                "src/text_edit_context_menu.rs: pub fn text_selection_context_menu_controllable<H: UiHost, TTrigger>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, ) -> AnyElement where TTrigger: IntoUiElement<H>,"
            ),
        ],
        "explicit raw/bridge helper entry points should remain small, reviewable, and opt-in"
    );
}

#[test]
fn text_edit_context_menu_helpers_keep_landing_seam_explicit_but_accept_typed_triggers() {
    assert!(
        TEXT_EDIT_CONTEXT_MENU_RS.contains(
            "This intentionally stays `-> AnyElement` because `ContextMenu::build(...)` is itself the final"
        ),
        "text_edit_context_menu.rs should explicitly document that the family keeps `-> AnyElement` as a deliberate final wrapper seam"
    );

    let normalized = normalize_ws(TEXT_EDIT_CONTEXT_MENU_RS);
    let required_markers = [
        "pub fn text_edit_context_menu<H: UiHost, TTrigger>( cx: &mut ElementContext<'_, H>, open: Model<bool>, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, ) -> AnyElement where TTrigger: IntoUiElement<H>,",
        "pub fn text_selection_context_menu<H: UiHost, TTrigger>( cx: &mut ElementContext<'_, H>, open: Model<bool>, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, ) -> AnyElement where TTrigger: IntoUiElement<H>,",
        "pub fn text_edit_context_menu_controllable<H: UiHost, TTrigger>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, ) -> AnyElement where TTrigger: IntoUiElement<H>,",
        "pub fn text_selection_context_menu_controllable<H: UiHost, TTrigger>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> TTrigger, ) -> AnyElement where TTrigger: IntoUiElement<H>,",
    ];
    let forbidden_markers = [
        "pub fn text_edit_context_menu<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Model<bool>, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement, ) -> AnyElement",
        "pub fn text_selection_context_menu<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Model<bool>, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement, ) -> AnyElement",
        "pub fn text_edit_context_menu_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement, ) -> AnyElement",
        "pub fn text_selection_context_menu_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement, ) -> AnyElement",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "text_edit_context_menu.rs should keep the root landing seam explicit while letting callers stay on the typed trigger lane"
        );
    }
    for marker in forbidden_markers {
        let marker = normalize_ws(marker);
        assert!(
            !normalized.contains(&marker),
            "text_edit_context_menu.rs reintroduced pre-landed `AnyElement` trigger requirements on the public helper surface"
        );
    }
}

#[test]
fn legacy_public_anyelement_helper_inventory_is_explicit_until_promoted_or_deleted() {
    let mut hits = Vec::new();
    for (label, source) in [
        ("src/combobox.rs", COMBOBOX_RS),
        ("src/drawer.rs", DRAWER_RS),
        ("src/menubar.rs", MENUBAR_RS),
    ] {
        for signature in public_non_method_anyelement_signatures(source) {
            hits.push(format!("{label}: {signature}"));
        }
    }

    assert_eq!(
        hits,
        Vec::<String>::new(),
        "legacy module-local `-> AnyElement` helpers should stay empty once the old root helpers are deleted"
    );
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
        for signature in public_non_method_anyelement_signatures(source) {
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
fn default_facing_clickable_widgets_keep_action_first_aliases_on_public_builders() {
    for (label, source, markers) in [
        (
            "breadcrumb.rs",
            BREADCRUMB_RS,
            &[
                "self.link = self.link.action(action);",
                "self.command = Some(action.into());",
            ][..],
        ),
        (
            "input_group.rs",
            INPUT_GROUP_RS,
            &["Bind a stable action ID to this input-group button (action-first authoring)."][..],
        ),
        (
            "item.rs",
            ITEM_RS,
            &["Bind a stable action ID to this item (action-first authoring)."][..],
        ),
        (
            "pagination.rs",
            PAGINATION_RS,
            &[
                "Bind a stable action ID to this pagination link (action-first authoring).",
                "Bind a stable action ID to this pagination previous-link (action-first authoring).",
                "Bind a stable action ID to this pagination next-link (action-first authoring).",
                "Bind a stable action ID to this pagination link build wrapper (action-first authoring).",
            ][..],
        ),
        (
            "table.rs",
            TABLE_RS,
            &[
                "Bind a stable action ID to this table row (action-first authoring).",
                "Bind a stable action ID to this table-row build wrapper (action-first authoring).",
            ][..],
        ),
        (
            "sidebar.rs",
            SIDEBAR_RS,
            &[
                "Bind a stable action ID to this sidebar trigger (action-first authoring).",
                "Bind a stable action ID to this sidebar rail (action-first authoring).",
                "Bind a stable action ID to this sidebar group action (action-first authoring).",
                "Bind a stable action ID to this sidebar menu action (action-first authoring).",
                "Bind a stable action ID to this sidebar menu sub-button (action-first authoring).",
                "Bind a stable action ID to this sidebar menu button (action-first authoring).",
            ][..],
        ),
    ] {
        for marker in markers {
            assert!(
                source.contains(marker),
                "{label} should keep action-first alias docs on default-facing clickable widgets"
            );
        }
        assert!(
            source.contains(
                "pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {"
            ),
            "{label} should expose an action-first builder alias"
        );
    }
}

#[test]
fn sonner_message_options_prefer_explicit_action_id_aliases_for_toast_actions() {
    let normalized = normalize_ws(SONNER_RS);
    let required_markers = [
        "pub fn action_id( self, label: impl Into<Arc<str>>, action: impl Into<fret_runtime::ActionId>, ) -> Self {",
        "pub fn action_command(self, label: impl Into<Arc<str>>, command: impl Into<CommandId>) -> Self {",
        "pub fn cancel_id( self, label: impl Into<Arc<str>>, action: impl Into<fret_runtime::ActionId>, ) -> Self {",
        "pub fn cancel_command(self, label: impl Into<Arc<str>>, command: impl Into<CommandId>) -> Self {",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "sonner.rs should expose explicit action-id/command aliases for toast message actions"
        );
    }
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
