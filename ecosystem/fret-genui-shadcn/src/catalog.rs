//! GenUI catalog definition for the `fret-ui-shadcn`-backed resolver.
//!
//! This catalog is the "guardrail contract" for LLM output and tooling.
//! Keep it conservative and stable: prefer a small, predictable surface.

use std::collections::{BTreeMap, BTreeSet};

use fret_genui_core::catalog::{
    CatalogActionV1, CatalogComponentV1, CatalogPropV1, CatalogV1, CatalogValueTypeV1,
};
use serde_json::json;

const SPACE_TOKENS: [&str; 15] = [
    "N0", "N0p5", "N1", "N1p5", "N2", "N2p5", "N3", "N3p5", "N4", "N5", "N6", "N8", "N10", "N11",
    "N12",
];

pub fn shadcn_catalog_v1() -> CatalogV1 {
    let mut catalog = CatalogV1::new();

    catalog.components = shadcn_components_v1();
    catalog.actions = shadcn_actions_v1();

    catalog
}

fn shadcn_components_v1() -> BTreeMap<String, CatalogComponentV1> {
    let mut out = BTreeMap::new();

    out.insert(
        "Text".to_string(),
        component("Plain text")
            .prop(
                "text",
                desc(CatalogPropV1::any(), "Text content (string or expression)."),
            )
            .prop(
                "content",
                desc(
                    CatalogPropV1::any(),
                    "json-render alias for `text` (string or expression).",
                ),
            )
            .prop(
                "muted",
                desc(
                    CatalogPropV1::boolean(),
                    "json-render alias: when true and `variant` is unset, renders as muted text.",
                ),
            )
            .prop(
                "variant",
                CatalogPropV1::enum_values([
                    "body",
                    "muted",
                    "small",
                    "lead",
                    "large",
                    "h1",
                    "h2",
                    "h3",
                    "h4",
                    "inlineCode",
                ])
                .default_value(json!("body")),
            )
            .note("Prefer `text` + `variant`; `content`/`muted` are compatibility aliases.")
            .build(),
    );
    out.insert(
        "Heading".to_string(),
        component("json-render Heading (typography alias)")
            .prop(
                "text",
                desc(
                    CatalogPropV1::string().required(true),
                    "Heading text content.",
                ),
            )
            .prop(
                "level",
                desc(
                    CatalogPropV1::enum_values(["h1", "h2", "h3", "h4"]).default_value(json!("h2")),
                    "Heading level (mapped to shadcn typography).",
                ),
            )
            .build(),
    );
    out.insert(
        "Stack".to_string(),
        component("json-render Stack (layout alias)")
            .prop(
                "direction",
                desc(
                    CatalogPropV1::enum_values(["horizontal", "vertical"])
                        .default_value(json!("vertical")),
                    "Flex direction.",
                ),
            )
            .prop(
                "gap",
                desc(
                    CatalogPropV1::enum_values(["sm", "md", "lg"]).default_value(json!("md")),
                    "Gap token mapped to Space (sm=N2, md=N4, lg=N6).",
                ),
            )
            .build(),
    );
    out.insert(
        "Label".to_string(),
        component("Form label text (medium weight)")
            .prop(
                "text",
                desc(
                    CatalogPropV1::any().required(true),
                    "Label text (string or expression).",
                ),
            )
            .build(),
    );
    out.insert(
        "VStack".to_string(),
        component("Vertical stack (flex column)")
            .prop(
                "gap",
                desc(
                    CatalogPropV1::enum_values(SPACE_TOKENS),
                    "Space token between children (e.g. N2).",
                ),
            )
            .prop(
                "p",
                desc(CatalogPropV1::enum_values(SPACE_TOKENS), "Padding token."),
            )
            .prop(
                "px",
                desc(
                    CatalogPropV1::enum_values(SPACE_TOKENS),
                    "Horizontal padding token.",
                ),
            )
            .prop(
                "py",
                desc(
                    CatalogPropV1::enum_values(SPACE_TOKENS),
                    "Vertical padding token.",
                ),
            )
            .prop(
                "items",
                desc(
                    CatalogPropV1::enum_values(["start", "center", "end", "stretch"]),
                    "Cross-axis alignment.",
                ),
            )
            .prop(
                "justify",
                desc(
                    CatalogPropV1::enum_values(["start", "center", "end", "between"]),
                    "Main-axis alignment.",
                ),
            )
            .prop(
                "wrap",
                desc(
                    CatalogPropV1::boolean(),
                    "Allow wrapping (mostly relevant for HStack).",
                ),
            )
            .prop(
                "wFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(true)),
                    "Fill parent width.",
                ),
            )
            .prop(
                "hFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Fill parent height.",
                ),
            )
            .prop(
                "minW0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in width (fix row overflow).",
                ),
            )
            .prop(
                "minH0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in height (fix column overflow).",
                ),
            )
            .build(),
    );
    out.insert(
        "HStack".to_string(),
        component("Horizontal stack (flex row)")
            .prop(
                "gap",
                desc(
                    CatalogPropV1::enum_values(SPACE_TOKENS),
                    "Space token between children (e.g. N2).",
                ),
            )
            .prop(
                "p",
                desc(CatalogPropV1::enum_values(SPACE_TOKENS), "Padding token."),
            )
            .prop(
                "px",
                desc(
                    CatalogPropV1::enum_values(SPACE_TOKENS),
                    "Horizontal padding token.",
                ),
            )
            .prop(
                "py",
                desc(
                    CatalogPropV1::enum_values(SPACE_TOKENS),
                    "Vertical padding token.",
                ),
            )
            .prop(
                "items",
                desc(
                    CatalogPropV1::enum_values(["start", "center", "end", "stretch"]),
                    "Cross-axis alignment.",
                ),
            )
            .prop(
                "justify",
                desc(
                    CatalogPropV1::enum_values(["start", "center", "end", "between"]),
                    "Main-axis alignment.",
                ),
            )
            .prop(
                "wrap",
                desc(
                    CatalogPropV1::boolean(),
                    "Allow wrapping (recommended for button rows).",
                ),
            )
            .prop(
                "wFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(true)),
                    "Fill parent width.",
                ),
            )
            .prop(
                "hFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Fill parent height.",
                ),
            )
            .prop(
                "minW0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in width (use with Input.flex1 in rows).",
                ),
            )
            .prop(
                "minH0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in height.",
                ),
            )
            .build(),
    );
    out.insert(
        "Box".to_string(),
        component("Generic container (padding + sizing)")
            .prop(
                "p",
                desc(CatalogPropV1::enum_values(SPACE_TOKENS), "Padding token."),
            )
            .prop(
                "px",
                desc(
                    CatalogPropV1::enum_values(SPACE_TOKENS),
                    "Horizontal padding token.",
                ),
            )
            .prop(
                "py",
                desc(
                    CatalogPropV1::enum_values(SPACE_TOKENS),
                    "Vertical padding token.",
                ),
            )
            .prop(
                "wFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(true)),
                    "Fill parent width.",
                ),
            )
            .prop(
                "hFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Fill parent height.",
                ),
            )
            .prop(
                "minW0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in width (fix overflow).",
                ),
            )
            .prop(
                "minH0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in height (fix overflow).",
                ),
            )
            .note("Box is not a stack: use VStack/HStack for `gap/items/justify/wrap`.")
            .build(),
    );
    out.insert(
        "Card".to_string(),
        component("Card container")
            .prop("wrapContent", CatalogPropV1::boolean())
            .note("Default: wraps children with CardContent; set wrapContent=false to provide CardHeader/CardContent/CardFooter explicitly.")
            .note("Prefer Box for generic padding/sizing boundaries; do not use CardContent as a general-purpose padding wrapper.")
            .build(),
    );
    out.insert(
        "CardHeader".to_string(),
        component("Card header section (layout only)").build(),
    );
    out.insert(
        "CardContent".to_string(),
        component("Card content section (layout only)")
            .note(
                "CardContent is for Card internals; use Box for generic padding/sizing boundaries.",
            )
            .build(),
    );
    out.insert(
        "CardFooter".to_string(),
        component("Card footer section (layout only)").build(),
    );
    out.insert(
        "CardTitle".to_string(),
        component("Card title text")
            .prop("text", CatalogPropV1::any())
            .prop("title", CatalogPropV1::any())
            .build(),
    );
    out.insert(
        "CardDescription".to_string(),
        component("Card description text")
            .prop("text", CatalogPropV1::any())
            .prop("description", CatalogPropV1::any())
            .build(),
    );
    out.insert(
        "Button".to_string(),
        component("Button (clickable)")
            .prop(
                "label",
                desc(CatalogPropV1::any(), "Button label (string or expression)."),
            )
            .prop(
                "disabled",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Disable interaction (UI-only; action policy is app-owned).",
                ),
            )
            .prop(
                "variant",
                desc(
                    CatalogPropV1::enum_values([
                        "default",
                        "destructive",
                        "outline",
                        "secondary",
                        "ghost",
                        "link",
                    ])
                    .default_value(json!("default")),
                    "Button variant (shadcn).",
                ),
            )
            .prop(
                "size",
                desc(
                    CatalogPropV1::enum_values(["default", "sm", "lg", "icon", "iconSm", "iconLg"])
                        .default_value(json!("default")),
                    "Button size (shadcn).",
                ),
            )
            .prop(
                "wFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Fill parent width.",
                ),
            )
            .prop(
                "flex1",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Expand to fill available space in a row.",
                ),
            )
            .prop(
                "minW0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in width (fix overflow).",
                ),
            )
            .events(["press"])
            .build(),
    );
    out.insert(
        "Badge".to_string(),
        component("Badge label")
            .prop("label", CatalogPropV1::any())
            .prop(
                "text",
                desc(
                    CatalogPropV1::any(),
                    "json-render alias for `label` (string or expression).",
                ),
            )
            .prop(
                "variant",
                CatalogPropV1::enum_values(["default", "secondary", "destructive", "outline"]),
            )
            .note("variant: default|secondary|destructive|outline")
            .build(),
    );
    out.insert(
        "Input".to_string(),
        component("Single-line input")
            .prop(
                "label",
                desc(
                    CatalogPropV1::string(),
                    "Optional label rendered above the input (json-render compatibility).",
                ),
            )
            .prop(
                "placeholder",
                desc(CatalogPropV1::string(), "Placeholder text."),
            )
            .prop(
                "value",
                desc(
                    CatalogPropV1::string(),
                    "Input value (string or expression). Use {\"$bindState\": \"/path\"} for two-way binding.",
                ),
            )
            .prop(
                "type",
                desc(
                    CatalogPropV1::enum_values(["text", "email", "password", "number", "tel"])
                        .nullable(true),
                    "Input type (parsed by apps; currently UI-only compatibility).",
                ),
            )
            .prop(
                "disabled",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Disable interaction.",
                ),
            )
            .prop(
                "ariaInvalid",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Apply error chrome (aria-invalid).",
                ),
            )
            .prop(
                "checks",
                desc(
                    CatalogPropV1::array_of(CatalogPropV1::object_fields_allowing_additional([
                        (
                            "type",
                            desc(
                                CatalogPropV1::string().required(true),
                                "Validation check type (e.g. required, email, minLength).",
                            ),
                        ),
                        (
                            "message",
                            desc(
                                CatalogPropV1::string().required(true),
                                "Error message for this check.",
                            ),
                        ),
                        (
                            "args",
                            desc(
                                CatalogPropV1::object_fields_allowing_additional(
                                    std::iter::empty::<(String, CatalogPropV1)>(),
                                ),
                                "Optional check arguments (values or {\"$state\":\"/path\"}).",
                            ),
                        ),
                    ])),
                    "Optional validation checks (json-render-style).",
                ),
            )
            .prop(
                "validateOn",
                desc(
                    CatalogPropV1::enum_values(["change", "blur", "submit"]),
                    "When to run validation (UI policy).",
                ),
            )
            .prop(
                "validateEnabled",
                desc(
                    CatalogPropV1::any(),
                    "Optional condition to enable validation (VisibilityConditionV1 shape).",
                ),
            )
            .prop(
                "wFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Fill parent width.",
                ),
            )
            .prop(
                "flex1",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Expand to fill available space in a row.",
                ),
            )
            .prop(
                "minW0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in width (use with flex1=true inside HStack).",
                ),
            )
            .note("Row pattern: HStack(items=center, wrap=true) + Input(flex1=true, minW0=true) to avoid overflow.")
            .build(),
    );
    out.insert(
        "Form".to_string(),
        component("json-render Form (Enter key submit wrapper)")
            .events(["submit"])
            .note("Form listens for Enter/NumpadEnter and emits `submit` (no implicit submit button).")
            .build(),
    );
    out.insert(
        "Textarea".to_string(),
        component("Multi-line text input")
            .prop(
                "label",
                desc(
                    CatalogPropV1::string(),
                    "Optional label rendered above the textarea (json-render compatibility).",
                ),
            )
            .prop(
                "placeholder",
                desc(CatalogPropV1::string(), "Placeholder text."),
            )
            .prop(
                "value",
                desc(
                    CatalogPropV1::string(),
                    "Textarea value (string or expression). Use {\"$bindState\": \"/path\"} for two-way binding.",
                ),
            )
            .prop(
                "rows",
                desc(
                    CatalogPropV1::integer().nullable(true),
                    "Row count hint (json-render compatibility; currently ignored).",
                ),
            )
            .prop(
                "minHeightPx",
                desc(
                    CatalogPropV1::integer().default_value(json!(64)),
                    "Minimum height in px.",
                ),
            )
            .prop(
                "disabled",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Disable interaction.",
                ),
            )
            .prop(
                "ariaInvalid",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Apply error chrome (aria-invalid).",
                ),
            )
            .prop(
                "wFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Fill parent width.",
                ),
            )
            .prop(
                "flex1",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Expand to fill available space in a row.",
                ),
            )
            .prop(
                "minW0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in width (use with flex1=true inside HStack).",
                ),
            )
            .build(),
    );
    out.insert(
        "Select".to_string(),
        component("Select dropdown")
            .prop(
                "value",
                desc(
                    CatalogPropV1::string().nullable(true),
                    "Selected value (string or expression). Use {\"$bindState\": \"/path\"} for two-way binding.",
                ),
            )
            .prop(
                "placeholder",
                desc(CatalogPropV1::string(), "Placeholder when no value is selected."),
            )
            .prop(
                "options",
                desc(
                    CatalogPropV1::array_of(CatalogPropV1::object_fields_allowing_additional([
                        (
                            "value",
                            desc(
                                CatalogPropV1::string().required(true),
                                "Option value (string).",
                            ),
                        ),
                        (
                            "label",
                            desc(
                                CatalogPropV1::any().required(true),
                                "Option label (string or expression).",
                            ),
                        ),
                        (
                            "disabled",
                            desc(CatalogPropV1::boolean(), "Disable this option."),
                        ),
                    ]))
                    .required(true),
                    "Options list.",
                ),
            )
            .prop(
                "disabled",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Disable interaction.",
                ),
            )
            .prop(
                "ariaInvalid",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Apply error chrome (aria-invalid).",
                ),
            )
            .prop(
                "wFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Fill parent width.",
                ),
            )
            .prop(
                "flex1",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Expand to fill available space in a row.",
                ),
            )
            .prop(
                "minW0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in width (use with flex1=true inside HStack).",
                ),
            )
            .note("Row pattern: HStack(items=center, wrap=true) + Select(flex1=true, minW0=true) to avoid overflow.")
            .build(),
    );
    out.insert(
        "Switch".to_string(),
        component("Boolean toggle")
            .prop(
                "checked",
                desc(
                    CatalogPropV1::boolean(),
                    "Checked value (boolean or expression). Use {\"$bindState\": \"/path\"} for two-way binding.",
                ),
            )
            .prop(
                "defaultChecked",
                desc(
                    CatalogPropV1::boolean().nullable(true),
                    "Uncontrolled default (json-render compatibility).",
                ),
            )
            .prop(
                "label",
                desc(CatalogPropV1::any(), "Optional label shown next to the switch."),
            )
            .prop(
                "disabled",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Disable interaction.",
                ),
            )
            .build(),
    );
    out.insert(
        "Checkbox".to_string(),
        component("Checkbox toggle (boolean)")
            .prop(
                "checked",
                desc(
                    CatalogPropV1::boolean(),
                    "Checked value (boolean or expression). Use {\"$bindState\": \"/path\"} for two-way binding.",
                ),
            )
            .prop(
                "defaultChecked",
                desc(
                    CatalogPropV1::boolean().nullable(true),
                    "Uncontrolled default (json-render compatibility).",
                ),
            )
            .prop(
                "label",
                desc(CatalogPropV1::any(), "Optional label shown next to the checkbox."),
            )
            .prop(
                "disabled",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Disable interaction.",
                ),
            )
            .build(),
    );
    out.insert(
        "Avatar".to_string(),
        component("json-render Avatar (fallback-only for now)")
            .prop(
                "src",
                desc(
                    CatalogPropV1::string().nullable(true),
                    "Image URL (currently ignored; Avatar renders fallback text).",
                ),
            )
            .prop(
                "alt",
                desc(
                    CatalogPropV1::string().nullable(true),
                    "Alt text (currently ignored).",
                ),
            )
            .prop(
                "fallback",
                desc(
                    CatalogPropV1::string().required(true),
                    "Fallback initials/text rendered when no image is available.",
                ),
            )
            .note("URL → ImageId ingestion is not implemented in GenUI yet; `src` is ignored.")
            .build(),
    );
    out.insert(
        "Tooltip".to_string(),
        component("Tooltip on hover (json-render)")
            .prop(
                "content",
                desc(
                    CatalogPropV1::string().required(true),
                    "Tooltip content text.",
                ),
            )
            .build(),
    );
    out.insert(
        "Popover".to_string(),
        component("Popover overlay with trigger (json-render)")
            .prop(
                "trigger",
                desc(
                    CatalogPropV1::string().required(true),
                    "Trigger button label.",
                ),
            )
            .build(),
    );
    out.insert(
        "DropdownMenu".to_string(),
        component("Dropdown menu with action items (json-render)")
            .prop(
                "trigger",
                desc(
                    CatalogPropV1::string().required(true),
                    "Trigger button label.",
                ),
            )
            .prop(
                "items",
                desc(
                    CatalogPropV1::array_of(CatalogPropV1::object_fields_allowing_additional([
                        (
                            "label",
                            desc(CatalogPropV1::string().required(true), "Menu item label."),
                        ),
                        (
                            "action",
                            desc(
                                CatalogPropV1::string().nullable(true),
                                "Action name (app-owned). When null, item is inert.",
                            ),
                        ),
                        (
                            "actionParams",
                            desc(
                                CatalogPropV1::object_fields_allowing_additional(
                                    std::iter::empty::<(String, CatalogPropV1)>(),
                                )
                                .nullable(true),
                                "Action params (values or expressions).",
                            ),
                        ),
                        (
                            "params",
                            desc(
                                CatalogPropV1::object_fields_allowing_additional(
                                    std::iter::empty::<(String, CatalogPropV1)>(),
                                )
                                .nullable(true),
                                "Alias for actionParams.",
                            ),
                        ),
                        (
                            "disabled",
                            desc(
                                CatalogPropV1::boolean().default_value(json!(false)),
                                "Disable interaction.",
                            ),
                        ),
                        (
                            "variant",
                            desc(
                                CatalogPropV1::enum_values(["default", "destructive"])
                                    .default_value(json!("default")),
                                "Visual variant.",
                            ),
                        ),
                        (
                            "type",
                            desc(
                                CatalogPropV1::enum_values(["item", "separator"])
                                    .default_value(json!("item")),
                                "Structural item type.",
                            ),
                        ),
                        (
                            "testId",
                            desc(
                                CatalogPropV1::string().nullable(true),
                                "Optional diagnostics selector anchor for the menu item.",
                            ),
                        ),
                    ]))
                    .required(true),
                    "Menu entries.",
                ),
            )
            .build(),
    );
    out.insert(
        "Dialog".to_string(),
        component("Modal dialog with trigger (json-render)")
            .prop(
                "trigger",
                desc(
                    CatalogPropV1::string().required(true),
                    "Trigger button label.",
                ),
            )
            .prop(
                "title",
                desc(CatalogPropV1::string().required(true), "Dialog title."),
            )
            .prop(
                "description",
                desc(
                    CatalogPropV1::string().nullable(true),
                    "Optional description text.",
                ),
            )
            .build(),
    );
    out.insert(
        "Drawer".to_string(),
        component("Slide-out drawer panel with trigger (json-render)")
            .prop(
                "trigger",
                desc(
                    CatalogPropV1::string().required(true),
                    "Trigger button label.",
                ),
            )
            .prop(
                "title",
                desc(CatalogPropV1::string().required(true), "Drawer title."),
            )
            .prop(
                "description",
                desc(
                    CatalogPropV1::string().nullable(true),
                    "Optional description text.",
                ),
            )
            .prop(
                "side",
                desc(
                    CatalogPropV1::enum_values(["top", "bottom", "left", "right"])
                        .default_value(json!("bottom")),
                    "Drawer side.",
                ),
            )
            .build(),
    );
    out.insert(
        "Pagination".to_string(),
        component("Page navigation (json-render)")
            .prop(
                "currentPage",
                desc(
                    CatalogPropV1::integer().required(true),
                    "Current page (1-based).",
                ),
            )
            .prop(
                "totalPages",
                desc(
                    CatalogPropV1::integer().required(true),
                    "Total pages (>= 1).",
                ),
            )
            .prop(
                "onPageChange",
                desc(
                    CatalogPropV1::string().nullable(true),
                    "Action name (app-owned). Params: { page: number }.",
                ),
            )
            .build(),
    );
    out.insert(
        "BarChart".to_string(),
        component("Bar chart (json-render; placeholder in GenUI for now)")
            .prop(
                "title",
                desc(CatalogPropV1::string().nullable(true), "Optional title."),
            )
            .prop(
                "data",
                desc(
                    CatalogPropV1::array_of(CatalogPropV1::object_fields_allowing_additional(
                        std::iter::empty::<(String, CatalogPropV1)>(),
                    )),
                    "Data array (objects).",
                ),
            )
            .prop(
                "xKey",
                desc(CatalogPropV1::string().required(true), "X axis key."),
            )
            .prop(
                "yKey",
                desc(CatalogPropV1::string().required(true), "Y axis key."),
            )
            .prop(
                "aggregate",
                desc(
                    CatalogPropV1::enum_values(["sum", "count", "avg"]).nullable(true),
                    "Aggregation policy.",
                ),
            )
            .prop(
                "color",
                desc(CatalogPropV1::string().nullable(true), "Color hint."),
            )
            .prop(
                "height",
                desc(CatalogPropV1::integer().nullable(true), "Height hint."),
            )
            .note("Chart rendering is not implemented yet; this component is a placeholder.")
            .build(),
    );
    out.insert(
        "LineChart".to_string(),
        component("Line chart (json-render; placeholder in GenUI for now)")
            .prop(
                "title",
                desc(CatalogPropV1::string().nullable(true), "Optional title."),
            )
            .prop(
                "data",
                desc(
                    CatalogPropV1::array_of(CatalogPropV1::object_fields_allowing_additional(
                        std::iter::empty::<(String, CatalogPropV1)>(),
                    )),
                    "Data array (objects).",
                ),
            )
            .prop(
                "xKey",
                desc(CatalogPropV1::string().required(true), "X axis key."),
            )
            .prop(
                "yKey",
                desc(CatalogPropV1::string().required(true), "Y axis key."),
            )
            .prop(
                "aggregate",
                desc(
                    CatalogPropV1::enum_values(["sum", "count", "avg"]).nullable(true),
                    "Aggregation policy.",
                ),
            )
            .prop(
                "color",
                desc(CatalogPropV1::string().nullable(true), "Color hint."),
            )
            .prop(
                "height",
                desc(CatalogPropV1::integer().nullable(true), "Height hint."),
            )
            .note("Chart rendering is not implemented yet; this component is a placeholder.")
            .build(),
    );
    out.insert(
        "RadioGroup".to_string(),
        component("Radio group (single choice)")
            .prop(
                "value",
                desc(
                    CatalogPropV1::string().nullable(true),
                    "Selected value (string or expression). Use {\"$bindState\": \"/path\"} for two-way binding.",
                ),
            )
            .prop(
                "options",
                desc(
                    CatalogPropV1::array_of(CatalogPropV1::object_fields_allowing_additional([
                        (
                            "value",
                            desc(
                                CatalogPropV1::string().required(true),
                                "Option value (string).",
                            ),
                        ),
                        (
                            "label",
                            desc(
                                CatalogPropV1::any().required(true),
                                "Option label (string or expression).",
                            ),
                        ),
                        (
                            "disabled",
                            desc(CatalogPropV1::boolean(), "Disable this option."),
                        ),
                    ]))
                    .required(true),
                    "Options list.",
                ),
            )
            .prop(
                "orientation",
                desc(
                    CatalogPropV1::enum_values(["vertical", "horizontal"])
                        .default_value(json!("vertical")),
                    "Layout orientation.",
                ),
            )
            .prop(
                "disabled",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Disable interaction.",
                ),
            )
            .prop(
                "ariaInvalid",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Apply error chrome to items (aria-invalid).",
                ),
            )
            .prop(
                "wFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Fill parent width.",
                ),
            )
            .prop(
                "flex1",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Expand to fill available space in a row.",
                ),
            )
            .prop(
                "minW0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in width (use with flex1=true inside HStack).",
                ),
            )
            .build(),
    );
    out.insert(
        "Slider".to_string(),
        component("Slider (single value; commits on release)")
            .prop(
                "value",
                desc(
                    CatalogPropV1::number(),
                    "Slider value (number or expression). Use {\"$bindState\": \"/path\"} for two-way binding.",
                ),
            )
            .prop(
                "min",
                desc(
                    CatalogPropV1::number().default_value(json!(0)),
                    "Minimum value.",
                ),
            )
            .prop(
                "max",
                desc(
                    CatalogPropV1::number().default_value(json!(100)),
                    "Maximum value.",
                ),
            )
            .prop(
                "step",
                desc(
                    CatalogPropV1::number().default_value(json!(1)),
                    "Step size.",
                ),
            )
            .prop(
                "disabled",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Disable interaction.",
                ),
            )
            .prop(
                "wFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Fill parent width.",
                ),
            )
            .prop(
                "flex1",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Expand to fill available space in a row.",
                ),
            )
            .prop(
                "minW0",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Allow shrinking in width (use with flex1=true inside HStack).",
                ),
            )
            .build(),
    );
    out.insert(
        "Separator".to_string(),
        component("Divider line")
            .prop(
                "orientation",
                CatalogPropV1::enum_values(["horizontal", "vertical"]),
            )
            .prop("flexStretchCrossAxis", CatalogPropV1::boolean())
            .build(),
    );
    out.insert(
        "ScrollArea".to_string(),
        component("Scroll container")
            .prop("axis", CatalogPropV1::enum_values(["x", "y", "both"]))
            .prop("showScrollbar", CatalogPropV1::boolean())
            .build(),
    );
    out.insert(
        "Alert".to_string(),
        component("Alert box")
            .prop(
                "variant",
                desc(
                    CatalogPropV1::enum_values(["default", "destructive"])
                        .default_value(json!("default")),
                    "Alert variant.",
                ),
            )
            .prop("title", desc(CatalogPropV1::any(), "Optional title text."))
            .prop(
                "description",
                desc(CatalogPropV1::any(), "Optional description text."),
            )
            .build(),
    );
    out.insert(
        "Progress".to_string(),
        component("Progress indicator")
            .prop(
                "value",
                desc(
                    CatalogPropV1::number(),
                    "Progress value (number or expression).",
                ),
            )
            .prop(
                "min",
                desc(
                    CatalogPropV1::number().default_value(json!(0)),
                    "Minimum value.",
                ),
            )
            .prop(
                "max",
                desc(
                    CatalogPropV1::number().default_value(json!(100)),
                    "Maximum value.",
                ),
            )
            .prop(
                "mirrorInRtl",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Mirror fill direction in RTL.",
                ),
            )
            .prop(
                "wFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(true)),
                    "Fill parent width.",
                ),
            )
            .build(),
    );
    out.insert(
        "Spinner".to_string(),
        component("Spinner")
            .prop(
                "sizePx",
                desc(
                    CatalogPropV1::integer().default_value(json!(16)),
                    "Icon size in px.",
                ),
            )
            .prop(
                "speed",
                desc(
                    CatalogPropV1::number().default_value(json!(0.12)),
                    "Rotation speed in radians per frame (0 disables animation).",
                ),
            )
            .build(),
    );
    out.insert(
        "Skeleton".to_string(),
        component("Skeleton block")
            .prop(
                "wFull",
                desc(
                    CatalogPropV1::boolean().default_value(json!(true)),
                    "Fill parent width.",
                ),
            )
            .prop(
                "hPx",
                desc(
                    CatalogPropV1::integer().default_value(json!(16)),
                    "Height in px.",
                ),
            )
            .prop(
                "secondary",
                desc(
                    CatalogPropV1::boolean().default_value(json!(false)),
                    "Reduced opacity variant.",
                ),
            )
            .prop(
                "animatePulse",
                desc(
                    CatalogPropV1::boolean().default_value(json!(true)),
                    "Enable pulse animation.",
                ),
            )
            .build(),
    );

    out.insert(
        "Table".to_string(),
        component("Data table (json-render-style): renders a table from columns[] + data[].")
            .prop(
                "data",
                desc(
                    CatalogPropV1::any().required(true),
                    "Rows data. Prefer {\"$state\":\"/path\"} pointing to an array of objects.",
                ),
            )
            .prop(
                "dataPath",
                desc(
                    CatalogPropV1::string(),
                    "Optional JSON Pointer to the backing state array (enables $item/$index in row action params).",
                ),
            )
            .prop(
                "columns",
                desc(
                    CatalogPropV1::array_of(CatalogPropV1::object_fields_allowing_additional([
                        (
                            "key",
                            desc(
                                CatalogPropV1::string().required(true),
                                "Field key for each row object.",
                            ),
                        ),
                        (
                            "label",
                            desc(
                                CatalogPropV1::string().required(true),
                                "Column header label.",
                            ),
                        ),
                    ]))
                    .required(true),
                    "Table columns.",
                ),
            )
            .prop(
                "rowActions",
                desc(
                    CatalogPropV1::array_of(CatalogPropV1::object_fields_allowing_additional([
                        (
                            "label",
                            desc(CatalogPropV1::string().required(true), "Button label."),
                        ),
                        (
                            "action",
                            desc(
                                CatalogPropV1::string().required(true),
                                "Action name to enqueue on press.",
                            ),
                        ),
                        (
                            "params",
                            desc(
                                CatalogPropV1::object_fields_allowing_additional(
                                    std::iter::empty::<(String, CatalogPropV1)>(),
                                ),
                                "Optional action params (values or expressions). If dataPath is set, $item/$index are available.",
                            ),
                        ),
                        (
                            "variant",
                            desc(
                                CatalogPropV1::enum_values([
                                    "default",
                                    "secondary",
                                    "destructive",
                                    "outline",
                                    "ghost",
                                    "link",
                                ]),
                                "Button variant.",
                            ),
                        ),
                        (
                            "disabled",
                            desc(
                                CatalogPropV1::boolean().default_value(json!(false)),
                                "Disable the action button.",
                            ),
                        ),
                    ])),
                    "Optional per-row action buttons.",
                ),
            )
            .prop(
                "emptyMessage",
                desc(CatalogPropV1::string(), "Text to show when the data array is empty."),
            )
            .note("Table is a macro component: it renders internal rows/cells; it does not require child elements.")
            .build(),
    );

    out.insert(
        "ResponsiveGrid".to_string(),
        component(
            "Responsive grid layout: chunks children into rows based on container/viewport width.",
        )
        .prop(
            "columns",
            CatalogPropV1::one_of([CatalogValueTypeV1::Integer, breakpoint_columns_object_ty()])
                .required(true),
        )
        .prop(
            "query",
            CatalogPropV1::enum_values(["container", "viewport"]).default_value(json!("container")),
        )
        .prop(
            "gap",
            CatalogPropV1::enum_values(SPACE_TOKENS).default_value(json!("N2")),
        )
        .prop(
            "fillLastRow",
            CatalogPropV1::boolean().default_value(json!(true)),
        )
        .build(),
    );

    out.insert(
        "ResponsiveStack".to_string(),
        component(
            "Responsive stack layout: switches between VStack/HStack via container/viewport width.",
        )
        .prop(
            "direction",
            CatalogPropV1::one_of([
                stack_direction_enum_ty(),
                breakpoint_stack_direction_object_ty(),
            ])
            .default_value(json!("vertical")),
        )
        .prop(
            "query",
            CatalogPropV1::enum_values(["container", "viewport"]).default_value(json!("container")),
        )
        .prop(
            "gap",
            CatalogPropV1::enum_values(SPACE_TOKENS).default_value(json!("N2")),
        )
        .build(),
    );

    out.insert(
        "Tabs".to_string(),
        component("Tabs (macro component): builds a shadcn TabsRoot from TabContent children.")
            .prop(
                "defaultValue",
                desc(CatalogPropV1::string(), "Initial tab value (uncontrolled)."),
            )
            .prop(
                "forceMountContent",
                desc(
                    CatalogPropV1::boolean().default_value(json!(true)),
                    "Keep all panels mounted and gate interactivity when inactive.",
                ),
            )
            .prop(
                "tabs",
                desc(
                    CatalogPropV1::array_of(CatalogPropV1::object_fields_allowing_additional([
                        (
                            "value",
                            desc(
                                CatalogPropV1::string().required(true),
                                "Tab value (stable id). Must match a TabContent.value.",
                            ),
                        ),
                        (
                            "label",
                            desc(
                                CatalogPropV1::string().required(true),
                                "Trigger label text.",
                            ),
                        ),
                    ]))
                    .required(true),
                    "Tab trigger definitions (order + labels).",
                ),
            )
            .note("Children must be TabContent elements; Tabs assembles the final widget from child meta.")
            .build(),
    );

    out.insert(
        "TabContent".to_string(),
        component("Tab panel content (used by Tabs macro component).")
            .prop(
                "value",
                desc(
                    CatalogPropV1::string().required(true),
                    "Tab value this panel belongs to.",
                ),
            )
            .note("Use as a child of Tabs. The TabContent element itself is treated as the panel content wrapper.")
            .build(),
    );

    out.insert(
        "Accordion".to_string(),
        component(
            "Accordion (macro component): builds a shadcn Accordion from AccordionItem children.",
        )
        .prop(
            "type",
            desc(
                CatalogPropV1::enum_values(["single", "multiple"]).default_value(json!("single")),
                "Accordion kind.",
            ),
        )
        .prop(
            "collapsible",
            desc(
                CatalogPropV1::boolean().default_value(json!(true)),
                "Single-mode only: allow collapsing the open item.",
            ),
        )
        .prop(
            "defaultValue",
            desc(CatalogPropV1::string(), "Single-mode only: initial open value."),
        )
        .prop(
            "defaultValues",
            desc(
                CatalogPropV1::array_of(CatalogPropV1::string()),
                "Multiple-mode only: initial open values.",
            ),
        )
        .note("Children must be AccordionItem elements; Accordion assembles the final widget from child meta.")
        .build(),
    );

    out.insert(
        "AccordionItem".to_string(),
        component("Accordion item content wrapper (used by Accordion macro component).")
            .prop(
                "value",
                desc(CatalogPropV1::string().required(true), "Item value (stable id)."),
            )
            .prop(
                "title",
                desc(
                    CatalogPropV1::string().required(true),
                    "Trigger title text.",
                ),
            )
            .note("Use as a child of Accordion. The AccordionItem element itself is treated as the item content wrapper.")
            .build(),
    );

    out
}

