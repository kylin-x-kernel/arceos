use alloc::{boxed::Box, vec::Vec};

use kernel_guard::NoPreemptIrqSave;
use kspin::SpinRaw;

use axhal::time::{TimeValue, wall_time};

type TimerCb = Box<dyn Fn(TimeValue) + Send + Sync>;

static TIMER_CALLBACKS: SpinRaw<Vec<TimerCb>> = SpinRaw::new(Vec::new());

/// Registers a callback function to be called on each timer tick.
pub fn register_timer_callback<F>(callback: F)
where
    F: Fn(TimeValue) + Send + Sync + 'static,
{
    let _g = NoPreemptIrqSave::new();
    TIMER_CALLBACKS.lock().push(Box::new(callback));
}

pub fn check_events() {
    for callback in TIMER_CALLBACKS.lock().iter() {
        callback(wall_time());
    }
    crate::future::check_timer_events();
}
