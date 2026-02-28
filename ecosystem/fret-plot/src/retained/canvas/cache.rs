//! Text and legend caching for retained plot canvases.

use super::*;

fn hash_value<T: Hash>(value: &T) -> u64 {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone)]
pub(super) struct LegendEntry {
    pub(super) id: SeriesId,
    pub(super) text: PreparedText,
}

impl<L: PlotLayer + 'static> PlotCanvas<L> {
    pub(super) fn clear_axis_label_cache(&mut self, services: &mut dyn UiServices) {
        self.axis_text_cache.clear(services);
        self.axis_labels_x.clear();
        self.axis_labels_y.clear();
        self.axis_labels_y2.clear();
        self.axis_labels_y3.clear();
        self.axis_labels_y4.clear();
        self.axis_ticks_x.clear();
        self.axis_ticks_y.clear();
        self.axis_ticks_y2.clear();
        self.axis_ticks_y3.clear();
        self.axis_ticks_y4.clear();
        self.axis_label_key = None;
    }

    pub(super) fn clear_legend_cache(&mut self, services: &mut dyn UiServices) {
        self.legend_text_cache.clear(services);
        self.legend_entries.clear();
        self.legend_key = None;
    }

    pub(super) fn legend_layout(&self, layout: PlotLayout) -> Option<(Rect, Vec<Rect>)> {
        if self.legend_entries.len() <= 1 {
            return None;
        }
        if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
            return None;
        }

        let margin = Px(8.0);
        let pad = Px(8.0);
        let gap = Px(8.0);
        let row_gap = Px(4.0);
        let swatch_w = Px(14.0);
        let swatch_h = Px(self.style.stroke_width.0.clamp(2.0, 6.0));

        let mut max_label_w = 0.0f32;
        let mut total_h = 0.0f32;
        for (i, entry) in self.legend_entries.iter().enumerate() {
            if i > 0 {
                total_h += row_gap.0;
            }
            max_label_w = max_label_w.max(entry.text.metrics.size.width.0);
            total_h += entry.text.metrics.size.height.0.max(swatch_h.0);
        }

        let legend_w = Px(pad.0 * 2.0 + swatch_w.0 + gap.0 + max_label_w);
        let legend_h = Px(pad.0 * 2.0 + total_h);

        let mut x = Px(layout.plot.origin.x.0 + layout.plot.size.width.0 - legend_w.0 - margin.0);
        let mut y = Px(layout.plot.origin.y.0 + margin.0);
        x = Px(x.0.max(layout.plot.origin.x.0));
        y = Px(y.0.max(layout.plot.origin.y.0));

        let rect = Rect::new(Point::new(x, y), Size::new(legend_w, legend_h));

        let mut rows: Vec<Rect> = Vec::with_capacity(self.legend_entries.len());
        let mut cursor_y = rect.origin.y.0 + pad.0;
        for (i, entry) in self.legend_entries.iter().enumerate() {
            let row_h = entry.text.metrics.size.height.0.max(swatch_h.0);
            rows.push(Rect::new(
                Point::new(rect.origin.x, Px(cursor_y)),
                Size::new(rect.size.width, Px(row_h)),
            ));
            cursor_y += row_h;
            if i + 1 < self.legend_entries.len() {
                cursor_y += row_gap.0;
            }
        }

        Some((rect, rows))
    }

    pub(super) fn legend_swatch_column(rect: Rect) -> Rect {
        let pad = Px(8.0);
        let swatch_w = Px(14.0);
        Rect::new(
            Point::new(Px(rect.origin.x.0 + pad.0), rect.origin.y),
            Size::new(swatch_w, rect.size.height),
        )
    }

    pub(super) fn hash_u64(mut state: u64, v: u64) -> u64 {
        state ^= v
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state
    }

    pub(super) fn hash_f32_bits(state: u64, v: f32) -> u64 {
        Self::hash_u64(state, u64::from(v.to_bits()))
    }

    pub(super) fn hash_f64_bits(state: u64, v: f64) -> u64 {
        Self::hash_u64(state, v.to_bits())
    }

    pub(super) fn read_data_bounds<H: UiHost>(&self, app: &mut H) -> DataRect {
        let data_bounds = self
            .model
            .read(app, |_app, m| L::data_bounds(m))
            .unwrap_or(DataRect {
                x_min: 0.0,
                x_max: 1.0,
                y_min: 0.0,
                y_max: 1.0,
            });
        sanitize_data_rect(data_bounds)
    }

    pub(super) fn read_data_bounds_y2<H: UiHost>(&self, app: &mut H) -> Option<DataRect> {
        let bounds = self
            .model
            .read(app, |_app, m| L::data_bounds_y2(m))
            .ok()
            .flatten()?;
        Some(sanitize_data_rect(bounds))
    }

    pub(super) fn read_data_bounds_y3<H: UiHost>(&self, app: &mut H) -> Option<DataRect> {
        let bounds = self
            .model
            .read(app, |_app, m| L::data_bounds_y3(m))
            .ok()
            .flatten()?;
        Some(sanitize_data_rect(bounds))
    }

    pub(super) fn read_data_bounds_y4<H: UiHost>(&self, app: &mut H) -> Option<DataRect> {
        let bounds = self
            .model
            .read(app, |_app, m| L::data_bounds_y4(m))
            .ok()
            .flatten()?;
        Some(sanitize_data_rect(bounds))
    }

    pub(super) fn text_style_key(style: &TextStyle) -> u64 {
        let mut state = 0u64;
        state = Self::hash_u64(state, hash_value(&style.font));
        state = Self::hash_u64(state, u64::from(style.weight.0));
        state = Self::hash_f32_bits(state, style.size.0);
        state = Self::hash_u64(
            state,
            u64::from(style.line_height.map(|v| v.0.to_bits()).unwrap_or(0)),
        );
        state = Self::hash_u64(
            state,
            u64::from(style.letter_spacing_em.map(|v| v.to_bits()).unwrap_or(0)),
        );
        state
    }

    pub(super) fn prepare_text(
        &mut self,
        services: &mut dyn UiServices,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> PreparedText {
        let mut state = 0u64;
        for b in text.as_bytes() {
            state = Self::hash_u64(state, u64::from(*b));
        }
        state = Self::hash_u64(state, Self::text_style_key(style));
        state = Self::hash_u64(state, u64::from(constraints.scale_factor.to_bits()));
        state = Self::hash_u64(
            state,
            u64::from(constraints.max_width.map(|v| v.0.to_bits()).unwrap_or(0)),
        );
        state = Self::hash_u64(state, hash_value(&constraints.wrap));
        state = Self::hash_u64(state, hash_value(&constraints.overflow));

        let (blob, metrics) = services.text().prepare_str(text, style, constraints);
        PreparedText {
            blob,
            metrics,
            key: state,
        }
    }

    pub(super) fn prepare_axis_text(
        &mut self,
        services: &mut dyn UiServices,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> PreparedText {
        self.axis_text_cache
            .prepare(services, text, style, constraints)
    }

    pub(super) fn prepare_legend_text(
        &mut self,
        services: &mut dyn UiServices,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> PreparedText {
        self.legend_text_cache
            .prepare(services, text, style, constraints)
    }

    pub(super) fn rebuild_axis_labels_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        layout: PlotLayout,
        view_bounds: DataRect,
        view_bounds_y2: Option<DataRect>,
        view_bounds_y3: Option<DataRect>,
        view_bounds_y4: Option<DataRect>,
        theme_revision: u64,
        font_stack_key: u64,
    ) -> bool {
        let scale_bits = cx.scale_factor.to_bits();

        let mut key = 0u64;
        key = Self::hash_u64(key, u64::from(scale_bits));
        key = Self::hash_u64(key, theme_revision);
        key = Self::hash_u64(key, font_stack_key);
        key = Self::hash_f32_bits(key, layout.plot.size.width.0);
        key = Self::hash_f32_bits(key, layout.plot.size.height.0);
        key = Self::hash_f32_bits(key, layout.y_axis_left.size.width.0);
        key = Self::hash_f32_bits(key, layout.y_axis_right.size.width.0);
        key = Self::hash_f32_bits(key, layout.y_axis_right2.size.width.0);
        key = Self::hash_f32_bits(key, layout.y_axis_right3.size.width.0);
        key = Self::hash_f32_bits(key, layout.x_axis.size.height.0);
        key = Self::hash_f64_bits(key, view_bounds.x_min);
        key = Self::hash_f64_bits(key, view_bounds.x_max);
        key = Self::hash_f64_bits(key, view_bounds.y_min);
        key = Self::hash_f64_bits(key, view_bounds.y_max);
        if let Some(y2) = view_bounds_y2 {
            key = Self::hash_f64_bits(key, y2.y_min);
            key = Self::hash_f64_bits(key, y2.y_max);
        } else {
            key = Self::hash_u64(key, 0);
        }
        if let Some(y3) = view_bounds_y3 {
            key = Self::hash_f64_bits(key, y3.y_min);
            key = Self::hash_f64_bits(key, y3.y_max);
        } else {
            key = Self::hash_u64(key, 0);
        }
        if let Some(y4) = view_bounds_y4 {
            key = Self::hash_f64_bits(key, y4.y_min);
            key = Self::hash_f64_bits(key, y4.y_max);
        } else {
            key = Self::hash_u64(key, 0);
        }
        key = Self::hash_u64(key, u64::from(self.style.axis_gap.0.to_bits()));
        key = Self::hash_u64(key, u64::from(self.style.tick_count as u32));
        key = Self::hash_u64(key, self.x_axis_ticks.key());
        key = Self::hash_u64(key, self.y_axis_ticks.key());
        key = Self::hash_u64(key, self.y2_axis_ticks.key());
        key = Self::hash_u64(key, self.y3_axis_ticks.key());
        key = Self::hash_u64(key, self.y4_axis_ticks.key());
        key = Self::hash_u64(key, self.x_scale.key());
        key = Self::hash_u64(key, self.y_scale.key());
        key = Self::hash_u64(key, self.y2_scale.key());
        key = Self::hash_u64(key, self.y3_scale.key());
        key = Self::hash_u64(key, self.y4_scale.key());
        key = Self::hash_u64(key, self.x_axis_labels.key());
        key = Self::hash_u64(key, self.y_axis_labels.key());
        key = Self::hash_u64(key, self.y2_axis_labels.key());
        key = Self::hash_u64(key, self.y3_axis_labels.key());
        key = Self::hash_u64(key, self.y4_axis_labels.key());
        key = Self::hash_u64(key, u64::from(self.show_y2_axis));
        key = Self::hash_u64(key, u64::from(self.show_y3_axis));
        key = Self::hash_u64(key, u64::from(self.show_y4_axis));

        if self.axis_label_key == Some(key) {
            return false;
        }

        self.clear_axis_label_cache(cx.services);

        let font_size = cx
            .theme()
            .metric_by_key("font.size")
            .unwrap_or(cx.theme().metrics.font_size);
        let style = TextStyle {
            font: FontId::default(),
            size: Px((font_size.0 * 0.90).max(10.0)),
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: None,
            line_height_em: None,
            line_height_policy: Default::default(),
            letter_spacing_em: None,
            features: Vec::new(),
            axes: Vec::new(),
            vertical_placement: Default::default(),
            leading_distribution: Default::default(),
            strut_style: Default::default(),
        };

        let axis_span = |min: f64, max: f64, scale: AxisScale| -> f64 {
            let span_data = (max - min).abs();
            scale
                .to_axis(min)
                .zip(scale.to_axis(max))
                .map(|(a, b)| (b - a).abs())
                .unwrap_or(span_data)
        };

        let x_span = axis_span(view_bounds.x_min, view_bounds.x_max, self.x_scale);
        let y_span = axis_span(view_bounds.y_min, view_bounds.y_max, self.y_scale);
        let y2_span = view_bounds_y2
            .map(|b| axis_span(b.y_min, b.y_max, self.y2_scale))
            .unwrap_or(0.0);
        let y3_span = view_bounds_y3
            .map(|b| axis_span(b.y_min, b.y_max, self.y3_scale))
            .unwrap_or(0.0);
        let y4_span = view_bounds_y4
            .map(|b| axis_span(b.y_min, b.y_max, self.y4_scale))
            .unwrap_or(0.0);

        let constraints_x = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        let constraints_y = TextConstraints {
            max_width: Some(layout.y_axis_left.size.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        let constraints_y2 = TextConstraints {
            max_width: Some(layout.y_axis_right.size.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        let constraints_y3 = TextConstraints {
            max_width: Some(layout.y_axis_right2.size.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };
        let constraints_y4 = TextConstraints {
            max_width: Some(layout.y_axis_right3.size.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };

        let plot_w = layout.plot.size.width.0.max(0.0);
        let plot_h = layout.plot.size.height.0.max(0.0);

        let estimate_ticks = |span_px: f32, target_spacing_px: f32| -> usize {
            if !span_px.is_finite() || span_px <= 0.0 {
                return 2;
            }
            let spacing = target_spacing_px.max(1.0);
            ((span_px / spacing).floor() as usize)
                .saturating_add(1)
                .max(2)
        };

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
        let transform_y1 = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
        };

        let min_ticks = self.style.tick_count.max(2).min(128);
        let mut x_tick_count = estimate_ticks(plot_w, 70.0).max(min_ticks).min(128);
        let mut y_tick_count = estimate_ticks(plot_h, 28.0).max(min_ticks).min(128);

        let mut x_ticks: Vec<f64> = Vec::new();
        let mut y_ticks: Vec<f64> = Vec::new();

        // X axis: reduce ticks until formatted labels fit horizontally.
        for _ in 0..8 {
            x_ticks = axis_ticks_scaled(
                view_bounds.x_min,
                view_bounds.x_max,
                x_tick_count,
                self.x_axis_ticks,
                self.x_scale,
            );
            if x_ticks.len() <= 1 {
                break;
            }

            let mut max_w = 0.0f32;
            for v in &x_ticks {
                let text = self.x_axis_labels.format(*v, x_span);
                let prepared = self.prepare_text(cx.services, &text, &style, constraints_x);
                max_w = max_w.max(prepared.metrics.size.width.0);
                cx.services.text().release(prepared.blob);
            }

            let mut min_spacing_px = plot_w;
            let mut prev: Option<f32> = None;
            for v in &x_ticks {
                let Some(px) = transform_y1.data_x_to_px(*v) else {
                    continue;
                };
                let x = px.0;
                if let Some(prev) = prev {
                    let dx = (x - prev).abs();
                    if dx.is_finite() && dx > 0.0 {
                        min_spacing_px = min_spacing_px.min(dx);
                    }
                }
                prev = Some(x);
            }
            let spacing_px = min_spacing_px;
            let needed = max_w + 8.0;
            if spacing_px.is_finite() && needed.is_finite() && spacing_px >= needed {
                break;
            }

            let suggested = ((plot_w / needed.max(1.0)).floor() as usize)
                .saturating_add(1)
                .max(2);
            let next = x_tick_count.min(suggested);
            x_tick_count = if next == x_tick_count {
                x_tick_count.saturating_sub(1).max(2)
            } else {
                next
            };
            if x_tick_count <= 2 {
                break;
            }
        }

        // Y axis: reduce ticks until labels fit vertically (avoid overlap).
        for _ in 0..8 {
            y_ticks = axis_ticks_scaled(
                view_bounds.y_min,
                view_bounds.y_max,
                y_tick_count,
                self.y_axis_ticks,
                self.y_scale,
            );
            if y_ticks.len() <= 1 {
                break;
            }

            let mut max_h = 0.0f32;
            for v in &y_ticks {
                let text = self.y_axis_labels.format(*v, y_span);
                let prepared = self.prepare_text(cx.services, &text, &style, constraints_y);
                max_h = max_h.max(prepared.metrics.size.height.0);
                cx.services.text().release(prepared.blob);
            }

            let mut min_spacing_px = plot_h;
            let mut prev: Option<f32> = None;
            for v in &y_ticks {
                let Some(px) = transform_y1.data_y_to_px(*v) else {
                    continue;
                };
                let y = px.0;
                if let Some(prev) = prev {
                    let dy = (y - prev).abs();
                    if dy.is_finite() && dy > 0.0 {
                        min_spacing_px = min_spacing_px.min(dy);
                    }
                }
                prev = Some(y);
            }
            let spacing_px = min_spacing_px;
            let needed = max_h + 4.0;
            if spacing_px.is_finite() && needed.is_finite() && spacing_px >= needed {
                break;
            }

            let suggested = ((plot_h / needed.max(1.0)).floor() as usize)
                .saturating_add(1)
                .max(2);
            let next = y_tick_count.min(suggested);
            y_tick_count = if next == y_tick_count {
                y_tick_count.saturating_sub(1).max(2)
            } else {
                next
            };
            if y_tick_count <= 2 {
                break;
            }
        }

        self.axis_ticks_x = x_ticks.clone();
        self.axis_ticks_y = y_ticks.clone();
        self.axis_ticks_y2 = if self.show_y2_axis {
            if let Some(y2_bounds) = view_bounds_y2 {
                axis_ticks_scaled(
                    y2_bounds.y_min,
                    y2_bounds.y_max,
                    y_tick_count,
                    self.y2_axis_ticks,
                    self.y2_scale,
                )
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        self.axis_ticks_y3 = if self.show_y3_axis {
            if let Some(y3_bounds) = view_bounds_y3 {
                axis_ticks_scaled(
                    y3_bounds.y_min,
                    y3_bounds.y_max,
                    y_tick_count,
                    self.y3_axis_ticks,
                    self.y3_scale,
                )
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        self.axis_ticks_y4 = if self.show_y4_axis {
            if let Some(y4_bounds) = view_bounds_y4 {
                axis_ticks_scaled(
                    y4_bounds.y_min,
                    y4_bounds.y_max,
                    y_tick_count,
                    self.y4_axis_ticks,
                    self.y4_scale,
                )
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        for v in x_ticks {
            let text = if self.x_scale == AxisScale::Log10 && self.x_axis_labels.is_number_auto() {
                log10_tick_label_or_empty(v)
            } else {
                self.x_axis_labels.format(v, x_span)
            };
            let prepared = self.prepare_axis_text(cx.services, &text, &style, constraints_x);
            self.axis_labels_x.push(prepared);
        }

        for v in y_ticks {
            let text = if self.y_scale == AxisScale::Log10 && self.y_axis_labels.is_number_auto() {
                log10_tick_label_or_empty(v)
            } else {
                self.y_axis_labels.format(v, y_span)
            };
            let prepared = self.prepare_axis_text(cx.services, &text, &style, constraints_y);
            self.axis_labels_y.push(prepared);
        }

        if self.show_y2_axis {
            let y2_ticks = self.axis_ticks_y2.clone();
            for v in y2_ticks {
                let text =
                    if self.y2_scale == AxisScale::Log10 && self.y2_axis_labels.is_number_auto() {
                        log10_tick_label_or_empty(v)
                    } else {
                        self.y2_axis_labels.format(v, y2_span)
                    };
                let prepared = self.prepare_axis_text(cx.services, &text, &style, constraints_y2);
                self.axis_labels_y2.push(prepared);
            }
        }

        if self.show_y3_axis {
            let y3_ticks = self.axis_ticks_y3.clone();
            for v in y3_ticks {
                let text =
                    if self.y3_scale == AxisScale::Log10 && self.y3_axis_labels.is_number_auto() {
                        log10_tick_label_or_empty(v)
                    } else {
                        self.y3_axis_labels.format(v, y3_span)
                    };
                let prepared = self.prepare_axis_text(cx.services, &text, &style, constraints_y3);
                self.axis_labels_y3.push(prepared);
            }
        }

        if self.show_y4_axis {
            let y4_ticks = self.axis_ticks_y4.clone();
            for v in y4_ticks {
                let text =
                    if self.y4_scale == AxisScale::Log10 && self.y4_axis_labels.is_number_auto() {
                        log10_tick_label_or_empty(v)
                    } else {
                        self.y4_axis_labels.format(v, y4_span)
                    };
                let prepared = self.prepare_axis_text(cx.services, &text, &style, constraints_y4);
                self.axis_labels_y4.push(prepared);
            }
        }

        self.axis_label_key = Some(key);

        // Axis thickness auto-fit: expand the axis gaps to fit the largest tick label.
        // This mirrors egui_plot's approach where axis thickness is cached and increased as needed.
        let mut changed = false;
        let min_axis = self.style.axis_gap.0.max(0.0);

        let content_w = layout.plot.size.width.0
            + layout.y_axis_left.size.width.0
            + layout.y_axis_right.size.width.0
            + layout.y_axis_right2.size.width.0
            + layout.y_axis_right3.size.width.0;
        let content_h = layout.plot.size.height.0 + layout.x_axis.size.height.0;

        let max_y_label_w = self
            .axis_labels_y
            .iter()
            .map(|t| t.metrics.size.width.0)
            .fold(0.0f32, f32::max);
        let max_y2_label_w = self
            .axis_labels_y2
            .iter()
            .map(|t| t.metrics.size.width.0)
            .fold(0.0f32, f32::max);
        let max_y3_label_w = self
            .axis_labels_y3
            .iter()
            .map(|t| t.metrics.size.width.0)
            .fold(0.0f32, f32::max);
        let max_y4_label_w = self
            .axis_labels_y4
            .iter()
            .map(|t| t.metrics.size.width.0)
            .fold(0.0f32, f32::max);
        let max_x_label_h = self
            .axis_labels_x
            .iter()
            .map(|t| t.metrics.size.height.0)
            .fold(0.0f32, f32::max);

        let desired_y = (max_y_label_w + 8.0).max(min_axis);
        let desired_x = (max_x_label_h + 6.0).max(min_axis);
        let desired_y2 = (max_y2_label_w + 8.0).max(min_axis);
        let desired_y3 = (max_y3_label_w + 8.0).max(min_axis);
        let desired_y4 = (max_y4_label_w + 8.0).max(min_axis);

        // Prevent axes from consuming the majority of the plot area in degenerate cases.
        let cap_y = (content_w * 0.5).max(min_axis);
        let cap_x = (content_h * 0.5).max(min_axis);

        let next_y = desired_y.min(cap_y);
        let next_x = desired_x.min(cap_x);
        let next_y2 = desired_y2.min(cap_y);
        let next_y3 = desired_y3.min(cap_y);
        let next_y4 = desired_y4.min(cap_y);

        if next_y.is_finite() && next_y > self.y_axis_thickness.0 + 0.5 {
            self.y_axis_thickness = Px(next_y);
            changed = true;
        }
        if self.show_y2_axis && next_y2.is_finite() && next_y2 > self.y_axis_right_thickness.0 + 0.5
        {
            self.y_axis_right_thickness = Px(next_y2);
            changed = true;
        }
        if self.show_y3_axis
            && next_y3.is_finite()
            && next_y3 > self.y_axis_right2_thickness.0 + 0.5
        {
            self.y_axis_right2_thickness = Px(next_y3);
            changed = true;
        }
        if self.show_y4_axis
            && next_y4.is_finite()
            && next_y4 > self.y_axis_right3_thickness.0 + 0.5
        {
            self.y_axis_right3_thickness = Px(next_y4);
            changed = true;
        }
        if next_x.is_finite() && next_x > self.x_axis_thickness.0 + 0.5 {
            self.x_axis_thickness = Px(next_x);
            changed = true;
        }

        changed
    }

    pub(super) fn rebuild_legend_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        theme_revision: u64,
        font_stack_key: u64,
    ) {
        let series: Vec<SeriesMeta> = self
            .model
            .read(cx.app, |_app, m| L::series_meta(m))
            .unwrap_or_default();
        if let Some(hovered) = self.legend_hover
            && series.iter().all(|s| s.id != hovered)
        {
            self.legend_hover = None;
        }

        if series.len() <= 1 {
            if self.legend_key.is_some() {
                self.clear_legend_cache(cx.services);
            }
            return;
        }

        let font_size = cx
            .theme()
            .metric_by_key("font.size")
            .unwrap_or(cx.theme().metrics.font_size);
        let style = TextStyle {
            font: FontId::default(),
            size: Px((font_size.0 * 0.85).max(10.0)),
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: None,
            line_height_em: None,
            line_height_policy: Default::default(),
            letter_spacing_em: None,
            features: Vec::new(),
            axes: Vec::new(),
            vertical_placement: Default::default(),
            leading_distribution: Default::default(),
            strut_style: Default::default(),
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };

        let mut key = 0u64;
        key = Self::hash_u64(key, theme_revision);
        key = Self::hash_u64(key, font_stack_key);
        key = Self::hash_u64(key, u64::from(cx.scale_factor.to_bits()));
        key = Self::hash_u64(key, u64::from(series.len() as u32));
        key = Self::hash_u64(key, Self::text_style_key(&style));
        for s in &series {
            key = Self::hash_u64(key, s.id.0);
            for b in s.label.as_bytes() {
                key = Self::hash_u64(key, u64::from(*b));
            }
        }

        if self.legend_key == Some(key) {
            return;
        }

        self.clear_legend_cache(cx.services);

        self.legend_entries = Vec::with_capacity(series.len());
        for s in series {
            let text = s.label.to_string();
            let prepared = self.prepare_legend_text(cx.services, &text, &style, constraints);
            self.legend_entries.push(LegendEntry {
                id: s.id,
                text: prepared,
            });
        }

        self.legend_key = Some(key);
    }
}
