extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

#[percpu::def_percpu]
pub(crate) static WATCHDOG_TASK_QUEUE: Vec<Box<dyn WatchdogTask>> = Vec::new();

/// Watchdog task trait.
pub trait WatchdogTask {
    /// Unique identifier for the task (e.g. name or ID).
    /// Keep it simple in no_std environments.
    fn id(&self) -> &str;

    /// Check whether the task is healthy.
    /// Return `true` if healthy, `false` to trigger recovery actions.
    fn check(&self) -> bool;
}

/// Register a watchdog task for the current CPU.
///
/// This function adds the task into the per-CPU watchdog task queue.
pub fn register_watchdog_task(task: Box<dyn WatchdogTask>) {
    unsafe {
        let queue = WATCHDOG_TASK_QUEUE.current_ref_mut_raw();
        queue.push(task);
    }
}

/// Check watchdog tasks and return the first failed task ID if any.
pub(crate) fn check_watchdog_tasks() -> Option<&'static str> {
    unsafe {
        let queue = WATCHDOG_TASK_QUEUE.current_ref_mut_raw();
        for task in queue.iter() {
            if !task.check() {
                return Some(task.id());
            }
        }
        None
    }
}
