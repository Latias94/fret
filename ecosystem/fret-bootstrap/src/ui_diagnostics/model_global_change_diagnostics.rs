#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiModelChangeHotspotV1 {
    pub model: u64,
    pub observation_edges: u32,
    #[serde(default)]
    pub changed_type: Option<String>,
    #[serde(default)]
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiModelChangeHotspotV1 {
    fn from_hotspot(hotspot: &fret_ui::tree::UiDebugModelChangeHotspot) -> Self {
        let changed_type = hotspot.changed.map(|c| c.type_name.to_string());
        let changed_at = hotspot.changed.map(|c| UiSourceLocationV1 {
            file: c.file.to_string(),
            line: c.line,
            column: c.column,
        });
        Self {
            model: hotspot.model.data().as_ffi(),
            observation_edges: hotspot.observation_edges,
            changed_type,
            changed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSourceLocationV1 {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiModelChangeUnobservedV1 {
    pub model: u64,
    pub created_type: Option<String>,
    pub created_at: Option<UiSourceLocationV1>,
    #[serde(default)]
    pub changed_type: Option<String>,
    #[serde(default)]
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiModelChangeUnobservedV1 {
    fn from_unobserved(unobserved: &fret_ui::tree::UiDebugModelChangeUnobserved) -> Self {
        let created_type = unobserved.created.map(|c| c.type_name.to_string());
        let created_at = unobserved.created.map(|c| UiSourceLocationV1 {
            file: c.file.to_string(),
            line: c.line,
            column: c.column,
        });
        let changed_type = unobserved.changed.map(|c| c.type_name.to_string());
        let changed_at = unobserved.changed.map(|c| UiSourceLocationV1 {
            file: c.file.to_string(),
            line: c.line,
            column: c.column,
        });

        Self {
            model: unobserved.model.data().as_ffi(),
            created_type,
            created_at,
            changed_type,
            changed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiGlobalChangeHotspotV1 {
    pub type_name: String,
    pub observation_edges: u32,
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiGlobalChangeHotspotV1 {
    fn from_hotspot(app: &App, hotspot: &fret_ui::tree::UiDebugGlobalChangeHotspot) -> Self {
        let type_name = app
            .global_type_name(hotspot.global)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{:?}", hotspot.global));
        let changed_at = app
            .global_changed_at(hotspot.global)
            .map(|at| UiSourceLocationV1 {
                file: at.file().to_string(),
                line: at.line(),
                column: at.column(),
            });

        Self {
            type_name,
            observation_edges: hotspot.observation_edges,
            changed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiGlobalChangeUnobservedV1 {
    pub type_name: String,
    pub changed_at: Option<UiSourceLocationV1>,
}

impl UiGlobalChangeUnobservedV1 {
    fn from_unobserved(
        app: &App,
        unobserved: &fret_ui::tree::UiDebugGlobalChangeUnobserved,
    ) -> Self {
        let type_name = app
            .global_type_name(unobserved.global)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{:?}", unobserved.global));
        let changed_at = app
            .global_changed_at(unobserved.global)
            .map(|at| UiSourceLocationV1 {
                file: at.file().to_string(),
                line: at.line(),
                column: at.column(),
            });

        Self {
            type_name,
            changed_at,
        }
    }
}
