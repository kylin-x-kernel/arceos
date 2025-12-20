# axwatchdog

A Non-Maskable Interrupt (NMI) based hard lockup detection watchdog for system monitoring.

## Overview

axwatchdog is a hard lockup detection implementation that uses NMI mechanisms to periodically trigger interrupts and monitor system state. When a hard lockup occurs, the watchdog can trigger appropriate handling mechanisms.

## Features

- **Multiple NMI Sources**:
  - PMU (Performance Monitoring Unit) overflow interrupts (implemented)
  - SDEI (Software Delegated Exception Interface) NMI (planned)
- **Multi-core Support**: Runs in SMP (Symmetric Multi-Processing) environments
- **Configurable Thresholds**: Customizable lockup detection time thresholds
- **No-std Compatibility**: Pure `no_std` environment operation

## NMI Sources

### PMU NMI Source (Current Implementation)

Based on ARMv8 PMU cycle counter overflow mechanism:
- Uses PMU cycle counters to monitor CPU activity
- Triggers NMI interrupts on counter overflow
- Supports custom overflow thresholds (cycle counts)

### SDEI NMI Source (Planned)

Based on ARM SDEI software-delegated NMI mechanism.

## Usage

### Initialization

```rust
use axwatchdog::nmi::{HARD_LOCKUP_THRESHOLD, init_primary, init_secondary};
// Initialize on primary core
init_primary(HARD_LOCKUP_THRESHOLD)?;
// Initialize on secondary cores (call when each secondary core boots)
init_secondary(HARD_LOCKUP_THRESHOLD)?;
```

## Configuration

### Compile-time Configuration

Enable features in `Cargo.toml`:

```toml
[dependencies.axwatchdog]
features = ["pmu"] # Enable PMU support
```

## Hardware Requirements

### PMU NMI Source
- ARMv8-A architecture (AArch64)
- PMUv3-compatible processor
- Performance Monitoring Unit support

### SDEI NMI Source (Planned)
- ARM SDEI compatible firmware/hypervisor

## Error Handling

Operations return `NmiError` type:

```rust
pub enum NmiError {
    NotAvailable, // NMI source not available
    NotInitialized, // NMI source not initialized
    AlreadyInitialized, // NMI source already initialized
    InvalidConfig, // Invalid configuration parameter
    HandlerExists, // Handler already registered
    NoHandler, // No handler registered
    NotSupported, // Operation not supported
    HardwareError, // Hardware error
}
```

## Notes

- Currently supports only AArch64 architecture
- PMU support requires `pmu` feature enabled
- Interrupt priority set to highest (0)
- Requires initialization on each core in multi-core systems

## Development Status

- ✅ PMU NMI Source: Implemented
- 🔄 SDEI NMI Source: In Development
- ✅ Multi-core Support: Implemented