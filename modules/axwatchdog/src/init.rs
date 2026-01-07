use core::sync::atomic::{AtomicBool, Ordering};

use axtask::{AxCpuMask, TaskInner};
use log::debug;

static HARDLOCKUP_REPORTED: AtomicBool = AtomicBool::new(false);

/// Common watchdog initialization for both primary and secondary CPUs.
///
/// It sets up:
/// - soft lockup detection (timer + watchdog task)
/// - hard lockup detection (PMU/NMI based)
fn init_common() {
    init_softlockup_detection();

    // Register hard lockup detection task.
    crate::register_hardlockup_detection_task();

    // Initialize and enable NMI source for hard lockup detection.
    axhal::nmi::init(axhal::time::timer_frequency() * 10);
    axhal::nmi::enable();

    // Register NMI handler
    axhal::nmi::register_nmi_handler(|| {
        if let Some(_failed_task_id) = crate::watchdog_task::check_watchdog_tasks() {
            // Only one CPU is allowed to dump globally
            if HARDLOCKUP_REPORTED
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                // Another CPU is already reporting
                return;
            }

            // (Optional) stop other CPUs via NMI_IPI
            // axhal::percpu::send_nmi_ipi_to_all_other_cpus();

            for i in 0..axconfig::plat::CPU_NUM {
                axtask::dump_cpu_task_stack(i);
            }

            HARDLOCKUP_REPORTED.store(false, Ordering::Release);

            // (Optional) panic after dumping all CPUs
            // panic!("Hard lockup detected");
        }
    });

    debug!(
        "watchdog init success on cpu {}",
        axhal::percpu::this_cpu_id()
    );
}

/// Initialize soft lockup detection.
///
/// A per-CPU watchdog task periodically updates a timestamp,
/// and timer callbacks check whether the timestamp is stale.
pub fn init_softlockup_detection() {
    // Timer callback used to detect soft lockup conditions.
    axtask::register_timer_callback(|_| {
        let now_ns = axhal::time::monotonic_time_nanos();
        crate::timer_tick();

        if crate::check_softlockup(now_ns) {
            axtask::dump_cpu_task_stack(axhal::percpu::this_cpu_id());
        }
    });

    // Watchdog task that periodically "touches" the soft lockup timestamp.
    let watchdog_task = TaskInner::new(
        move || loop {
            crate::touch_softlockup(axhal::time::monotonic_time_nanos());
            axhal::time::busy_wait(axhal::time::Duration::from_millis(40));
            axtask::yield_now();
        },
        "watchdog".into(),
        axconfig::TASK_STACK_SIZE,
    );

    // Bind watchdog task to the local CPU.
    watchdog_task.set_cpumask(AxCpuMask::one_shot(axhal::percpu::this_cpu_id()));
    axtask::spawn_task(watchdog_task);
}

pub fn init_primary() {
    init_common();
}

pub fn init_secondary() {
    init_common();
}
