#![no_std]
#![no_main]

mod executor;
mod irq;

use core::{ convert::Infallible, pin::pin };

use irq::wait_for_exti_irq;
use rusty_macros::main;
use rusty_peripheral::{
    cortex_m::Peripherals, exti::line::Line, gpio::{
        self,
        pin::Pin,
        port::Port,
        InterruptType,
        OutputType,
        PinConfig,
        PinSpeed,
        PullUpPullDown,
    }
};

extern crate rusty_rt;

#[main]
fn main() -> ! {
    let async_main = pin!(async_main());

    executor::run_tasks(&mut [async_main]);
}

async fn async_main() -> Infallible {
    let mut gpio_a = gpio::get_port(Port::A);
    let mut gpio_g = gpio::get_port(Port::G);

    gpio_a.enable_clock();
    gpio_g.enable_clock();

    gpio_a.init_pins(
        Pin::PIN0,
        PinConfig::Input(PinSpeed::High, PullUpPullDown::None, InterruptType::RisingEdge)
    );

    gpio_g.init_pins(
        Pin::PIN13,
        PinConfig::Output(OutputType::PushPull, PinSpeed::High, PullUpPullDown::None)
    );

    let mut nvic = Peripherals::nvic();
    nvic.irq_enable(6);
    nvic.irq_set_priority(6, 13);

    loop {
        wait_for_exti_irq(Line::LINE0).await;
        gpio_g.toggle_pins(Pin::PIN13);
    }
}
