use axhal::percpu::this_cpu_id;
use log::debug;

/// Hard lockup detection threshold: 10 seconds at 1GHz
pub const HARD_LOCKUP_THRESHOLD: u64 = 0x0000_0002_540B_E400;

/// Initialize watchdog on primary cores
pub fn init_primary(threshold: u64){
    axhal::nmi::init(threshold);
    axhal::nmi::enable();
    // Register interrupt handler on primary core only
    axhal::irq::set_priority(axconfig::devices::PMU_IRQ, 0);
    axhal::irq::register(axconfig::devices::PMU_IRQ, || {
        debug!("PMU NMI watchdog interrupt received on cpu {}", this_cpu_id());
        axhal::nmi::handle();
    });
    debug!("watchdog init success on cpu {}", this_cpu_id());
}

/// Initialize watchdog on secondary cores
/// Called when each secondary core is brought up
pub fn init_secondary(threshold: u64){
    axhal::nmi::init(threshold);
    axhal::nmi::enable();
    // Set interrupt priority without registering a handler
    axhal::irq::set_priority(axconfig::devices::PMU_IRQ, 0);
    axhal::irq::set_enable(axconfig::devices::PMU_IRQ, true);
    debug!("watchdog init success on cpu {}", this_cpu_id());
}