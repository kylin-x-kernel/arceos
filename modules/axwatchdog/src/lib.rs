#![no_std]
pub mod nmi;

#[cfg(feature = "pmu")]
pub use nmi::pmu_nmi::{PMU_NMI,init};
#[cfg(feature = "sdei")]
pub use nmi::sdei_nmi::{init};
