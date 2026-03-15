use super::TextSystem;
use fret_core::{AttributedText, TextConstraints, TextMetrics, TextStyle};

impl TextSystem {
    pub fn measure(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        return self.layout_cache.measure.measure_plain(
            &mut self.parley_shaper,
            text,
            style,
            constraints,
            self.font_runtime.font_stack_key,
        );

        #[cfg(any())]
        {
            const MEASURE_CACHE_PER_BUCKET_LIMIT: usize = 256;
            const MEASURE_CACHE_PER_BUCKET_LIMIT_WRAP_NONE: usize = 2048;

            let mut normalized_constraints = constraints;
            if normalized_constraints.wrap == TextWrap::None {
                normalized_constraints.max_width = None;
            }

            let key = TextMeasureKey::new(style, normalized_constraints, self.font_stack_key);
            let text_hash = hash_text(text);
            if let Some(bucket) = self.measure_cache.get_mut(&key)
                && let Some(idx) = bucket.iter().position(|e| {
                    e.text_hash == text_hash && e.spans_hash == 0 && e.text.as_ref() == text
                })
                && let Some(hit) = bucket.remove(idx)
            {
                let mut metrics = hit.metrics;
                bucket.push_back(hit);
                if constraints.wrap == TextWrap::None
                    && constraints.overflow == TextOverflow::Ellipsis
                    && let Some(max_width) = constraints.max_width
                {
                    metrics.size.width = max_width;
                }
                return metrics;
            }

            let scale = effective_text_scale_factor(constraints.scale_factor);
            let allow_fast_wrap_measure = constraints.scale_factor.is_finite()
                && constraints.scale_factor.fract().abs() <= 1e-4;
            let max_width_for_fast = match constraints {
                TextConstraints {
                    max_width: Some(max_width),
                    wrap: TextWrap::Word | TextWrap::WordBreak | TextWrap::Grapheme,
                    overflow: TextOverflow::Clip,
                    ..
                } if allow_fast_wrap_measure && !text.contains('\n') => Some(max_width),
                _ => None,
            };

            let metrics = if let Some(max_width) = max_width_for_fast {
                let allow_shaping_cache =
                    text.len() >= fret_render_text::measure_shaping_cache_min_text_len_bytes();

                let shaping_key = TextMeasureShapingKey {
                    text_hash,
                    text_len: text.len(),
                    spans_shaping_key: 0,
                    font: style.font.clone(),
                    font_stack_key: self.font_stack_key,
                    size_bits: style.size.0.to_bits(),
                    weight: style.weight.0,
                    slant: match style.slant {
                        TextSlant::Normal => 0,
                        TextSlant::Italic => 1,
                        TextSlant::Oblique => 2,
                    },
                    line_height_bits: style.line_height.map(|px| px.0.to_bits()),
                    letter_spacing_bits: style.letter_spacing_em.map(|v| v.to_bits()),
                    scale_bits: constraints.scale_factor.to_bits(),
                };

                let max_width_px = max_width.0 * scale;

                if allow_shaping_cache {
                    let (width_px, baseline_px, line_height_px, _clusters) = if let Some(hit) =
                        self.measure_shaping_cache.get(&shaping_key)
                        && hit.text.as_ref() == text
                        && hit.spans.is_none()
                    {
                        (
                            hit.width_px,
                            hit.baseline_px,
                            hit.line_height_px,
                            hit.clusters.clone(),
                        )
                    } else {
                        let line = self
                            .parley_shaper
                            .shape_single_line_metrics(TextInputRef::plain(text, style), scale);
                        let clusters: Arc<[fret_render_text::ShapedCluster]> =
                            Arc::from(line.clusters);

                        let existed = self
                            .measure_shaping_cache
                            .insert(
                                shaping_key.clone(),
                                TextMeasureShapingEntry {
                                    text: Arc::<str>::from(text),
                                    spans: None,
                                    width_px: line.width,
                                    baseline_px: line.baseline,
                                    line_height_px: line.line_height,
                                    clusters: clusters.clone(),
                                },
                            )
                            .is_some();
                        if !existed {
                            self.measure_shaping_fifo.push_back(shaping_key.clone());
                            let limit = fret_render_text::measure_shaping_cache_entries();
                            while self.measure_shaping_fifo.len() > limit {
                                let Some(evict) = self.measure_shaping_fifo.pop_front() else {
                                    break;
                                };
                                self.measure_shaping_cache.remove(&evict);
                            }
                        }

                        (line.width, line.baseline, line.line_height, clusters)
                    };

                    if width_px <= max_width_px + 0.5 {
                        metrics_for_uniform_lines(
                            width_px.max(0.0),
                            1,
                            baseline_px,
                            line_height_px,
                            scale,
                        )
                    } else {
                        let wrapped = fret_render_text::wrap_with_constraints_measure_only(
                            &mut self.parley_shaper,
                            TextInputRef::plain(text, style),
                            normalized_constraints,
                        );
                        metrics_from_wrapped_lines(&wrapped.lines, scale)
                    }
                } else {
                    let line = self
                        .parley_shaper
                        .shape_single_line_metrics(TextInputRef::plain(text, style), scale);
                    let width_px = line.width;
                    let baseline_px = line.baseline;
                    let line_height_px = line.line_height;
                    let _clusters = line.clusters;

                    if width_px <= max_width_px + 0.5 {
                        metrics_for_uniform_lines(
                            width_px.max(0.0),
                            1,
                            baseline_px,
                            line_height_px,
                            scale,
                        )
                    } else {
                        let wrapped = fret_render_text::wrap_with_constraints_measure_only(
                            &mut self.parley_shaper,
                            TextInputRef::plain(text, style),
                            normalized_constraints,
                        );
                        metrics_from_wrapped_lines(&wrapped.lines, scale)
                    }
                }
            } else {
                let wrapped = fret_render_text::wrap_with_constraints_measure_only(
                    &mut self.parley_shaper,
                    TextInputRef::plain(text, style),
                    normalized_constraints,
                );
                metrics_from_wrapped_lines(&wrapped.lines, scale)
            };

            let bucket = self.measure_cache.entry(key).or_default();
            bucket.push_back(TextMeasureEntry {
                text_hash,
                spans_hash: 0,
                text: Arc::<str>::from(text),
                spans: None,
                metrics,
            });
            let limit = match normalized_constraints.wrap {
                TextWrap::None => MEASURE_CACHE_PER_BUCKET_LIMIT_WRAP_NONE,
                TextWrap::Word | TextWrap::WordBreak | TextWrap::Grapheme => {
                    MEASURE_CACHE_PER_BUCKET_LIMIT
                }
            };
            while bucket.len() > limit {
                bucket.pop_front();
            }

            let mut metrics = metrics;
            if constraints.wrap == TextWrap::None
                && constraints.overflow == TextOverflow::Ellipsis
                && let Some(max_width) = constraints.max_width
            {
                metrics.size.width = max_width;
            }
            metrics
        }
    }

