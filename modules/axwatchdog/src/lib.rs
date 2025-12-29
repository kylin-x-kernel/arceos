#![no_std]
pub mod nmi;

pub use crate::nmi::{init_primary,init_secondary};