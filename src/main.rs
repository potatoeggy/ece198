#![no_std]
#![no_main]

extern crate panic_halt;

use cortex_m_rt::entry;
use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode, HD44780};
use keypad2::Keypad;
use stm32f4xx_hal::{pac, prelude::*};

// Connections:
// GND: GND
// VDD: 5V
// V0:  10k poti between 5V and GND
// RS:  PB7
// RW:  GND
// E:   PB8
// D4-D7: PB6-PB3
// A:   5V
// K:   GND

// Keypad connections:
// from left to right:
// D0
// A5
// A4
// A3
// A2
// A1
// A0

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let gpiob = dp.GPIOB.split();
    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();

    let clocks = rcc.cfgr.freeze();
    let mut delay = dp.TIM1.delay_us(&clocks);

    let rows = (
        gpiob.pb0.into_pull_up_input(),
        gpioa.pa4.into_pull_up_input(),
        gpioa.pa0.into_pull_up_input(),
        gpioa.pa1.into_pull_up_input(),
    );
    let cols = (
        gpioa.pa3.into_open_drain_output(),
        gpioc.pc0.into_open_drain_output(),
        gpioc.pc1.into_open_drain_output(),
    );

    let mut keypad = Keypad::new(rows, cols);

    let rs = gpioa.pa8.into_push_pull_output();
    let en = gpiob.pb10.into_push_pull_output();
    let d4 = gpiob.pb5.into_push_pull_output();
    let d5 = gpiob.pb4.into_push_pull_output();
    let d6 = gpiob.pb3.into_push_pull_output();
    let d7 = gpioa.pa10.into_push_pull_output();

    let mut lcd = HD44780::new_4bit(rs, en, d4, d5, d6, d7, &mut delay).unwrap();
    lcd.reset(&mut delay).unwrap();
    lcd.clear(&mut delay).unwrap();
    lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Visible,
            cursor_blink: CursorBlink::On,
        },
        &mut delay,
    )
    .unwrap();
    lcd.write_str("Booting...", &mut delay).unwrap();
    lcd.set_cursor_pos(40, &mut delay).unwrap();
    lcd.write_str("Num2", &mut delay).unwrap();

    let mut led = gpioa.pa5.into_push_pull_output();
    #[allow(clippy::empty_loop)]
    loop {
        delay.delay_ms(500_u16);
        let key = keypad.read_char(&mut delay);
        led.set_high();
        if key != ' ' {
            lcd.reset(&mut delay).unwrap();
            lcd.write_char(key, &mut delay).unwrap();
        }
    }
}

/*
#![deny(unsafe_code)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use hal::{
    gpio::{self, Output, PushPull},
    timer::CounterUs,
};
use libm::sqrt;
// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4::stm32f401::TIM2;
use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*, timer::Channel};

type LedPin = gpio::PA5<Output<PushPull>>;

static G_LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));
static G_TIM: Mutex<RefCell<Option<CounterUs<TIM2>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the LED. On the Nucleo-446RE it"s connected to pin PA5.
        let gpioa = dp.GPIOA.split();
        let gpiod = dp.GPIOD.split();
        let mut led = gpioa.pa5.into_push_pull_output();

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        // Create a delay abstraction based on SysTick
        let mut delay = cp.SYST.delay(&clocks);

        /*
        let buzzer = gpioa.pa9.into_alternate();
        let mut buzz_pwm = dp.TIM1.pwm_hz(buzzer, 2000.Hz(), &clocks);

        let max_duty = buzz_pwm.get_max_duty();
        buzz_pwm.set_duty(Channel::C2, max_duty / 2);
        */

        loop {
            led.set_high();
            delay.delay_ms(1000_u32);
            led.set_low();
            delay.delay_ms(1000_u32);
        }
    }

    loop {}
}

fn calc_mean(data: &[f64]) -> f64 {
    let mut total = 0.0;
    for &val in data {
        total += val;
    }
    total / (data.len() as f64)
}

fn calc_stdev(data: &[f64]) -> f64 {
    let len = data.len() as f64;
    let mut total = 0.0;
    for &val in data {
        total += val;
    }

    let mean = total / len;

    let mut sum = 0.0;
    for &val in data {
        sum += (val - mean) * (val - mean);
    }

    let variance = sum / len;
    sqrt(variance)
}

fn calc_median(data: &mut [f64]) -> f64 {
    bubble_sort(data);

// TODO: finish
    0.0
}

fn bubble_sort(data: &mut [f64]) {
// no builtin sorting in rust without std
    let mut new_len: usize;
    let mut len = data.len();

    loop {
        new_len = 0;
        for i in 1..len {
            if data[i - 1] > data[i] {
                data.swap(i - 1, i);
                new_len = i;
            }
        }

        if new_len == 0 {
            break;
        }

        len = new_len;
    }
}
*/