fn breakpoint_columns_object_ty() -> CatalogValueTypeV1 {
    let mut fields = BTreeMap::new();
    // Tailwind-compatible keys.
    fields.insert("base".to_string(), CatalogPropV1::integer());
    fields.insert("sm".to_string(), CatalogPropV1::integer());
    fields.insert("md".to_string(), CatalogPropV1::integer());
    fields.insert("lg".to_string(), CatalogPropV1::integer());
    fields.insert("xl".to_string(), CatalogPropV1::integer());
    fields.insert("xxl".to_string(), CatalogPropV1::integer());
    CatalogValueTypeV1::Object {
        fields,
        additional: false,
    }
}

fn stack_direction_enum_ty() -> CatalogValueTypeV1 {
    CatalogValueTypeV1::Enum {
        values: ["vertical", "horizontal"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

fn breakpoint_stack_direction_object_ty() -> CatalogValueTypeV1 {
    let mut fields = BTreeMap::new();
    // Tailwind-compatible keys.
    let dir = CatalogPropV1::one_of([stack_direction_enum_ty()]);
    fields.insert("base".to_string(), dir.clone());
    fields.insert("sm".to_string(), dir.clone());
    fields.insert("md".to_string(), dir.clone());
    fields.insert("lg".to_string(), dir.clone());
    fields.insert("xl".to_string(), dir.clone());
    fields.insert("xxl".to_string(), dir);
    CatalogValueTypeV1::Object {
        fields,
        additional: false,
    }
}

fn shadcn_actions_v1() -> BTreeMap<String, CatalogActionV1> {
    let mut out = BTreeMap::new();

    out.insert(
        "setState".to_string(),
        action("Write a JSON value at a JSON Pointer path.")
            .param(
                "statePath",
                desc(
                    CatalogPropV1::string().required(true),
                    "JSON Pointer path (e.g. /name)",
                ),
            )
            .param("value", desc(CatalogPropV1::any(), "New value (any JSON)"))
            .build(),
    );
    out.insert(
        "incrementState".to_string(),
        action("Increment an integer field at a JSON Pointer path.")
            .param(
                "statePath",
                desc(
                    CatalogPropV1::string().required(true),
                    "JSON Pointer path (e.g. /count)",
                ),
            )
            .param(
                "delta",
                desc(CatalogPropV1::integer(), "Integer delta (defaults to 1)"),
            )
            .build(),
    );
    out.insert(
        "pushState".to_string(),
        action("Append an item to an array at a JSON Pointer path.")
            .param(
                "statePath",
                desc(
                    CatalogPropV1::string().required(true),
                    "JSON Pointer path to an array (e.g. /todos)",
                ),
            )
            .param(
                "value",
                desc(
                    CatalogPropV1::any().required(true),
                    "New item value (any JSON); use \"$id\" to generate ids",
                ),
            )
            .param(
                "clearStatePath",
                desc(
                    CatalogPropV1::string(),
                    "Optional JSON Pointer path to clear after push",
                ),
            )
            .build(),
    );
    out.insert(
        "removeState".to_string(),
        action("Remove an item from an array at a JSON Pointer path by index.")
            .param(
                "statePath",
                desc(
                    CatalogPropV1::string().required(true),
                    "JSON Pointer path to an array (e.g. /todos)",
                ),
            )
            .param(
                "index",
                desc(
                    CatalogPropV1::integer().required(true),
                    "Array index (use {\"$index\": true} inside repeat)",
                ),
            )
            .build(),
    );

    out.insert(
        "openUrl".to_string(),
        action("Open a URL (portable effect; app-owned policy).")
            .param(
                "url",
                desc(
                    CatalogPropV1::string().required(true),
                    "URL to open (e.g. https://example.com).",
                ),
            )
            .param(
                "target",
                desc(CatalogPropV1::string(), "Optional target (web-only)."),
            )
            .param(
                "rel",
                desc(CatalogPropV1::string(), "Optional rel (web-only)."),
            )
            .build(),
    );

    out.insert(
        "clipboardSetText".to_string(),
        action("Set clipboard text (portable effect; app-owned policy).")
            .param(
                "text",
                desc(
                    CatalogPropV1::string().required(true),
                    "Text to copy to clipboard.",
                ),
            )
            .build(),
    );

    out.insert(
        "formSubmit".to_string(),
        action("Submit a form (app-owned validation/policy).")
            .param(
                "formName",
                desc(
                    CatalogPropV1::string(),
                    "Optional form name for UX/logging.",
                ),
            )
            .build(),
    );

    out
}

#[derive(Default)]
struct ComponentBuilder {
    c: CatalogComponentV1,
}

fn component(description: &str) -> ComponentBuilder {
    ComponentBuilder {
        c: CatalogComponentV1 {
            description: Some(description.to_string()),
            props: BTreeMap::new(),
            events: BTreeSet::new(),
        },
    }
}

impl ComponentBuilder {
    fn note(mut self, s: &str) -> Self {
        let base = self.c.description.take().unwrap_or_default();
        self.c.description = Some(if base.is_empty() {
            s.to_string()
        } else {
            format!("{base} {s}")
        });
        self
    }

    fn prop(mut self, name: &str, def: CatalogPropV1) -> Self {
        self.c.props.insert(name.to_string(), def);
        self
    }

    fn events<const N: usize>(mut self, names: [&'static str; N]) -> Self {
        for name in names {
            self.c.events.insert(name.to_string());
        }
        self
    }

    fn build(self) -> CatalogComponentV1 {
        self.c
    }
}

#[derive(Default)]
struct ActionBuilder {
    a: CatalogActionV1,
}

fn action(description: &str) -> ActionBuilder {
    ActionBuilder {
        a: CatalogActionV1 {
            description: Some(description.to_string()),
            params: BTreeMap::new(),
        },
    }
}

impl ActionBuilder {
    fn param(mut self, name: &str, def: CatalogPropV1) -> Self {
        self.a.params.insert(name.to_string(), def);
        self
    }

    fn build(self) -> CatalogActionV1 {
        self.a
    }
}

fn desc(mut prop: CatalogPropV1, description: &str) -> CatalogPropV1 {
    prop.description = Some(description.to_string());
    prop
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_exports_spec_schema() {
        let catalog = shadcn_catalog_v1();
        let schema = catalog.spec_json_schema();
        assert_eq!(
            schema
                .get("properties")
                .and_then(|p| p.get("schema_version"))
                .and_then(|v| v.get("const"))
                .and_then(|v| v.as_i64()),
            Some(1)
        );
    }
}
