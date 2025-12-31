use axhal::percpu::this_cpu_id;
use log::{debug, warn};

/// Hard lockup detection threshold: 10 seconds at 1GHz
pub const HARD_LOCKUP_THRESHOLD: u64 = 0x0000_0002_540B_E400;

/// Initialize watchdog on primary cores
pub fn init_primary(threshold: u64) {
    axhal::nmi::init(threshold);
    axhal::nmi::enable();
    axhal::nmi::register_nmi_handler(|| { warn!("nmi handler")});
    debug!("watchdog init success on cpu {}", this_cpu_id());
}

/// Initialize watchdog on secondary cores
/// Called when each secondary core is brought up
pub fn init_secondary(threshold: u64) {
    axhal::nmi::init(threshold);
    axhal::nmi::enable();
    axhal::nmi::register_nmi_handler(|| { warn!("nmi handler")});
    debug!("watchdog init success on cpu {}", this_cpu_id());
}
