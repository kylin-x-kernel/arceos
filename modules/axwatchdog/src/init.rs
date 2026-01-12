use core::sync::atomic::{AtomicBool, Ordering};

use axtask::{AxCpuMask, TaskInner};
use log::debug;

static REPORTED: AtomicBool = AtomicBool::new(false);

/// Common watchdog initialization for both primary and secondary CPUs.
///
/// It sets up:
/// - soft lockup detection (timer + watchdog task)
/// - hard lockup detection (PMU/NMI based)
fn init_common() {
    init_softlockup_detection();

    // Register hard lockup detection task.
    crate::register_hardlockup_detection_task();

    // Register mutex deadlock check
    crate::register_watchdog_task(&crate::watchdog_task::MUTEX_DEADLOCK_CHECK);

    // Initialize and enable NMI source for hard lockup detection.
    axhal::nmi::init(axhal::time::timer_frequency() * 10 * 16);
    axhal::nmi::enable();

    // Register NMI handler
    axhal::nmi::register_nmi_handler(|tf| {
        if let Some(_failed_task_id) = crate::watchdog_task::check_watchdog_tasks() {
            axtask::dump_cur_task_backtrace(tf);
            // Only one CPU is allowed to dump globally
            if REPORTED
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                // Another CPU is already reporting
                return;
            }

            // (Optional) stop other CPUs via NMI_IPI
            // axhal::percpu::send_nmi_ipi_to_all_other_cpus();
            for i in 0..axconfig::plat::CPU_NUM{
                axtask::dump_cpu_task_backtrace(i);
            }
            

            // panic after dumping all CPUs
            panic!("Watchdog task check failed");
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
            axtask::dump_cpu_task_backtrace(axhal::percpu::this_cpu_id());
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
    // init_test1();
    init_common();
}

pub fn init_secondary() {
    // init_test2();
    init_common();
}

/*
static L1: SpinNoIrq<u8> = SpinNoIrq::new(1);

static L2: SpinNoIrq<u8> = SpinNoIrq::new(2);

pub fn init_test1() {
    // Watchdog task that periodically "touches" the soft lockup timestamp.
    let watchdog_task = TaskInner::new(
        move || {
                    let l2 = L2.lock();
                    warn!("cpu {} get L2 lock",axhal::percpu::this_cpu_id());
                    axhal::time::busy_wait(axhal::time::Duration::from_secs(30));
                    let l1 = L1.lock();
                    warn!("cpu {} get L1 lock",axhal::percpu::this_cpu_id());
                    warn!("{:?}{:?}",l1,l2);
        },
        "test1".into(),
        axconfig::TASK_STACK_SIZE,
    );
    // Bind watchdog task to the local CPU.
    watchdog_task.set_cpumask(AxCpuMask::one_shot(axhal::percpu::this_cpu_id()));
    axtask::spawn_task(watchdog_task);
}

#[inline(never)]
fn test2_task_fn() {
    use axhal::time::Duration;
    use axhal::time::busy_wait;
    use axhal::percpu::this_cpu_id;
    let l1 = L1.lock();
    warn!("cpu {} get L1 lock", this_cpu_id());
    busy_wait(Duration::from_secs(30));
    let l2 = L2.lock();
    warn!("cpu {} get L2 lock", this_cpu_id());
    warn!("{:?}{:?}", l1, l2);
}


pub fn init_test2() {

    let watchdog_task = TaskInner::new(test2_task_fn, "test2".into(), axconfig::TASK_STACK_SIZE);
    // Bind watchdog task to the local CPU.
    watchdog_task.set_cpumask(AxCpuMask::one_shot(axhal::percpu::this_cpu_id()));
    axtask::spawn_task(watchdog_task);
}

use axsync::Mutex;

static M1: Mutex<u8> = Mutex::new(1);
static M2: Mutex<u8> = Mutex::new(2);

pub fn init_test1() {
    let t1 = TaskInner::new(
        move || {
            let _m2 = M2.lock();
            axhal::time::busy_wait(axhal::time::Duration::from_secs(1));
            let _m1 = M1.lock();
        },
        "test_mutex_21".into(),
        axconfig::TASK_STACK_SIZE,
    );

    t1.set_cpumask(AxCpuMask::one_shot(axhal::percpu::this_cpu_id()));
    axtask::spawn_task(t1);
}

#[inline(never)]
fn test2_task_fn() {
    let _m1 = M1.lock();
    axhal::time::busy_wait(axhal::time::Duration::from_secs(1));
    let _m2 = M2.lock();
}

pub fn init_test2() {
    let t2 = TaskInner::new(
        test2_task_fn,
        "test_mutex_12".into(),
        axconfig::TASK_STACK_SIZE,
    );

    t2.set_cpumask(AxCpuMask::one_shot(axhal::percpu::this_cpu_id()));
    axtask::spawn_task(t2);
}
*/