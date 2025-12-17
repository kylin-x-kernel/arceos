//! NMI (Non-Maskable Interrupt) source abstraction.
//!
//! This module defines the trait for NMI sources that can be used to trigger
//! watchdog checks. Different hardware mechanisms (SDEI, PMU overflow, etc.)
//! can implement this trait.

/// Error type for NMI operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NmiError {
    /// NMI source not available on this platform.
    NotAvailable,
    /// NMI source not initialized.
    NotInitialized,
    /// NMI source already initialized.
    AlreadyInitialized,
    /// Invalid configuration parameter.
    InvalidConfig,
    /// Handler already registered.
    HandlerExists,
    /// No handler registered.
    NoHandler,
    /// Operation not supported.
    NotSupported,
    /// Hardware error.
    HardwareError,
}

impl core::fmt::Display for NmiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NmiError::NotAvailable => write!(f, "NMI source not available"),
            NmiError::NotInitialized => write!(f, "NMI source not initialized"),
            NmiError::AlreadyInitialized => write!(f, "NMI source already initialized"),
            NmiError::InvalidConfig => write!(f, "Invalid configuration"),
            NmiError::HandlerExists => write!(f, "Handler already registered"),
            NmiError::NoHandler => write!(f, "No handler registered"),
            NmiError::NotSupported => write!(f, "Operation not supported"),
            NmiError::HardwareError => write!(f, "Hardware error"),
        }
    }
}

/// Result type for NMI operations.
pub type NmiResult<T> = Result<T, NmiError>;

/// Trait for NMI sources.
///
/// Implementors provide a mechanism to trigger NMI-like interrupts
/// at regular intervals for watchdog purposes.
pub trait NmiSource: Send + Sync {
    /// Initialize the NMI source.
    ///
    /// This should configure the hardware but not start triggering.
    fn init(&self) -> NmiResult<()>;

    /// Enable NMI generation.
    ///
    /// After this call, NMIs will be triggered at the configured period.
    fn enable(&self) -> NmiResult<()>;

    /// Disable NMI generation.
    fn disable(&self) -> NmiResult<()>;

    /// Check if NMI generation is currently enabled.
    fn is_enabled(&self) -> bool;

    /// Get the name of this NMI source (for debugging).
    fn name(&self) -> &'static str;
}

// =============================================================================
// SDEI-based NMI Source
// =============================================================================

#[cfg(feature = "sdei")]
pub mod sdei_nmi {
    //! SDEI-based NMI implementation.

    use aarch64_sdei::{SDEI_EVENT_SOFTWARE_NMI, Sdei};

    use super::*;

    /// SDEI-based NMI source.
    pub struct SdeiNmi {
        sdei: Sdei,
        storage: NmiHandlerStorage,
    }

    impl SdeiNmi {
        /// Create a new SDEI NMI source.
        pub const fn new() -> Self {
            todo!()
        }

        /// Get the underlying SDEI interface.
        pub fn sdei(&self) -> &Sdei {
            todo!()
        }
    }

    impl NmiSource for SdeiNmi {
        fn init(&self) -> NmiResult<()> {
            todo!()
        }

        fn enable(&self) -> NmiResult<()> {
            todo!()
        }

        fn disable(&self) -> NmiResult<()> {
            todo!()
        }

        fn is_enabled(&self) -> bool {
            todo!()
        }

        fn name(&self) -> &'static str {
            "SDEI"
        }
    }

    impl Default for SdeiNmi {
        fn default() -> Self {
            Self::new()
        }
    }
}

// =============================================================================
// PMU-based NMI Source
// =============================================================================

#[cfg(feature = "pmu")]
pub mod pmu_nmi {
    //! PMU overflow-based NMI implementation.

    use aarch64_pmuv3::pmuv3::{PmuCounter as PmuNmiHw, PmuEvent};

    use super::*;

    /// PMU overflow-based NMI source.
    pub struct PmuNmi{
        pmu: PmuNmiHw,
    }

    pub static PMU_NMI: PmuNmi = PmuNmi::new_cycle_counter( 0x0000_0000_f000_0000);

    /// todo: handle error
    pub fn init(){
        let _ = PMU_NMI.init();
        let _ = PMU_NMI.enable();
        //axhal::irq::set_priority(axconfig::devices::PMU_IRQ, 0);
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

        /// Handle PMU overflow.
        ///
        /// Call this from your interrupt handler.
        /// Returns true if overflow was handled.
        pub fn handle_overflow(&self) -> bool {
            if self.pmu.handle_overflow() {
                true
            } else {
                false
            }
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
}
