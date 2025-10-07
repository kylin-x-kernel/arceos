use core::{
    pin::Pin,
    task::{Context, Poll},
};

use axerrno::{AxError, AxResult};
use axpoll::{IoEvents, Pollable};

/// A helper to wrap a [`Pollable`] and a synchronous non-blocking I/O function
/// into a [`Future`].
pub struct Poller<'a, P, F> {
    pollable: &'a P,
    events: IoEvents,
    f: F,
    non_blocking: bool,
}

impl<'a, P, F> Poller<'a, P, F> {
    /// Creates a new [`Poller`].
    pub fn new(pollable: &'a P, events: IoEvents, f: F) -> Self {
        Poller {
            pollable,
            events,
            f,
            non_blocking: false,
        }
    }

    /// Sets whether the poller should operate in non-blocking mode.
    pub fn non_blocking(mut self, non_blocking: bool) -> Self {
        self.non_blocking = non_blocking;
        self
    }
}

impl<'a, P: Pollable, F: FnMut() -> AxResult<T> + Unpin, T> Future for Poller<'a, P, F> {
    type Output = AxResult<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match (self.f)() {
            Ok(value) => Poll::Ready(Ok(value)),
            Err(AxError::WouldBlock) => {
                if self.non_blocking {
                    return Poll::Ready(Err(AxError::WouldBlock));
                }
                self.pollable.register(cx, self.events);
                match (self.f)() {
                    Ok(value) => Poll::Ready(Ok(value)),
                    Err(AxError::WouldBlock) => Poll::Pending,
                    Err(e) => Poll::Ready(Err(e)),
                }
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

#[cfg(feature = "irq")]
/// Registers a waker for the given IRQ number.
pub fn register_irq_waker(irq: usize, waker: &core::task::Waker) {
    use alloc::collections::{BTreeMap, btree_map::Entry};
    use axpoll::PollSet;
    use kspin::SpinNoIrq;

    static POLL_IRQ: SpinNoIrq<BTreeMap<usize, PollSet>> = SpinNoIrq::new(BTreeMap::new());

    fn irq_handler(irq: usize) {
        let s = POLL_IRQ.lock().remove(&irq);
        if let Some(s) = s {
            s.wake();
        }
    }

    match POLL_IRQ.lock().entry(irq) {
        Entry::Vacant(e) => {
            axhal::irq::register(irq, irq_handler);
            e.insert(PollSet::new())
        }
        Entry::Occupied(e) => e.into_mut(),
    }
    .register(waker);
}
