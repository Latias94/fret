use std::sync::Arc;

use fret_core::{FontId, FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MermaidDiagramType {
    Flowchart,
    Sequence,
    Class,
    State,
    EntityRelationship,
    UserJourney,
    Gantt,
    Pie,
    Quadrant,
    Requirement,
    GitGraph,
    C4,
    Mindmap,
    Timeline,
    Kanban,
    Architecture,
    Packet,
    Info,
    Error,
    Radar,
    Treemap,
    ZenUML,
    Sankey,
    XYChart,
    Block,
    Unknown,
}

impl MermaidDiagramType {
    pub(super) fn display_name(&self) -> &'static str {
        match self {
            Self::Flowchart => "Flowchart",
            Self::Sequence => "Sequence Diagram",
            Self::Class => "Class Diagram",
            Self::State => "State Diagram",
            Self::EntityRelationship => "Entity-Relationship Diagram",
            Self::UserJourney => "User Journey",
            Self::Gantt => "Gantt Chart",
            Self::Pie => "Pie Chart",
            Self::Quadrant => "Quadrant Chart",
            Self::Requirement => "Requirement Diagram",
            Self::GitGraph => "Git Graph",
            Self::C4 => "C4 Diagram",
            Self::Mindmap => "Mindmap",
            Self::Timeline => "Timeline",
            Self::Kanban => "Kanban",
            Self::Architecture => "Architecture Diagram",
            Self::Packet => "Packet Diagram",
            Self::Info => "Info Diagram",
            Self::Error => "Error Diagram",
            Self::Radar => "Radar Chart",
            Self::Treemap => "Treemap",
            Self::ZenUML => "ZenUML Diagram",
            Self::Sankey => "Sankey Diagram",
            Self::XYChart => "XY Chart",
            Self::Block => "Block Diagram",
            Self::Unknown => "Diagram",
        }
    }
}

pub(super) fn is_mermaid_language(language: Option<&str>) -> bool {
    language
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .is_some_and(|s| s.eq_ignore_ascii_case("mermaid"))
}

pub(super) fn detect_mermaid_diagram_type(source: &str) -> MermaidDiagramType {
    // Find the first non-empty, non-comment line. Mermaid uses `%%` for comments.
    let first_line = source
        .lines()
        .map(|line| line.trim())
        .find(|line| !line.is_empty() && !line.starts_with("%%"))
        .unwrap_or("");

    let first_line_lower = first_line.to_ascii_lowercase();
    if first_line_lower.starts_with("flowchart")
        || first_line_lower.starts_with("graph")
        || first_line_lower.starts_with("flowchart-v2")
    {
        MermaidDiagramType::Flowchart
    } else if first_line_lower.starts_with("sequencediagram")
        || first_line_lower.starts_with("sequence")
    {
        MermaidDiagramType::Sequence
    } else if first_line_lower.starts_with("classdiagram") || first_line_lower.starts_with("class")
    {
        MermaidDiagramType::Class
    } else if first_line_lower.starts_with("statediagram") || first_line_lower.starts_with("state")
    {
        MermaidDiagramType::State
    } else if first_line_lower.starts_with("erdiagram") || first_line_lower == "er" {
        MermaidDiagramType::EntityRelationship
    } else if first_line_lower.starts_with("journey") {
        MermaidDiagramType::UserJourney
    } else if first_line_lower.starts_with("gantt") {
        MermaidDiagramType::Gantt
    } else if first_line_lower.starts_with("pie") {
        MermaidDiagramType::Pie
    } else if first_line_lower.starts_with("quadrantchart") {
        MermaidDiagramType::Quadrant
    } else if first_line_lower.starts_with("requirementdiagram")
        || first_line_lower.starts_with("requirement")
    {
        MermaidDiagramType::Requirement
    } else if first_line_lower.starts_with("gitgraph") {
        MermaidDiagramType::GitGraph
    } else if first_line_lower.starts_with("c4") {
        MermaidDiagramType::C4
    } else if first_line_lower.starts_with("mindmap") {
        MermaidDiagramType::Mindmap
    } else if first_line_lower.starts_with("timeline") {
        MermaidDiagramType::Timeline
    } else if first_line_lower.starts_with("kanban") {
        MermaidDiagramType::Kanban
    } else if first_line_lower.starts_with("architecture") {
        MermaidDiagramType::Architecture
    } else if first_line_lower.starts_with("packet") {
        MermaidDiagramType::Packet
    } else if first_line_lower.starts_with("info") {
        MermaidDiagramType::Info
    } else if first_line_lower.starts_with("error") {
        MermaidDiagramType::Error
    } else if first_line_lower.starts_with("radar") {
        MermaidDiagramType::Radar
    } else if first_line_lower.starts_with("treemap") {
        MermaidDiagramType::Treemap
    } else if first_line_lower.starts_with("zenuml") {
        MermaidDiagramType::ZenUML
    } else if first_line_lower.starts_with("sankey") {
        MermaidDiagramType::Sankey
    } else if first_line_lower.starts_with("xychart") {
        MermaidDiagramType::XYChart
    } else if first_line_lower.starts_with("block") {
        MermaidDiagramType::Block
    } else {
        MermaidDiagramType::Unknown
    }
}

pub(super) fn render_mermaid_header_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    diagram_type: MermaidDiagramType,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: Default::default(),
        text: Arc::<str>::from(format!("Mermaid · {}", diagram_type.display_name())),
        style: Some(TextStyle {
            font: FontId::monospace(),
            size: theme.metric_required("metric.font.mono_size"),
            weight: FontWeight::SEMIBOLD,
            slant: Default::default(),
            line_height: Some(theme.metric_required("metric.font.mono_line_height")),
            letter_spacing_em: None,
        }),
        color: Some(theme.color_required("muted-foreground")),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    })
}
