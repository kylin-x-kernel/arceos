use axhal::percpu::this_cpu_id;
use log::debug;

/// Initialize watchdog on primary cores
pub fn init_primary() {
    axhal::nmi::init(axhal::time::timer_frequency() * 10);
    axhal::nmi::enable();
    axhal::nmi::register_nmi_handler(|| {axtask::show_prev_task_backtrace();axtask::show_global_task_queue(this_cpu_id());} );
    debug!("watchdog init success on cpu {}", this_cpu_id());
}

/// Initialize watchdog on secondary cores
/// Called when each secondary core is brought up
pub fn init_secondary() {
    axhal::nmi::init(axhal::time::timer_frequency() * 10);
    axhal::nmi::enable();
    axhal::nmi::register_nmi_handler(|| {axtask::show_prev_task_backtrace();axtask::show_global_task_queue(this_cpu_id());});
    debug!("watchdog init success on cpu {}", this_cpu_id());
}