    pub fn measure_attributed(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        return self.layout_cache.measure.measure_attributed(
            &mut self.parley_shaper,
            rich,
            base_style,
            constraints,
            self.font_runtime.font_stack_key,
        );

        #[cfg(any())]
        {
            const MEASURE_CACHE_PER_BUCKET_LIMIT: usize = 256;
            const MEASURE_CACHE_PER_BUCKET_LIMIT_WRAP_NONE: usize = 2048;

            let mut normalized_constraints = constraints;
            if normalized_constraints.wrap == TextWrap::None {
                normalized_constraints.max_width = None;
            }

            let key = TextMeasureKey::new(base_style, normalized_constraints, self.font_stack_key);
            let text_hash = hash_text(rich.text.as_ref());
            let spans_hash = spans_shaping_fingerprint(rich.spans.as_ref());

            if let Some(bucket) = self.measure_cache.get_mut(&key)
                && let Some(idx) = bucket.iter().position(|e| {
                    e.text_hash == text_hash
                        && e.spans_hash == spans_hash
                        && e.text.as_ref() == rich.text.as_ref()
                        && e.spans.as_ref().is_some_and(|s| {
                            Arc::ptr_eq(s, &rich.spans) || s.as_ref() == rich.spans.as_ref()
                        })
                })
                && let Some(hit) = bucket.remove(idx)
            {
                let mut metrics = hit.metrics;
                bucket.push_back(hit);
                if constraints.wrap == TextWrap::None
                    && constraints.overflow == TextOverflow::Ellipsis
                    && let Some(max_width) = constraints.max_width
                {
                    metrics.size.width = max_width;
                }
                return metrics;
            }

            let scale = effective_text_scale_factor(constraints.scale_factor);
            let allow_fast_wrap_measure = constraints.scale_factor.is_finite()
                && constraints.scale_factor.fract().abs() <= 1e-4;
            let max_width_for_fast = match constraints {
                TextConstraints {
                    max_width: Some(max_width),
                    wrap: TextWrap::Word | TextWrap::WordBreak | TextWrap::Grapheme,
                    overflow: TextOverflow::Clip,
                    ..
                } if allow_fast_wrap_measure && !rich.text.as_ref().contains('\n') => {
                    Some(max_width)
                }
                _ => None,
            };

            let metrics = if let Some(max_width) = max_width_for_fast {
                let allow_shaping_cache =
                    rich.text.len() >= fret_render_text::measure_shaping_cache_min_text_len_bytes();

                let shaping_key = TextMeasureShapingKey {
                    text_hash,
                    text_len: rich.text.len(),
                    spans_shaping_key: spans_hash,
                    font: base_style.font.clone(),
                    font_stack_key: self.font_stack_key,
                    size_bits: base_style.size.0.to_bits(),
                    weight: base_style.weight.0,
                    slant: match base_style.slant {
                        TextSlant::Normal => 0,
                        TextSlant::Italic => 1,
                        TextSlant::Oblique => 2,
                    },
                    line_height_bits: base_style.line_height.map(|px| px.0.to_bits()),
                    letter_spacing_bits: base_style.letter_spacing_em.map(|v| v.to_bits()),
                    scale_bits: constraints.scale_factor.to_bits(),
                };

                let max_width_px = max_width.0 * scale;
                let text = rich.text.as_ref();

                if allow_shaping_cache {
                    let (width_px, baseline_px, line_height_px, _clusters) = if let Some(hit) =
                        self.measure_shaping_cache.get(&shaping_key)
                        && hit.text.as_ref() == rich.text.as_ref()
                        && hit.spans.as_ref().is_some_and(|s| {
                            Arc::ptr_eq(s, &rich.spans) || s.as_ref() == rich.spans.as_ref()
                        }) {
                        (
                            hit.width_px,
                            hit.baseline_px,
                            hit.line_height_px,
                            hit.clusters.clone(),
                        )
                    } else {
                        let line = self.parley_shaper.shape_single_line_metrics(
                            TextInputRef::Attributed {
                                text: rich.text.as_ref(),
                                base: base_style,
                                spans: rich.spans.as_ref(),
                            },
                            scale,
                        );
                        let clusters: Arc<[fret_render_text::ShapedCluster]> =
                            Arc::from(line.clusters);

                        let existed = self
                            .measure_shaping_cache
                            .insert(
                                shaping_key.clone(),
                                TextMeasureShapingEntry {
                                    text: rich.text.clone(),
                                    spans: Some(rich.spans.clone()),
                                    width_px: line.width,
                                    baseline_px: line.baseline,
                                    line_height_px: line.line_height,
                                    clusters: clusters.clone(),
                                },
                            )
                            .is_some();
                        if !existed {
                            self.measure_shaping_fifo.push_back(shaping_key.clone());
                            let limit = fret_render_text::measure_shaping_cache_entries();
                            while self.measure_shaping_fifo.len() > limit {
                                let Some(evict) = self.measure_shaping_fifo.pop_front() else {
                                    break;
                                };
                                self.measure_shaping_cache.remove(&evict);
                            }
                        }

                        (line.width, line.baseline, line.line_height, clusters)
                    };

                    if width_px <= max_width_px + 0.5 {
                        metrics_for_uniform_lines(
                            width_px.max(0.0),
                            1,
                            baseline_px,
                            line_height_px,
                            scale,
                        )
                    } else {
                        let wrapped = fret_render_text::wrap_with_constraints_measure_only(
                            &mut self.parley_shaper,
                            TextInputRef::Attributed {
                                text,
                                base: base_style,
                                spans: rich.spans.as_ref(),
                            },
                            normalized_constraints,
                        );
                        metrics_from_wrapped_lines(&wrapped.lines, scale)
                    }
                } else {
                    let line = self.parley_shaper.shape_single_line_metrics(
                        TextInputRef::Attributed {
                            text: rich.text.as_ref(),
                            base: base_style,
                            spans: rich.spans.as_ref(),
                        },
                        scale,
                    );
                    let width_px = line.width;
                    let baseline_px = line.baseline;
                    let line_height_px = line.line_height;
                    let _clusters = line.clusters;

                    if width_px <= max_width_px + 0.5 {
                        metrics_for_uniform_lines(
                            width_px.max(0.0),
                            1,
                            baseline_px,
                            line_height_px,
                            scale,
                        )
                    } else {
                        let wrapped = fret_render_text::wrap_with_constraints_measure_only(
                            &mut self.parley_shaper,
                            TextInputRef::Attributed {
                                text,
                                base: base_style,
                                spans: rich.spans.as_ref(),
                            },
                            normalized_constraints,
                        );
                        metrics_from_wrapped_lines(&wrapped.lines, scale)
                    }
                }
            } else {
                let text = rich.text.as_ref();
                let wrapped = fret_render_text::wrap_with_constraints_measure_only(
                    &mut self.parley_shaper,
                    TextInputRef::Attributed {
                        text,
                        base: base_style,
                        spans: rich.spans.as_ref(),
                    },
                    normalized_constraints,
                );
                metrics_from_wrapped_lines(&wrapped.lines, scale)
            };

            let bucket = self.measure_cache.entry(key).or_default();
            bucket.push_back(TextMeasureEntry {
                text_hash,
                spans_hash,
                text: rich.text.clone(),
                spans: Some(rich.spans.clone()),
                metrics,
            });
            let limit = match normalized_constraints.wrap {
                TextWrap::None => MEASURE_CACHE_PER_BUCKET_LIMIT_WRAP_NONE,
                TextWrap::Word | TextWrap::WordBreak | TextWrap::Grapheme => {
                    MEASURE_CACHE_PER_BUCKET_LIMIT
                }
            };
            while bucket.len() > limit {
                bucket.pop_front();
            }

            let mut metrics = metrics;
            if constraints.wrap == TextWrap::None
                && constraints.overflow == TextOverflow::Ellipsis
                && let Some(max_width) = constraints.max_width
            {
                metrics.size.width = max_width;
            }
            metrics
        }
    }
}
