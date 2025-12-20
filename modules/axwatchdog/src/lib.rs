#![no_std]
pub mod nmi;
pub mod arch;

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "aarch64", feature = "pmu"))] {
        pub use crate::arch::aarch64::pmu::*;
    }
}