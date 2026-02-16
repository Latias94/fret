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
        "Switch".to_string(),
        component("Boolean toggle")
            .prop(
                "checked",
                desc(
                    CatalogPropV1::boolean(),
                    "Checked value (boolean or expression). Use {\"$bindState\": \"/path\"} for two-way binding.",
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
