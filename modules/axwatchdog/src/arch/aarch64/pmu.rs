// =============================================================================
// PMU-based NMI Source
// =============================================================================
use crate::nmi::{NmiError, NmiResult, NmiSource};
use axhal::percpu::this_cpu_id;
use log::debug;
use aarch64_pmuv3::pmuv3::{PmuCounter as PmuNmiHw, PmuEvent};
use lazyinit::LazyInit;

#[percpu::def_percpu]
static WATCHDOG: LazyInit<PmuNmi> = LazyInit::new();
/// Handle PMU overflow.
///
/// Call this from your interrupt handler.
/// Returns true if overflow was handled.
fn pmu_irq_handler() -> bool {
    unsafe { WATCHDOG.current_ref_mut_raw().pmu().handle_overflow() }
}

pub fn init_primary(threshold: u64) -> Result<(), NmiError> {
    // 1. Construct and probe PMU hardware first (fallible)
    let pmu = PmuNmi::new_cycle_counter(threshold);
    pmu.init()?;      // Check PMU support
    pmu.enable()?;    // Enable PMU counter and interrupt

    // 2. Store into per-CPU LazyInit (must not fail)
    let cell = unsafe { WATCHDOG.current_ref_mut_raw() };
    cell.call_once(|| pmu);

    // Register interrupt handler on primary core only
    axhal::irq::set_priority(axconfig::devices::PMU_IRQ, 0);
    axhal::irq::register(axconfig::devices::PMU_IRQ, || {
        debug!("PMU NMI watchdog interrupt received on cpu {}", this_cpu_id());
        pmu_irq_handler();
    });

    Ok(())
}

/// Initialize watchdog on secondary cores
/// Called when each secondary core is brought up
pub fn init_secondary(threshold: u64) -> Result<(), NmiError> {
    // 1. Construct and probe PMU hardware first (fallible)
    let pmu = PmuNmi::new_cycle_counter(threshold);
    pmu.init()?;      // Check PMU support
    pmu.enable()?;    // Enable PMU counter and interrupt

    // 2. Store into per-CPU LazyInit (must not fail)
    let cell = unsafe { WATCHDOG.current_ref_mut_raw() };
    cell.call_once(|| pmu);

    // Set interrupt priority without registering a handler
    axhal::irq::set_priority(axconfig::devices::PMU_IRQ, 0);
    axhal::irq::set_enable(axconfig::devices::PMU_IRQ, true);

    Ok(())
}

/// PMU overflow-based NMI source.
pub struct PmuNmi{
    pmu: PmuNmiHw,
}

impl PmuNmi {
    /// Create a new PMU NMI source by cycle counter
    pub const fn new_cycle_counter(threshold: u64) -> Self {
        Self {
            pmu: PmuNmiHw::new_cycle_counter(threshold),
        }
    }

    /// Create a new PMU NMI source by event counter.
    pub const fn new_event_counter(index:u32,threshold: u64,event: PmuEvent) -> Self {
        Self {
            pmu: PmuNmiHw::new_event_counter(index,threshold,event),
        }
    }

    /// Get the underlying PMU interface.
    pub fn pmu(&self) -> &PmuNmiHw {
        &self.pmu
    }
}

impl NmiSource for PmuNmi {
    fn init(&self) -> NmiResult<()> {
        self.pmu.check_pmu_support().map_err(|_| NmiError::NotAvailable)
    }

    fn enable(&self) -> NmiResult<()> {
        self.pmu.enable().map_err(|_| NmiError::HardwareError)?;
        Ok(())
    }

    fn disable(&self) -> NmiResult<()> {
        self.pmu.disable().map_err(|_| NmiError::HardwareError)
    }

    fn is_enabled(&self) -> bool {
        self.pmu.is_enabled()
    }

    fn name(&self) -> &'static str {
        "PMU"
    }
}
