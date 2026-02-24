#[cfg(windows)]
use std::cell::Cell;

#[cfg(windows)]
use windows_sys::Win32::Foundation::FILETIME;
#[cfg(windows)]
use windows_sys::Win32::Foundation::HANDLE;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{GetCurrentThread, GetThreadTimes};
#[cfg(windows)]
use windows_sys::core::BOOL;

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn QueryThreadCycleTime(thread: HANDLE, cycle_time: *mut u64) -> BOOL;
}

#[cfg(windows)]
thread_local! {
    static LAST_THREAD_CPU_100NS: Cell<Option<u64>> = const { Cell::new(None) };
    static LAST_THREAD_CYCLES: Cell<Option<u64>> = const { Cell::new(None) };
    static CACHED_FRAME_ID: Cell<Option<u64>> = const { Cell::new(None) };
    static CACHED_DELTA_TIME_US: Cell<u64> = const { Cell::new(0) };
    static CACHED_TOTAL_TIME_US: Cell<u64> = const { Cell::new(0) };
    static CACHED_DELTA_CYCLES: Cell<u64> = const { Cell::new(0) };
    static CACHED_TOTAL_CYCLES: Cell<u64> = const { Cell::new(0) };
}

pub(super) struct UiThreadCpuSample {
    pub(super) delta_time_us: u64,
    pub(super) total_time_us: u64,
    pub(super) delta_cycles: u64,
    pub(super) total_cycles: u64,
}

pub(super) fn reset() {
    #[cfg(windows)]
    {
        LAST_THREAD_CPU_100NS.with(|slot| slot.set(None));
        LAST_THREAD_CYCLES.with(|slot| slot.set(None));
        CACHED_FRAME_ID.with(|slot| slot.set(None));
        CACHED_DELTA_TIME_US.with(|slot| slot.set(0));
        CACHED_TOTAL_TIME_US.with(|slot| slot.set(0));
        CACHED_DELTA_CYCLES.with(|slot| slot.set(0));
        CACHED_TOTAL_CYCLES.with(|slot| slot.set(0));
    }
}

pub(super) fn sample_current_thread(frame_id: u64) -> UiThreadCpuSample {
    #[cfg(windows)]
    {
        if CACHED_FRAME_ID.with(|slot| slot.get()) == Some(frame_id) {
            return UiThreadCpuSample {
                delta_time_us: CACHED_DELTA_TIME_US.with(|slot| slot.get()),
                total_time_us: CACHED_TOTAL_TIME_US.with(|slot| slot.get()),
                delta_cycles: CACHED_DELTA_CYCLES.with(|slot| slot.get()),
                total_cycles: CACHED_TOTAL_CYCLES.with(|slot| slot.get()),
            };
        }

        fn filetime_to_100ns(ft: FILETIME) -> u64 {
            ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
        }

        let mut creation: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut exit: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut kernel: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut user: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };

        let ok = unsafe {
            GetThreadTimes(
                GetCurrentThread(),
                &mut creation,
                &mut exit,
                &mut kernel,
                &mut user,
            )
        };
        if ok == 0 {
            return UiThreadCpuSample {
                delta_time_us: 0,
                total_time_us: 0,
                delta_cycles: 0,
                total_cycles: 0,
            };
        }

        let total_100ns = filetime_to_100ns(kernel).saturating_add(filetime_to_100ns(user));
        let total_us = total_100ns / 10;
        let delta_time_us = LAST_THREAD_CPU_100NS.with(|slot| {
            let prev = slot.get();
            slot.set(Some(total_100ns));
            prev.map_or(0, |prev| total_100ns.saturating_sub(prev) / 10)
        });

        let mut total_cycles: u64 = 0;
        let ok_cycles = unsafe { QueryThreadCycleTime(GetCurrentThread(), &mut total_cycles) };
        let delta_cycles = if ok_cycles == 0 {
            0
        } else {
            LAST_THREAD_CYCLES.with(|slot| {
                let prev = slot.get();
                slot.set(Some(total_cycles));
                prev.map_or(0, |prev| total_cycles.saturating_sub(prev))
            })
        };

        CACHED_FRAME_ID.with(|slot| slot.set(Some(frame_id)));
        CACHED_DELTA_TIME_US.with(|slot| slot.set(delta_time_us));
        CACHED_TOTAL_TIME_US.with(|slot| slot.set(total_us));
        CACHED_DELTA_CYCLES.with(|slot| slot.set(delta_cycles));
        CACHED_TOTAL_CYCLES.with(|slot| slot.set(total_cycles));
        return UiThreadCpuSample {
            delta_time_us,
            total_time_us: total_us,
            delta_cycles,
            total_cycles,
        };
    }

    #[cfg(not(windows))]
    {
        let _ = frame_id;
        UiThreadCpuSample {
            delta_time_us: 0,
            total_time_us: 0,
            delta_cycles: 0,
            total_cycles: 0,
        }
    }
}
