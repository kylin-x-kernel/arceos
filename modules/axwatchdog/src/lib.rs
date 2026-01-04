#![no_std]
pub mod init;
pub mod watchdog_task;

pub use crate::init::{init_primary, init_secondary};
