use core::{
    arch::asm,
    convert::Infallible,
    future::Future,
    pin::Pin,
    sync::atomic::{ compiler_fence, AtomicUsize, Ordering },
    task::{ Context, Poll, Waker, RawWaker, RawWakerVTable },
};

static WAKE_BITS: AtomicUsize = AtomicUsize::new(0);

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    |p| RawWaker::new(p, &VTABLE),
    |p| {
        WAKE_BITS.fetch_or(p as _, Ordering::SeqCst);
    },
    |p| {
        WAKE_BITS.fetch_or(p as _, Ordering::SeqCst);
    },
    |_| ()
);

#[allow(dead_code)]
pub fn run_tasks(futures: &mut [Pin<&mut dyn Future<Output = Infallible>>]) -> ! {
    WAKE_BITS.swap(!0, Ordering::SeqCst);
    loop {
        let mask = WAKE_BITS.swap(0, Ordering::SeqCst);
        for (i, f) in futures.iter_mut().enumerate() {
            let wake_mask = 1 << i;
            if (mask & wake_mask) != 0 {
                let waker = unsafe { Waker::from_raw(RawWaker::new(wake_mask as _, &VTABLE)) };
                match f.as_mut().poll(&mut Context::from_waker(&waker)) {
                    Poll::Ready(_) => panic!(),
                    Poll::Pending => (),
                }
            }

            if WAKE_BITS.load(Ordering::SeqCst) == 0 {
                wfi();
                isb();
            }
        }
    }
}

/// Wait For Interrupt
#[inline(always)]
pub fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack, preserves_flags)) }
}

/// Instruction Synchronization Barrier
///
/// Flushes the pipeline in the processor, so that all instructions following the `ISB` are fetched
/// from cache or memory, after the instruction has been completed.
#[inline(always)]
pub fn isb() {
    compiler_fence(Ordering::SeqCst);
    unsafe {
        asm!("isb", options(nomem, nostack, preserves_flags))
    };
    compiler_fence(Ordering::SeqCst);
}
