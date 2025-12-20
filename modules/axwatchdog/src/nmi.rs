//! NMI (Non-Maskable Interrupt) source abstraction.
//!
//! This module defines the trait for NMI sources that can be used to trigger
//! watchdog checks. Different hardware mechanisms (SDEI, PMU overflow, etc.)
//! can implement this trait.

/// Hard lockup detection threshold: 10 seconds at 1GHz
pub const HARD_LOCKUP_THRESHOLD:u64 = 0x0000_0002_540B_E400;

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