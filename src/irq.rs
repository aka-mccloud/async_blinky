use core::{ future::Future, pin::Pin, task::{ Context, Poll, Waker } };

use rusty_peripheral::{ cortex_m::Peripherals, exti::{ line::{Line, LineMask}, EXTI } };

pub fn wait_for_irq(irqn: i16) -> IRQFuture {
    unsafe {
        for irq in &mut IRQ_WAKERS {
            if irq.is_none() {
                irq.replace(IRQState::new(irqn));
                return IRQFuture { irq };
            }
        }
    }

    panic!()
}

pub fn wait_for_exti_irq(line: Line) -> IRQFuture {
    let irqn = irqn_from_exti_line(line);
    wait_for_irq(irqn)
}

struct IRQState {
    number: i16,
    pending: bool,
    waker: Option<Waker>,
}

impl IRQState {
    fn new(irqn: i16) -> Self {
        Self {
            number: irqn,
            pending: false,
            waker: None,
        }
    }
}

pub struct IRQFuture {
    irq: &'static mut Option<IRQState>,
}

impl Future for IRQFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.get_mut().irq {
            Some(state) => {
                match state.pending {
                    true => Poll::Ready(()),
                    false => {
                        state.waker.replace(cx.waker().clone());
                        Poll::Pending
                    }
                }
            }
            None => panic!(),
        }
    }
}

impl Drop for IRQFuture {
    fn drop(&mut self) {
        self.irq.take();
    }
}

#[no_mangle]
extern "C" fn __default_irq_handler() {
    let irqn = Peripherals::scb().get_active_interrupt_number();
    Peripherals::nvic().irq_clear_pending(irqn as usize);

    if let Some(line) = irqn_to_exti_line(irqn) {
        let exti = EXTI::get();
        let mask = exti.pr.get_pending_interrupts();
        if (mask & line) != 0.into() {
            exti.pr.clear_pending_interrupts(mask | line);
        }
    }

    unsafe {
        for irq in &mut IRQ_WAKERS {
            if let Some(state) = irq {
                if state.number == irqn {
                    state.pending = true;
                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                }
            }
        }
    }
}

#[inline(always)]
fn irqn_from_exti_line(line: Line) -> i16 {
    match line {
        Line::LINE0 => 6,
        Line::LINE1 => 7,
        Line::LINE2 => 8,
        Line::LINE3 => 9,
        Line::LINE4 => 10,
        Line::LINE5 => 23,
        Line::LINE6 => 23,
        Line::LINE7 => 23,
        Line::LINE8 => 23,
        Line::LINE9 => 23,
        Line::LINE10 => 40,
        Line::LINE11 => 40,
        Line::LINE12 => 40,
        Line::LINE13 => 40,
        Line::LINE14 => 40,
        Line::LINE15 => 40,
        Line::LINE16 => 1,
        Line::LINE17 => 41,
        Line::LINE18 => 42,
        Line::LINE19 => 62,
        Line::LINE20 => 76,
        Line::LINE21 => 2,
        Line::LINE22 => 3,
    }
}

#[inline(always)]
fn irqn_to_exti_line(irqn: i16) -> Option<LineMask> {
    match irqn {
        1 => Some(Line::LINE16.into()),
        2 => Some(Line::LINE21.into()),
        3 => Some(Line::LINE22.into()),
        6 => Some(Line::LINE0.into()),
        7 => Some(Line::LINE1.into()),
        8 => Some(Line::LINE2.into()),
        9 => Some(Line::LINE3.into()),
        10 => Some(Line::LINE4.into()),
        23 => Some(Line::LINE5 | Line::LINE6 | Line::LINE7 | Line::LINE8 | Line::LINE9),
        40 => Some(Line::LINE10 | Line::LINE11 | Line::LINE12 | Line::LINE13 | Line::LINE14 | Line::LINE15),
        41 => Some(Line::LINE17.into()),
        42 => Some(Line::LINE18.into()),
        62 => Some(Line::LINE19.into()),
        76 => Some(Line::LINE20.into()),
        _ => None,
    }
}

static mut IRQ_WAKERS: [Option<IRQState>; 32] = [
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];
