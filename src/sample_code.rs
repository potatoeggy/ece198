#![deny(unsafe_code)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

#[entry]
fn main() -> ! {
    if let Some(dp) = cortex_m::peripheral::Peripherals::take() {
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();
    }
}
