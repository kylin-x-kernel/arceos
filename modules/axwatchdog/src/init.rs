use axtask::{AxCpuMask, TaskInner};
use log::debug;

fn init_common() {
    init_softlockup_detection();
    axhal::nmi::init(axhal::time::timer_frequency() * 10);
    axhal::nmi::enable();
    axhal::nmi::register_nmi_handler(|| {
        if let Some(_failed_task_id) = crate::watchdog_task::check_watchdog_tasks() {
            axtask::show_curr_task_backtrace();
            axtask::show_global_task_queue(axhal::percpu::this_cpu_id());
        }
    });
    crate::register_hardlockup_detection_task();
    debug!(
        "watchdog init success on cpu {}",
        axhal::percpu::this_cpu_id()
    );
}

pub fn init_softlockup_detection() {
    axtask::register_timer_callback(|_| {
        let now_ns = axhal::time::monotonic_time_nanos();
        crate::timer_tick();
        if crate::check_softlockup(now_ns) {
            axtask::show_global_task_queue(axhal::percpu::this_cpu_id());
        }
    });

    let watchdog_task = TaskInner::new(
        move || loop {
            crate::touch_softlockup(axhal::time::monotonic_time_nanos());
            // axhal::time::busy_wait(axhal::time::Duration::from_nanos(1000));
            axtask::yield_now();
        },
        "watchdog".into(),
        axconfig::TASK_STACK_SIZE,
    );

    watchdog_task.set_cpumask(AxCpuMask::one_shot(axhal::percpu::this_cpu_id()));
    axtask::spawn_task(watchdog_task);
}

pub fn init_primary() {
    init_common();
}

pub fn init_secondary() {
    init_common();
}
