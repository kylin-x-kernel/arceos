use core::sync::atomic::{AtomicUsize, Ordering};

use axhal::percpu::this_cpu_id;

/// Rendezvous phases.
///
/// - 0: idle
/// - 1: triggered, all CPUs must enter NMI and mark arrived
/// - 2: dump done (all non-cause CPUs can stop spinning if desired)
const PHASE_IDLE: usize = 0;
const PHASE_TRIGGERED: usize = 1;
const PHASE_DUMP_DONE: usize = 2;

static PHASE: AtomicUsize = AtomicUsize::new(PHASE_IDLE);

/// The CPU id which detected the failure and triggered the rendezvous.
static CAUSE_CPU: AtomicUsize = AtomicUsize::new(usize::MAX);

/// Per-cpu arrived bitmap stored in an AtomicUsize where bit i means CPU i arrived.
static ARRIVED_BITMAP: AtomicUsize = AtomicUsize::new(0);

#[inline]
pub fn is_triggered() -> bool {
    PHASE.load(Ordering::Acquire) == PHASE_TRIGGERED
}

#[inline]
pub fn is_dump_done() -> bool {
    PHASE.load(Ordering::Acquire) == PHASE_DUMP_DONE
}

/// Try to trigger rendezvous.
///
/// Returns `true` if this CPU became the *cause CPU*.
#[inline]
pub fn try_trigger() -> bool {
    let cpu = this_cpu_id();
    if PHASE
        .compare_exchange(PHASE_IDLE, PHASE_TRIGGERED, Ordering::AcqRel, Ordering::Relaxed)
        .is_ok()
    {
        CAUSE_CPU.store(cpu, Ordering::Release);
        true
    } else {
        false
    }
}

#[inline]
pub fn cause_cpu() -> Option<usize> {
    if PHASE.load(Ordering::Acquire) == PHASE_IDLE {
        return None;
    }
    let cpu = CAUSE_CPU.load(Ordering::Acquire);
    (cpu != usize::MAX).then_some(cpu)
}

/// Mark current cpu as arrived.
#[inline]
pub fn mark_arrived() {
    let id = this_cpu_id();
    if id >= usize::BITS as usize {
        // Cannot represent this CPU in the bitmap without overflowing the shift.
        return;
    }
    ARRIVED_BITMAP.fetch_or(1usize << id, Ordering::AcqRel);
}

#[inline]
pub fn arrived_bitmap() -> usize {
    ARRIVED_BITMAP.load(Ordering::Acquire)
}

#[inline]
pub fn all_arrived_mask() -> usize {
    let n = axconfig::plat::CPU_NUM;
    if n >= usize::BITS as usize {
        usize::MAX
    } else {
        (1usize << n) - 1
    }
}

/// Busy-wait until all CPUs have arrived.
///
/// This is a *strong* rendezvous: no timeout.
#[inline]
pub fn wait_all_arrived_strong() {
    let expect = all_arrived_mask();
    while arrived_bitmap() & expect != expect {
        core::hint::spin_loop();
    }
}

/// Mark dump done so other CPUs can release from spinning.
#[inline]
pub fn mark_dump_done() {
    PHASE.store(PHASE_DUMP_DONE, Ordering::Release);
}

/// Reset rendezvous state.
///
/// Note: in your intended flow, other CPUs may keep spinning in NMI forever.
/// If you want them to be able to return from NMI, call this after
/// `mark_dump_done()` and after ensuring they observed it.
#[inline]
pub fn reset() {
    ARRIVED_BITMAP.store(0, Ordering::Release);
    CAUSE_CPU.store(usize::MAX, Ordering::Release);
    PHASE.store(PHASE_IDLE, Ordering::Release);
}
