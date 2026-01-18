use fret_core::{AppWindowId, Event, Size, WindowMetricsService};

use crate::GlobalsHost;

pub fn apply_window_metrics_event(host: &mut impl GlobalsHost, window: AppWindowId, event: &Event) {
    match event {
        Event::WindowResized { width, height } => {
            let next = Size::new(*width, *height);
            if host
                .global::<WindowMetricsService>()
                .and_then(|svc| svc.inner_size(window))
                == Some(next)
            {
                return;
            }

            host.with_global_mut(WindowMetricsService::default, |svc, _host| {
                svc.set_inner_size(window, next);
            });
        }
        Event::WindowMoved(next) => {
            if host
                .global::<WindowMetricsService>()
                .and_then(|svc| svc.logical_position(window))
                == Some(*next)
            {
                return;
            }

            host.with_global_mut(WindowMetricsService::default, |svc, _host| {
                svc.set_logical_position(window, *next);
            });
        }
        Event::WindowFocusChanged(next) => {
            if host
                .global::<WindowMetricsService>()
                .and_then(|svc| svc.focused(window))
                == Some(*next)
            {
                return;
            }

            host.with_global_mut(WindowMetricsService::default, |svc, _host| {
                svc.set_focused(window, *next);
            });
        }
        Event::WindowScaleFactorChanged(next) => {
            if host
                .global::<WindowMetricsService>()
                .and_then(|svc| svc.scale_factor(window))
                == Some(*next)
            {
                return;
            }

            host.with_global_mut(WindowMetricsService::default, |svc, _host| {
                svc.set_scale_factor(window, *next);
            });
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestGlobalsHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        with_global_mut_calls: usize,
    }

    impl GlobalsHost for TestGlobalsHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            self.with_global_mut_calls += 1;

            let type_id = TypeId::of::<T>();
            let mut value = match self.globals.remove(&type_id) {
                Some(v) => *v
                    .downcast::<T>()
                    .expect("TestGlobalsHost stored wrong type"),
                None => init(),
            };

            let out = f(&mut value, self);

            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    #[test]
    fn window_metrics_does_not_touch_globals_for_irrelevant_events() {
        let mut host = TestGlobalsHost::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(1));

        apply_window_metrics_event(&mut host, window, &Event::WindowCloseRequested);

        assert_eq!(host.with_global_mut_calls, 0);
        assert!(host.global::<WindowMetricsService>().is_none());
    }

    #[test]
    fn window_metrics_skips_write_when_value_is_unchanged() {
        let mut host = TestGlobalsHost::default();
        let window = AppWindowId::from(slotmap::KeyData::from_ffi(2));

        let resize_a = Event::WindowResized {
            width: fret_core::Px(10.0),
            height: fret_core::Px(20.0),
        };

        apply_window_metrics_event(&mut host, window, &resize_a);
        assert_eq!(host.with_global_mut_calls, 1);

        apply_window_metrics_event(&mut host, window, &resize_a);
        assert_eq!(host.with_global_mut_calls, 1);

        let resize_b = Event::WindowResized {
            width: fret_core::Px(11.0),
            height: fret_core::Px(20.0),
        };
        apply_window_metrics_event(&mut host, window, &resize_b);
        assert_eq!(host.with_global_mut_calls, 2);
    }
}
