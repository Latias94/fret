use fret_ui::element::AnyElement;
use fret_ui::element::LayoutQueryRegionProps;
use fret_ui::{ElementContext, Invalidation, UiHost};

use super::ShadcnResolver;

impl ShadcnResolver {
    pub(super) fn render_responsive_grid<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let gap = Self::parse_space(resolved_props.get("gap")).unwrap_or(fret_ui_kit::Space::N2);
        let fill_last_row = resolved_props
            .get("fillLastRow")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let columns_spec = Self::parse_columns_spec(resolved_props.get("columns"));
        let query = resolved_props
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("container");

        match (query, columns_spec) {
            (_, ColumnsSpec::Fixed(cols)) => {
                Self::build_grid_rows(cx, cols, gap, fill_last_row, children)
            }
            ("viewport", ColumnsSpec::Breakpoints(bp)) => {
                let mut breakpoints: Vec<(fret_core::Px, u8)> = Vec::new();
                use fret_ui_kit::declarative::viewport_queries::tailwind as tw;
                if let Some(v) = bp.sm {
                    breakpoints.push((tw::SM, v.max(1)));
                }
                if let Some(v) = bp.md {
                    breakpoints.push((tw::MD, v.max(1)));
                }
                if let Some(v) = bp.lg {
                    breakpoints.push((tw::LG, v.max(1)));
                }
                if let Some(v) = bp.xl {
                    breakpoints.push((tw::XL, v.max(1)));
                }
                if let Some(v) = bp.xxl {
                    breakpoints.push((tw::XXL, v.max(1)));
                }
                let base = bp.base.unwrap_or(1).max(1);
                let cols = fret_ui_kit::declarative::viewport_breakpoints(
                    cx,
                    Invalidation::Layout,
                    base,
                    &breakpoints,
                    fret_ui_kit::declarative::viewport_queries::ViewportQueryHysteresis::default(),
                );
                Self::build_grid_rows(cx, cols, gap, fill_last_row, children)
            }
            (_, ColumnsSpec::Breakpoints(bp)) => {
                fret_ui_kit::declarative::container_query_region_with_id(
                    cx,
                    "genui.responsive_grid",
                    LayoutQueryRegionProps::default(),
                    |cx, region| {
                        let mut breakpoints: Vec<(fret_core::Px, u8)> = Vec::new();
                        use fret_ui_kit::declarative::container_queries::tailwind as tw;
                        if let Some(v) = bp.sm {
                            breakpoints.push((tw::SM, v.max(1)));
                        }
                        if let Some(v) = bp.md {
                            breakpoints.push((tw::MD, v.max(1)));
                        }
                        if let Some(v) = bp.lg {
                            breakpoints.push((tw::LG, v.max(1)));
                        }
                        if let Some(v) = bp.xl {
                            breakpoints.push((tw::XL, v.max(1)));
                        }
                        if let Some(v) = bp.xxl {
                            breakpoints.push((tw::XXL, v.max(1)));
                        }
                        let base = bp.base.unwrap_or(1).max(1);
                        let cols = fret_ui_kit::declarative::container_breakpoints(
                            cx,
                            region,
                            Invalidation::Layout,
                            base,
                            &breakpoints,
                            fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
                        );
                        [Self::build_grid_rows(
                            cx,
                            cols,
                            gap,
                            fill_last_row,
                            children,
                        )]
                    },
                )
            }
        }
    }

    pub(super) fn render_responsive_stack<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let gap = Self::parse_space(resolved_props.get("gap")).unwrap_or(fret_ui_kit::Space::N2);
        let query = resolved_props
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("container");

        let direction_spec = Self::parse_direction_spec(resolved_props.get("direction"));

        match (query, direction_spec) {
            (_, DirectionSpec::Fixed(dir)) => Self::build_stack(cx, dir, gap, children),
            ("viewport", DirectionSpec::Breakpoints(bp)) => {
                let mut breakpoints: Vec<(fret_core::Px, StackDirection)> = Vec::new();
                use fret_ui_kit::declarative::viewport_queries::tailwind as tw;
                if let Some(v) = bp.sm {
                    breakpoints.push((tw::SM, v));
                }
                if let Some(v) = bp.md {
                    breakpoints.push((tw::MD, v));
                }
                if let Some(v) = bp.lg {
                    breakpoints.push((tw::LG, v));
                }
                if let Some(v) = bp.xl {
                    breakpoints.push((tw::XL, v));
                }
                if let Some(v) = bp.xxl {
                    breakpoints.push((tw::XXL, v));
                }

                let base = bp.base.unwrap_or(StackDirection::Vertical);
                let dir = fret_ui_kit::declarative::viewport_breakpoints(
                    cx,
                    Invalidation::Layout,
                    base,
                    &breakpoints,
                    fret_ui_kit::declarative::viewport_queries::ViewportQueryHysteresis::default(),
                );
                Self::build_stack(cx, dir, gap, children)
            }
            (_, DirectionSpec::Breakpoints(bp)) => {
                fret_ui_kit::declarative::container_query_region_with_id(
                    cx,
                    "genui.responsive_stack",
                    LayoutQueryRegionProps::default(),
                    |cx, region| {
                        let mut breakpoints: Vec<(fret_core::Px, StackDirection)> = Vec::new();
                        use fret_ui_kit::declarative::container_queries::tailwind as tw;
                        if let Some(v) = bp.sm {
                            breakpoints.push((tw::SM, v));
                        }
                        if let Some(v) = bp.md {
                            breakpoints.push((tw::MD, v));
                        }
                        if let Some(v) = bp.lg {
                            breakpoints.push((tw::LG, v));
                        }
                        if let Some(v) = bp.xl {
                            breakpoints.push((tw::XL, v));
                        }
                        if let Some(v) = bp.xxl {
                            breakpoints.push((tw::XXL, v));
                        }
                        let base = bp.base.unwrap_or(StackDirection::Vertical);
                        let dir = fret_ui_kit::declarative::container_breakpoints(
                            cx,
                            region,
                            Invalidation::Layout,
                            base,
                            &breakpoints,
                            fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
                        );
                        [Self::build_stack(cx, dir, gap, children)]
                    },
                )
            }
        }
    }

    fn parse_columns_spec(v: Option<&serde_json::Value>) -> ColumnsSpec {
        let Some(v) = v else {
            return ColumnsSpec::Fixed(1);
        };
        if let Some(i) = v.as_i64().and_then(|i| u8::try_from(i).ok()) {
            return ColumnsSpec::Fixed(i.max(1));
        }
        if let Some(u) = v.as_u64().and_then(|i| u8::try_from(i).ok()) {
            return ColumnsSpec::Fixed(u.max(1));
        }
        let Some(obj) = v.as_object() else {
            return ColumnsSpec::Fixed(1);
        };
        let mut bp = BreakpointColumns::default();
        bp.base = obj
            .get("base")
            .and_then(|v| v.as_u64())
            .and_then(|i| u8::try_from(i).ok())
            .or(bp.base);
        bp.sm = obj
            .get("sm")
            .and_then(|v| v.as_u64())
            .and_then(|i| u8::try_from(i).ok());
        bp.md = obj
            .get("md")
            .and_then(|v| v.as_u64())
            .and_then(|i| u8::try_from(i).ok());
        bp.lg = obj
            .get("lg")
            .and_then(|v| v.as_u64())
            .and_then(|i| u8::try_from(i).ok());
        bp.xl = obj
            .get("xl")
            .and_then(|v| v.as_u64())
            .and_then(|i| u8::try_from(i).ok());
        bp.xxl = obj
            .get("xxl")
            .and_then(|v| v.as_u64())
            .and_then(|i| u8::try_from(i).ok());
        ColumnsSpec::Breakpoints(bp)
    }

    fn parse_stack_direction(v: Option<&serde_json::Value>) -> Option<StackDirection> {
        let s = v?.as_str()?;
        Some(match s {
            "vertical" => StackDirection::Vertical,
            "horizontal" => StackDirection::Horizontal,
            _ => return None,
        })
    }

    fn parse_direction_spec(v: Option<&serde_json::Value>) -> DirectionSpec {
        if let Some(dir) = Self::parse_stack_direction(v) {
            return DirectionSpec::Fixed(dir);
        }
        let Some(obj) = v.and_then(|v| v.as_object()) else {
            return DirectionSpec::Fixed(StackDirection::Vertical);
        };

        let mut bp = BreakpointDirection::default();
        bp.base = Self::parse_stack_direction(obj.get("base")).or(bp.base);
        bp.sm = Self::parse_stack_direction(obj.get("sm"));
        bp.md = Self::parse_stack_direction(obj.get("md"));
        bp.lg = Self::parse_stack_direction(obj.get("lg"));
        bp.xl = Self::parse_stack_direction(obj.get("xl"));
        bp.xxl = Self::parse_stack_direction(obj.get("xxl"));
        DirectionSpec::Breakpoints(bp)
    }

    fn build_grid_rows(
        cx: &mut ElementContext<'_, impl UiHost>,
        columns: u8,
        gap: fret_ui_kit::Space,
        fill_last_row: bool,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let cols = columns.max(1).min(12) as usize;
        let mut rows: Vec<Vec<AnyElement>> = Vec::new();
        let mut cur: Vec<AnyElement> = Vec::new();
        for child in children {
            cur.push(child);
            if cur.len() >= cols {
                rows.push(std::mem::take(&mut cur));
            }
        }
        if !cur.is_empty() {
            rows.push(cur);
        }

        fret_ui_kit::ui::v_flex(move |cx| {
            let mut out: Vec<AnyElement> = Vec::with_capacity(rows.len());
            for row_children in rows {
                let mut cells: Vec<AnyElement> = Vec::with_capacity(cols);
                for child in row_children {
                    let cell = fret_ui_kit::ui::container(move |_cx| [child])
                        .flex_1()
                        .min_w_0()
                        .into_element(cx);
                    cells.push(cell);
                }
                if fill_last_row && cells.len() < cols {
                    let missing = cols - cells.len();
                    for _ in 0..missing {
                        cells.push(
                            fret_ui_kit::ui::container(|_cx| Vec::<AnyElement>::new())
                                .flex_1()
                                .min_w_0()
                                .into_element(cx),
                        );
                    }
                }

                out.push(
                    fret_ui_kit::ui::h_flex(move |_cx| cells)
                        .gap(gap)
                        .items_start()
                        .w_full()
                        .into_element(cx),
                );
            }
            out
        })
        .gap(gap)
        .items_start()
        .w_full()
        .into_element(cx)
    }

    fn build_stack(
        cx: &mut ElementContext<'_, impl UiHost>,
        direction: StackDirection,
        gap: fret_ui_kit::Space,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        match direction {
            StackDirection::Vertical => fret_ui_kit::ui::v_flex(move |_cx| children)
                .gap(gap)
                .items_start()
                .w_full()
                .into_element(cx),
            StackDirection::Horizontal => fret_ui_kit::ui::h_flex(move |_cx| children)
                .gap(gap)
                .items_center()
                .w_full()
                .into_element(cx),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ColumnsSpec {
    Fixed(u8),
    Breakpoints(BreakpointColumns),
}

#[derive(Debug, Default, Clone, Copy)]
struct BreakpointColumns {
    base: Option<u8>,
    sm: Option<u8>,
    md: Option<u8>,
    lg: Option<u8>,
    xl: Option<u8>,
    xxl: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackDirection {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy)]
enum DirectionSpec {
    Fixed(StackDirection),
    Breakpoints(BreakpointDirection),
}

#[derive(Debug, Default, Clone, Copy)]
struct BreakpointDirection {
    base: Option<StackDirection>,
    sm: Option<StackDirection>,
    md: Option<StackDirection>,
    lg: Option<StackDirection>,
    xl: Option<StackDirection>,
    xxl: Option<StackDirection>,
}
